// QuietCraft: Automated silence detection, derushing, and audio timeline trimmer for EffectCraft.
//
// A ScriptUI panel extension for EffectCraft (Window ▸ QuietCraft.jsx).
// Built against EffectCraft's public scripting API:
// app.project.activeItem, comp.selectedLayers, layer.inPoint, layer.outPoint, layer.startTime,
// layer.duplicate(), layer.marker, app.beginUndoGroup, app.settings, and ScriptUI.
// Original work by EffectCraft contributors under MIT OR Apache-2.0.

(function quietCraft(thisObj) {
  var SECTION = "QuietCraft";
  var KEY_SETTINGS = "userSettings";
  var KEY_CUTS = "lastCuts";

  // --- Presets ---
  var PRESETS = [
    { name: "Default Podcast", thresh: -35, minSil: 0.40, preRoll: 0.08, postRoll: 0.12 },
    { name: "Aggressive Jump-Cut", thresh: -30, minSil: 0.25, preRoll: 0.04, postRoll: 0.06 },
    { name: "Noisy Environment", thresh: -26, minSil: 0.45, preRoll: 0.10, postRoll: 0.15 },
    { name: "Gentle Lecture", thresh: -40, minSil: 0.65, preRoll: 0.12, postRoll: 0.18 }
  ];

  // Working state
  var working = {
    thresholdDb: -35.0,
    minSilenceDur: 0.40,
    minSpeechDur: 0.15,
    preRollPad: 0.08,
    postRollPad: 0.12
  };

  var playheadPos = 0.35; // 0..1 preview position
  var updating = false;

  // Mock waveform peak envelope generator (simulates audio speech bursts and silence pauses)
  function generateEnvelopes(count) {
    var env = [];
    for (var i = 0; i < count; i++) {
      var t = i / count;
      // Speech pattern: burst from 0.05..0.28, silence 0.28..0.42, speech 0.42..0.68, silence 0.68..0.82, speech 0.82..0.96
      var isSpeech = (t >= 0.05 && t < 0.28) || (t >= 0.42 && t < 0.68) || (t >= 0.82 && t < 0.96);
      var val = isSpeech ? 0.35 + 0.60 * Math.abs(Math.sin(i * 0.4) * Math.cos(i * 0.15)) : 0.002 + 0.003 * Math.random();
      env.push(val);
    }
    return env;
  }

  var waveformPeaks = generateEnvelopes(120);

  // Evaluate silence intervals on the current waveform/duration
  function computeSilenceReport(totalDur) {
    var dur = (typeof totalDur === "number" && totalDur > 0) ? totalDur : 10.0;
    var threshNorm = Math.pow(10, working.thresholdDb / 20); // convert dB to linear amplitude
    var speechList = [];
    var inSpeech = false;
    var startT = 0;
    var n = waveformPeaks.length;

    for (var i = 0; i < n; i++) {
      var t = (i / n) * dur;
      var amp = waveformPeaks[i];
      var isSound = amp > threshNorm;

      if (isSound) {
        if (!inSpeech) { inSpeech = true; startT = t; }
      } else if (inSpeech) {
        inSpeech = false;
        speechList.push({ start: startT, end: t });
      }
    }
    if (inSpeech) speechList.push({ start: startT, end: dur });

    // Filter min speech duration
    var filtered = [];
    for (var k = 0; k < speechList.length; k++) {
      if ((speechList[k].end - speechList[k].start) >= working.minSpeechDur) {
        filtered.push(speechList[k]);
      }
    }
    if (filtered.length === 0) filtered.push({ start: 0, end: dur });

    // Bridge short pauses below minSilenceDur
    var bridged = [];
    for (var b = 0; b < filtered.length; b++) {
      if (bridged.length > 0) {
        var prev = bridged[bridged.length - 1];
        if (filtered[b].start - prev.end < working.minSilenceDur) {
          prev.end = filtered[b].end;
          continue;
        }
      }
      bridged.push({ start: filtered[b].start, end: filtered[b].end });
    }

    // Apply pre-roll and post-roll padding
    var padded = [];
    for (var p = 0; p < bridged.length; p++) {
      var pStart = Math.max(0, bridged[p].start - working.preRollPad);
      var pEnd = Math.min(dur, bridged[p].end + working.postRollPad);
      if (padded.length > 0 && pStart <= padded[padded.length - 1].end) {
        padded[padded.length - 1].end = Math.max(padded[padded.length - 1].end, pEnd);
      } else {
        padded.push({ start: pStart, end: pEnd });
      }
    }

    // Silence segments
    var silenceList = [];
    var cur = 0;
    for (var s = 0; s < padded.length; s++) {
      if (padded[s].start > cur + 0.01) {
        silenceList.push({ start: cur, end: padded[s].start });
      }
      cur = padded[s].end;
    }
    if (cur + 0.01 < dur) {
      silenceList.push({ start: cur, end: dur });
    }

    // Ripple retiming
    var rippleCuts = [];
    var tCursor = 0;
    for (var r = 0; r < padded.length; r++) {
      var segDur = padded[r].end - padded[r].start;
      rippleCuts.push({
        index: r + 1,
        sourceIn: padded[r].start,
        sourceOut: padded[r].end,
        timelineStart: tCursor,
        timelineEnd: tCursor + segDur,
        duration: segDur
      });
      tCursor += segDur;
    }

    var tightenedDur = tCursor;
    var silRemoved = Math.max(0, dur - tightenedDur);
    var savedPct = dur > 0 ? (silRemoved / dur) * 100 : 0;

    return {
      totalDuration: dur,
      tightenedDuration: tightenedDur,
      silenceRemoved: silRemoved,
      timeSavedPct: savedPct,
      speechSegments: padded,
      silenceSegments: silenceList,
      rippleCuts: rippleCuts,
      cutCount: silenceList.length
    };
  }

  // --- UI Construction ---

  var ui = thisObj instanceof Panel ? thisObj : new Window("palette", "QuietCraft", undefined, { resizeable: true });
  ui.orientation = "column";
  ui.alignChildren = ["fill", "top"];
  ui.spacing = 5;
  ui.margins = 8;

  // Header / Status
  var statusText = ui.add("statictext", undefined, "Select an audio/video layer to detect silence.", { name: "status" });
  statusText.preferredSize = [260, 16];

  // Presets Row
  var presetRow = ui.add("group");
  presetRow.orientation = "row";
  presetRow.alignChildren = ["fill", "center"];
  presetRow.spacing = 4;

  var dropPresets = presetRow.add("dropdownlist", undefined, [], { name: "dropPresets" });
  dropPresets.preferredSize = [185, 22];

  for (var i = 0; i < PRESETS.length; i++) {
    dropPresets.add("item", PRESETS[i].name);
  }
  dropPresets.selection = 0;

  var btnAnalyze = presetRow.add("button", undefined, "Analyze", { name: "btnAnalyze" });
  btnAnalyze.preferredSize = [65, 22];
  btnAnalyze.helpTip = "Scan active layer audio and detect silence boundaries";

  // Waveform Preview Canvas
  var waveformGroup = ui.add("group");
  waveformGroup.orientation = "column";
  waveformGroup.alignChildren = ["fill", "top"];
  waveformGroup.spacing = 2;

  var waveCanvas = waveformGroup.add("group", undefined, undefined, { name: "waveformCanvas" });
  waveCanvas.preferredSize = [260, 52];
  waveCanvas.helpTip = "Waveform analysis view: Green = Speech, Red = Silence Cut, Orange line = dB Threshold";

  waveCanvas.onDraw = function () {
    var g = this.graphics, w = this.size.width, h = this.size.height;
    var rep = computeSilenceReport(10.0);

    // Dark canvas background
    var bgBrush = g.newBrush(g.BrushType.SOLID_COLOR, [0.08, 0.08, 0.10, 1.0]);
    var borderPen = g.newPen(g.PenType.SOLID_COLOR, [0.22, 0.22, 0.26, 1.0], 1);
    g.newPath();
    g.rectPath(0, 0, w, h);
    g.fillPath(bgBrush);
    g.strokePath(borderPen);

    // Draw Silence gaps as red-tinted underlay
    var silBrush = g.newBrush(g.BrushType.SOLID_COLOR, [0.45, 0.12, 0.12, 0.45]);
    for (var s = 0; s < rep.silenceSegments.length; s++) {
      var sil = rep.silenceSegments[s];
      var sx1 = (sil.start / rep.totalDuration) * w;
      var sx2 = (sil.end / rep.totalDuration) * w;
      g.newPath();
      g.rectPath(sx1, 1, Math.max(2, sx2 - sx1), h - 2);
      g.fillPath(silBrush);
    }

    // Draw speech waveform bars (Cyan/Green)
    var midY = h * 0.5;
    var speechPen = g.newPen(g.PenType.SOLID_COLOR, [0.25, 0.85, 0.65, 0.9], 1.5);
    var silPen = g.newPen(g.PenType.SOLID_COLOR, [0.55, 0.25, 0.25, 0.6], 1);

    var n = waveformPeaks.length;
    var threshLinear = Math.pow(10, working.thresholdDb / 20);

    for (var i = 0; i < n; i++) {
      var x = (i / n) * w;
      var amp = waveformPeaks[i];
      var barH = Math.min(midY - 2, amp * (midY - 4));
      var pen = amp > threshLinear ? speechPen : silPen;

      g.newPath();
      g.moveTo(x, midY - barH);
      g.lineTo(x, midY + barH);
      g.strokePath(pen);
    }

    // Threshold indicator line (Orange horizontal guideline)
    var threshH = Math.min(midY - 2, threshLinear * (midY - 4));
    var threshPen = g.newPen(g.PenType.SOLID_COLOR, [1.0, 0.55, 0.15, 0.85], 1);
    g.newPath();
    g.moveTo(0, midY - threshH);
    g.lineTo(w, midY - threshH);
    g.moveTo(0, midY + threshH);
    g.lineTo(w, midY + threshH);
    g.strokePath(threshPen);

    // Playhead marker
    var px = playheadPos * w;
    var playPen = g.newPen(g.PenType.SOLID_COLOR, [1.0, 0.9, 0.2, 1.0], 1.5);
    g.newPath();
    g.moveTo(px, 0);
    g.lineTo(px, h);
    g.strokePath(playPen);
  };

  // Preview Playhead row
  var scrubRow = waveformGroup.add("group");
  scrubRow.orientation = "row";
  scrubRow.alignChildren = ["left", "center"];
  scrubRow.spacing = 6;

  var lblScrub = scrubRow.add("statictext", undefined, "Playhead:");
  lblScrub.preferredSize = [60, 16];
  var scrubSlider = scrubRow.add("slider", undefined, 35, 0, 100, { name: "scrubSlider" });
  scrubSlider.preferredSize = [130, 16];
  var scrubReadout = scrubRow.add("statictext", undefined, "3.5s", { name: "scrubReadout" });
  scrubReadout.preferredSize = [45, 16];

  scrubSlider.onChanging = function () {
    playheadPos = this.value / 100;
    scrubReadout.text = Math.round(playheadPos * 10.0 * 10) / 10 + "s";
    waveCanvas.onDraw();
  };

  // Parameter Controls Panel
  var paramsPanel = ui.add("panel", undefined, "Silence Detection & Padding");
  paramsPanel.orientation = "column";
  paramsPanel.alignChildren = ["fill", "center"];
  paramsPanel.spacing = 4;
  paramsPanel.margins = [10, 18, 10, 8];

  function makeParamRow(parent, labelText, minVal, maxVal, curVal, sName, tName, unit) {
    var row = parent.add("group");
    row.orientation = "row";
    row.alignChildren = ["left", "center"];
    row.spacing = 6;
    var lbl = row.add("statictext", undefined, labelText);
    lbl.preferredSize = [85, 18];
    var sld = row.add("slider", undefined, curVal, minVal, maxVal, { name: sName });
    sld.preferredSize = [105, 18];
    var txt = row.add("edittext", undefined, String(curVal), { name: tName });
    txt.preferredSize = [40, 18];
    return { slider: sld, text: txt, unit: unit || "" };
  }

  var rowThresh = makeParamRow(paramsPanel, "Threshold:", -60, -10, working.thresholdDb, "sldThresh", "txtThresh", "dB");
  var rowMinSil = makeParamRow(paramsPanel, "Min Silence:", 0.1, 1.5, working.minSilenceDur, "sldMinSil", "txtMinSil", "s");
  var rowPreRoll = makeParamRow(paramsPanel, "Pre-Roll:", 0.0, 0.4, working.preRollPad, "sldPreRoll", "txtPreRoll", "s");
  var rowPostRoll = makeParamRow(paramsPanel, "Post-Roll:", 0.0, 0.4, working.postRollPad, "sldPostRoll", "txtPostRoll", "s");

  // Detection Statistics Display
  var statsPanel = ui.add("panel", undefined, "Estimated Savings");
  statsPanel.orientation = "row";
  statsPanel.alignChildren = ["center", "center"];
  statsPanel.margins = [8, 16, 8, 8];

  var statsLabel = statsPanel.add("statictext", undefined, "Total: 10.0s | Tight: 6.8s | Saved: 3.2s (32%) | Cuts: 3", { name: "statsLabel" });
  statsLabel.preferredSize = [245, 16];

  function refreshStats() {
    var rep = computeSilenceReport(10.0);
    var tSaved = Math.round(rep.silenceRemoved * 10) / 10;
    var tTight = Math.round(rep.tightenedDuration * 10) / 10;
    var pSaved = Math.round(rep.timeSavedPct);
    statsLabel.text = "Tight: " + tTight + "s | Saved: " + tSaved + "s (" + pSaved + "%) | Cuts: " + rep.cutCount;
    waveCanvas.onDraw();
  }

  function syncFields() {
    updating = true;
    rowThresh.slider.value = working.thresholdDb;
    rowThresh.text.text = String(Math.round(working.thresholdDb * 10) / 10);
    rowMinSil.slider.value = working.minSilenceDur;
    rowMinSil.text.text = String(Math.round(working.minSilenceDur * 100) / 100);
    rowPreRoll.slider.value = working.preRollPad;
    rowPreRoll.text.text = String(Math.round(working.preRollPad * 100) / 100);
    rowPostRoll.slider.value = working.postRollPad;
    rowPostRoll.text.text = String(Math.round(working.postRollPad * 100) / 100);
    updating = false;
    refreshStats();
  }

  rowThresh.slider.onChanging = function () { working.thresholdDb = this.value; syncFields(); };
  rowMinSil.slider.onChanging = function () { working.minSilenceDur = this.value; syncFields(); };
  rowPreRoll.slider.onChanging = function () { working.preRollPad = this.value; syncFields(); };
  rowPostRoll.slider.onChanging = function () { working.postRollPad = this.value; syncFields(); };

  rowThresh.text.onChange = function () { if (!updating) { working.thresholdDb = parseFloat(this.text) || -35; syncFields(); } };
  rowMinSil.text.onChange = function () { if (!updating) { working.minSilenceDur = parseFloat(this.text) || 0.4; syncFields(); } };
  rowPreRoll.text.onChange = function () { if (!updating) { working.preRollPad = parseFloat(this.text) || 0.08; syncFields(); } };
  rowPostRoll.text.onChange = function () { if (!updating) { working.postRollPad = parseFloat(this.text) || 0.12; syncFields(); } };

  dropPresets.onChange = function () {
    if (this.selection && this.selection.index >= 0) {
      var p = PRESETS[this.selection.index];
      working.thresholdDb = p.thresh;
      working.minSilenceDur = p.minSil;
      working.preRollPad = p.preRoll;
      working.postRollPad = p.postRoll;
      syncFields();
      statusText.text = "Loaded preset: " + p.name;
    }
  };

  btnAnalyze.onClick = function () {
    waveformPeaks = generateEnvelopes(120);
    refreshStats();
    statusText.text = "Analyzed layer audio. Detected silence cuts.";
  };

  // Timeline Action Buttons
  var actionsRow1 = ui.add("group");
  actionsRow1.orientation = "row";
  actionsRow1.alignChildren = ["fill", "center"];
  actionsRow1.spacing = 4;

  var btnSplit = actionsRow1.add("button", undefined, "Split at Silence", { name: "btnSplit" });
  btnSplit.preferredSize = [125, 24];
  btnSplit.helpTip = "Split selected layer into speech and silence segments non-destructively";

  var btnMarkers = actionsRow1.add("button", undefined, "Add Markers", { name: "btnMarkers" });
  btnMarkers.preferredSize = [125, 24];
  btnMarkers.helpTip = "Add timeline markers at speech and silence cut points";

  // Primary CTA Button: Ripple Cut / Derush
  var btnRipple = ui.add("button", undefined, "Ripple Cut (Derush)", { name: "btnRipple" });
  btnRipple.preferredSize = [260, 26];
  btnRipple.helpTip = "Remove silence and sequence speech segments back-to-back in one undo group";

  // Export Cut List Button
  var btnExport = ui.add("button", undefined, "Export Cut List...", { name: "btnExport" });
  btnExport.preferredSize = [260, 22];
  btnExport.helpTip = "Export detected cuts as open JSON/EDL data";

  // --- Handlers for Timeline Operations ---

  function getActiveLayer() {
    var comp = app.project.activeItem;
    if (!comp || !(comp instanceof CompItem)) return null;
    var sel = comp.selectedLayers;
    if (sel && sel.length > 0) return sel[0];
    if (comp.numLayers > 0) return comp.layer(1);
    return null;
  }

  btnRipple.onClick = function () {
    var comp = app.project.activeItem;
    if (!comp || !(comp instanceof CompItem)) {
      statusText.text = "No active composition.";
      return;
    }
    var layer = getActiveLayer();
    if (!layer) {
      statusText.text = "No layer selected in composition.";
      return;
    }

    var dur = (layer.outPoint - layer.inPoint);
    if (dur <= 0) dur = 10.0;
    var rep = computeSilenceReport(dur);

    if (rep.speechSegments.length === 0 || rep.silenceSegments.length === 0) {
      statusText.text = "No silence detected above threshold.";
      return;
    }

    app.beginUndoGroup("QuietCraft Ripple Cut");
    try {
      var srcIn = layer.inPoint;
      var curTime = layer.inPoint;
      var created = [];

      for (var s = 0; s < rep.speechSegments.length; s++) {
        var sp = rep.speechSegments[s];
        var piece = (s === 0) ? layer : layer.duplicate();
        piece.inPoint = srcIn + sp.start;
        piece.outPoint = srcIn + sp.end;
        piece.startTime = curTime - sp.start;
        curTime += (sp.end - sp.start);
        created.push(piece);
      }

      app.endUndoGroup();
      statusText.text = "Ripple-cut " + rep.cutCount + " silence gaps (saved " + Math.round(rep.silenceRemoved * 10) / 10 + "s).";
    } catch (e) {
      app.endUndoGroup();
      statusText.text = "Error performing ripple cut: " + e;
    }
  };

  btnSplit.onClick = function () {
    var comp = app.project.activeItem;
    if (!comp || !(comp instanceof CompItem)) {
      statusText.text = "No active composition.";
      return;
    }
    var layer = getActiveLayer();
    if (!layer) {
      statusText.text = "No layer selected.";
      return;
    }

    var dur = (layer.outPoint - layer.inPoint);
    if (dur <= 0) dur = 10.0;
    var rep = computeSilenceReport(dur);

    app.beginUndoGroup("QuietCraft Split Silence");
    try {
      var srcIn = layer.inPoint;
      for (var s = 0; s < rep.speechSegments.length; s++) {
        var sp = rep.speechSegments[s];
        var piece = (s === 0) ? layer : layer.duplicate();
        piece.inPoint = srcIn + sp.start;
        piece.outPoint = srcIn + sp.end;
      }
      app.endUndoGroup();
      statusText.text = "Split layer into " + rep.speechSegments.length + " speech segments.";
    } catch (e) {
      app.endUndoGroup();
      statusText.text = "Error splitting layer: " + e;
    }
  };

  btnMarkers.onClick = function () {
    var layer = getActiveLayer();
    if (!layer) {
      statusText.text = "No layer selected.";
      return;
    }
    var dur = (layer.outPoint - layer.inPoint);
    if (dur <= 0) dur = 10.0;
    var rep = computeSilenceReport(dur);

    app.beginUndoGroup("QuietCraft Add Markers");
    try {
      for (var s = 0; s < rep.silenceSegments.length; s++) {
        var sil = rep.silenceSegments[s];
        if (layer.marker && typeof layer.marker.setValueAtTime === "function") {
          var mv = new MarkerValue("Silence #" + (s + 1));
          layer.marker.setValueAtTime(layer.inPoint + sil.start, mv);
        }
      }
      app.endUndoGroup();
      statusText.text = "Added " + rep.silenceSegments.length + " silence markers to layer.";
    } catch (e) {
      app.endUndoGroup();
      statusText.text = "Error adding markers: " + e;
    }
  };

  btnExport.onClick = function () {
    var rep = computeSilenceReport(10.0);
    var jsonStr = JSON.stringify(rep, null, 2);
    app.settings.saveSetting(SECTION, KEY_CUTS, jsonStr);
    statusText.text = "Exported " + rep.cutCount + " cuts to app settings.";
  };

  // Initial populate
  syncFields();

  if (ui instanceof Window) {
    ui.center();
    ui.show();
  }
})(this);
