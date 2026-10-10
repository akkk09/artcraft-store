// CaptionCraft: Original subtitle authoring, SRT/WebVTT processor, and caption generator for EffectCraft.
//
// A ScriptUI panel extension for EffectCraft (Window ▸ CaptionCraft.jsx).
// Built against EffectCraft's public scripting API:
// comp.layers.addText, TextDocument, layer.timing, layer.inPoint, layer.outPoint,
// app.project.activeItem, app.settings, and ScriptUI.
// Original work by EffectCraft contributors under MIT OR Apache-2.0.

(function captionCraft(thisObj) {
  var SECTION = "CaptionCraft";
  var KEY_CUES = "savedCues";
  var KEY_STYLE = "savedStyle";

  function pad(n, width) {
    var s = String(Math.floor(n));
    while (s.length < width) s = "0" + s;
    return s;
  }

  function formatTime(secs, sep) {
    secs = Math.max(0, Number(secs) || 0);
    var ms = Math.floor((secs % 1) * 1000);
    var totalS = Math.floor(secs);
    var s = totalS % 60;
    var m = Math.floor(totalS / 60) % 60;
    var h = Math.floor(totalS / 3600);
    return pad(h, 2) + ":" + pad(m, 2) + ":" + pad(s, 2) + sep + pad(ms, 3);
  }

  function parseTime(s) {
    if (typeof s === "number") return s;
    s = String(s || "").trim().replace(",", ".");
    var parts = s.split(":");
    if (parts.length === 3) {
      return (parseFloat(parts[0]) || 0) * 3600 + (parseFloat(parts[1]) || 0) * 60 + (parseFloat(parts[2]) || 0);
    } else if (parts.length === 2) {
      return (parseFloat(parts[0]) || 0) * 60 + (parseFloat(parts[1]) || 0);
    }
    return parseFloat(s) || 0;
  }

  // --- Subtitle Parsers ---

  function parseSRT(text) {
    var segs = [];
    var blocks = String(text || "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n\n");
    for (var i = 0; i < blocks.length; i++) {
      var lines = blocks[i].split("\n");
      var tIdx = -1;
      for (var l = 0; l < lines.length; l++) {
        if (lines[l].indexOf("-->") >= 0) {
          tIdx = l;
          break;
        }
      }
      if (tIdx >= 0) {
        var arrow = lines[tIdx].split("-->");
        var start = parseTime(arrow[0]);
        var end = parseTime(arrow[1]);
        var body = lines.slice(tIdx + 1).join("\n").replace(/^\s+|\s+$/g, "");
        if (body.length > 0) {
          segs.push({ id: segs.length + 1, start: start, end: end, text: body });
        }
      }
    }
    return segs;
  }

  function parseVTT(text) {
    var clean = String(text || "").replace(/^WEBVTT[^\n]*\n+/i, "");
    return parseSRT(clean);
  }

  function toSRT(segs) {
    var out = [];
    for (var i = 0; i < segs.length; i++) {
      var s = segs[i];
      out.push(String(i + 1));
      out.push(formatTime(s.start, ",") + " --> " + formatTime(s.end, ","));
      out.push(s.text);
      out.push("");
    }
    return out.join("\n");
  }

  function toVTT(segs) {
    var out = ["WEBVTT", ""];
    for (var i = 0; i < segs.length; i++) {
      var s = segs[i];
      out.push(String(i + 1));
      out.push(formatTime(s.start, ".") + " --> " + formatTime(s.end, "."));
      out.push(s.text);
      out.push("");
    }
    return out.join("\n");
  }

  // Sample default cues
  var defaultCues = [
    { id: 1, start: 1.0, end: 4.5, text: "Welcome to EffectCraft!\nProfessional motion graphics." },
    { id: 2, start: 5.0, end: 8.5, text: "CaptionCraft provides easy subtitle\nauthoring and typography styling." },
    { id: 3, start: 9.0, end: 12.0, text: "Supports SRT, WebVTT, and\nreal-time timeline rendering." }
  ];

  function loadCues() {
    if (app.settings.haveSetting(SECTION, KEY_CUES)) {
      try {
        var saved = JSON.parse(app.settings.getSetting(SECTION, KEY_CUES));
        if (saved instanceof Array && saved.length > 0) return saved;
      } catch (e) {}
    }
    return defaultCues.slice();
  }

  function saveCues(list) {
    app.settings.saveSetting(SECTION, KEY_CUES, JSON.stringify(list));
  }

  var cues = loadCues();
  var selectedIdx = 0;
  var playheadTime = 2.0;

  // --- UI Layout ---

  var ui = thisObj instanceof Panel ? thisObj : new Window("palette", "CaptionCraft", undefined, { resizeable: true });
  ui.orientation = "column";
  ui.alignChildren = ["fill", "top"];
  ui.spacing = 6;
  ui.margins = 8;

  // Header / Status
  var statusText = ui.add("statictext", undefined, "Ready. 3 caption cues loaded.", { name: "status" });
  statusText.preferredSize = [260, 16];

  // Top Action Bar
  var topBar = ui.add("group");
  topBar.orientation = "row";
  topBar.alignChildren = ["fill", "center"];
  topBar.spacing = 4;

  var btnImport = topBar.add("button", undefined, "Import...", { name: "import" });
  btnImport.helpTip = "Import subtitles from SRT or WebVTT";

  var btnExportSRT = topBar.add("button", undefined, "Export SRT", { name: "exportSrt" });
  btnExportSRT.helpTip = "Export subtitles as standard SubRip .srt";

  var btnExportVTT = topBar.add("button", undefined, "Export VTT", { name: "exportVtt" });
  btnExportVTT.helpTip = "Export subtitles as WebVTT .vtt";

  // Subtitle Cues List
  var list = ui.add("listbox", undefined, [], { name: "cueList" });
  list.preferredSize = [260, 75];
  list.helpTip = "Caption cues list. Click to edit.";

  function refreshList() {
    list.removeAll();
    for (var i = 0; i < cues.length; i++) {
      var c = cues[i];
      var dur = Math.round((c.end - c.start) * 10) / 10;
      var preview = c.text.replace(/\n/g, " ");
      if (preview.length > 25) preview = preview.substring(0, 25) + "…";
      var overlap = (i + 1 < cues.length && cues[i + 1].start < c.end) ? " ⚠️" : "";
      list.add("item", "#" + c.id + " [" + Math.round(c.start * 10) / 10 + "s - " + Math.round(c.end * 10) / 10 + "s] " + preview + overlap);
    }
    if (cues.length > 0) {
      if (selectedIdx >= cues.length) selectedIdx = cues.length - 1;
      list.selection = selectedIdx;
      syncEditorToCue();
    }
  }

  // Segment Operations Toolbar
  var opsBar = ui.add("group");
  opsBar.orientation = "row";
  opsBar.alignChildren = ["fill", "center"];
  opsBar.spacing = 3;

  var btnAdd = opsBar.add("button", undefined, "+ Cue", { name: "addCue" });
  var btnDel = opsBar.add("button", undefined, "Del", { name: "delCue" });
  var btnSplit = opsBar.add("button", undefined, "Split", { name: "splitCue" });
  var btnMerge = opsBar.add("button", undefined, "Merge", { name: "mergeCue" });
  var btnFixOverlaps = opsBar.add("button", undefined, "Fix Overlaps", { name: "fixOverlaps" });

  // Timing Editor Row
  var timePanel = ui.add("panel", undefined, "Selected Cue Timing");
  timePanel.orientation = "row";
  timePanel.alignChildren = ["left", "center"];
  timePanel.spacing = 6;
  timePanel.margins = [8, 18, 8, 8];

  var lblStart = timePanel.add("statictext", undefined, "Start:");
  lblStart.preferredSize = [34, 18];
  var txtStart = timePanel.add("edittext", undefined, "0.0", { name: "txtStart" });
  txtStart.preferredSize = [44, 18];

  var lblEnd = timePanel.add("statictext", undefined, "End:");
  lblEnd.preferredSize = [28, 18];
  var txtEnd = timePanel.add("edittext", undefined, "3.0", { name: "txtEnd" });
  txtEnd.preferredSize = [44, 18];

  var lblDur = timePanel.add("statictext", undefined, "Dur: 3.0s", { name: "lblDur" });
  lblDur.preferredSize = [65, 18];

  // Text Editor
  var textPanel = ui.add("panel", undefined, "Caption Text");
  textPanel.orientation = "column";
  textPanel.alignChildren = ["fill", "top"];
  textPanel.margins = [8, 18, 8, 8];

  var txtCaption = textPanel.add("edittext", undefined, "", { multiline: true, name: "txtCaption" });
  txtCaption.preferredSize = [250, 36];

  function syncEditorToCue() {
    if (selectedIdx >= 0 && selectedIdx < cues.length) {
      var c = cues[selectedIdx];
      txtStart.text = String(Math.round(c.start * 100) / 100);
      txtEnd.text = String(Math.round(c.end * 100) / 100);
      var dur = Math.round((c.end - c.start) * 100) / 100;
      lblDur.text = "Dur: " + dur + "s";
      txtCaption.text = c.text;
    }
  }

  function readEditorToCue() {
    if (selectedIdx >= 0 && selectedIdx < cues.length) {
      var c = cues[selectedIdx];
      c.start = Math.max(0, parseFloat(txtStart.text) || 0);
      c.end = Math.max(c.start, parseFloat(txtEnd.text) || c.start);
      c.text = txtCaption.text;
      var dur = Math.round((c.end - c.start) * 100) / 100;
      lblDur.text = "Dur: " + dur + "s";
      saveCues(cues);
      graphPreview.onDraw();
    }
  }

  txtStart.onChange = function () { readEditorToCue(); refreshList(); };
  txtEnd.onChange = function () { readEditorToCue(); refreshList(); };
  txtCaption.onChange = function () { readEditorToCue(); refreshList(); };

  list.onChange = function () {
    if (this.selection && this.selection.index >= 0) {
      selectedIdx = this.selection.index;
      syncEditorToCue();
      graphPreview.onDraw();
    }
  };

  // Preview Canvas (Simulation of Video Frame & Subtitle Layout)
  var previewGroup = ui.add("group");
  previewGroup.orientation = "column";
  previewGroup.alignChildren = ["fill", "top"];
  previewGroup.spacing = 2;

  var previewHeader = previewGroup.add("group");
  previewHeader.orientation = "row";
  previewHeader.alignChildren = ["left", "center"];
  previewHeader.spacing = 6;
  var lblPrev = previewHeader.add("statictext", undefined, "Preview Playhead:");
  lblPrev.preferredSize = [105, 18];
  var scrubSlider = previewHeader.add("slider", undefined, 20, 0, 150, { name: "scrubSlider" });
  scrubSlider.preferredSize = [95, 18];
  var timeDisplay = previewHeader.add("statictext", undefined, "2.0s", { name: "timeDisplay" });
  timeDisplay.preferredSize = [40, 18];

  var graphPreview = previewGroup.add("group", undefined, undefined, { name: "previewCanvas" });
  graphPreview.preferredSize = [260, 48];

  graphPreview.onDraw = function () {
    var g = this.graphics, w = this.size.width, h = this.size.height;
    // Frame background (Letterbox dark canvas)
    var bgBrush = g.newBrush(g.BrushType.SOLID_COLOR, [0.08, 0.08, 0.1, 1.0]);
    var borderPen = g.newPen(g.PenType.SOLID_COLOR, [0.25, 0.25, 0.28, 1.0], 1);
    g.newPath();
    g.rectPath(0, 0, w, h);
    g.fillPath(bgBrush);
    g.strokePath(borderPen);

    // Title-safe area (Dashed frame)
    var safePen = g.newPen(g.PenType.SOLID_COLOR, [0.2, 0.35, 0.45, 0.6], 1);
    g.newPath();
    g.rectPath(w * 0.08, h * 0.1, w * 0.84, h * 0.8);
    g.strokePath(safePen);

    // Find active caption at current playhead
    var activeCue = null;
    for (var i = 0; i < cues.length; i++) {
      if (playheadTime >= cues[i].start && playheadTime < cues[i].end) {
        activeCue = cues[i];
        break;
      }
    }

    if (activeCue) {
      // Subtitle bounding background pill
      var pillBrush = g.newBrush(g.BrushType.SOLID_COLOR, [0.0, 0.0, 0.0, 0.75]);
      var pillY = h * 0.58;
      g.newPath();
      g.rectPath(w * 0.1, pillY, w * 0.8, 14);
      g.fillPath(pillBrush);

      // Active Subtitle text indicator (Cyan/White marker line)
      var textPen = g.newPen(g.PenType.SOLID_COLOR, [1.0, 1.0, 0.9, 1.0], 2);
      g.newPath();
      g.moveTo(w * 0.2, pillY + 7);
      g.lineTo(w * 0.8, pillY + 7);
      g.strokePath(textPen);
    }
  };

  scrubSlider.onChanging = function () {
    playheadTime = Math.round((this.value / 10) * 10) / 10;
    timeDisplay.text = playheadTime + "s";
    graphPreview.onDraw();
  };

  // Typography & Styling Options
  var stylePanel = ui.add("panel", undefined, "Typography & Layout");
  stylePanel.orientation = "row";
  stylePanel.alignChildren = ["left", "center"];
  stylePanel.spacing = 6;
  stylePanel.margins = [8, 18, 8, 8];

  var lblSize = stylePanel.add("statictext", undefined, "Size:");
  lblSize.preferredSize = [30, 18];
  var txtSize = stylePanel.add("edittext", undefined, "48", { name: "txtSize" });
  txtSize.preferredSize = [34, 18];

  var lblStroke = stylePanel.add("statictext", undefined, "Stroke:");
  lblStroke.preferredSize = [44, 18];
  var txtStroke = stylePanel.add("edittext", undefined, "3.5", { name: "txtStroke" });
  txtStroke.preferredSize = [34, 18];

  var lblAlign = stylePanel.add("statictext", undefined, "Align:");
  lblAlign.preferredSize = [34, 18];
  var dropAlign = stylePanel.add("dropdownlist", undefined, ["Center", "Left", "Right"], { name: "dropAlign" });
  dropAlign.selection = 0;
  dropAlign.preferredSize = [72, 20];

  // Generate Timeline Layers Button
  var btnBurn = ui.add("button", undefined, "Generate Timeline Layers", { name: "btnBurn" });
  btnBurn.preferredSize = [260, 26];
  btnBurn.helpTip = "Generate timed subtitle text layers in the active composition";

  // --- Button Handlers ---

  btnAdd.onClick = function () {
    var last = cues.length > 0 ? cues[cues.length - 1] : { end: 0 };
    var newStart = Math.round((last.end + 0.5) * 10) / 10;
    var newEnd = Math.round((newStart + 3.0) * 10) / 10;
    cues.push({ id: cues.length + 1, start: newStart, end: newEnd, text: "New caption cue" });
    saveCues(cues);
    selectedIdx = cues.length - 1;
    refreshList();
    statusText.text = "Added cue #" + cues.length;
  };

  btnDel.onClick = function () {
    if (cues.length > 0 && selectedIdx >= 0 && selectedIdx < cues.length) {
      cues.splice(selectedIdx, 1);
      for (var i = 0; i < cues.length; i++) cues[i].id = i + 1;
      saveCues(cues);
      selectedIdx = Math.max(0, selectedIdx - 1);
      refreshList();
      statusText.text = "Deleted cue.";
    }
  };

  btnSplit.onClick = function () {
    if (selectedIdx >= 0 && selectedIdx < cues.length) {
      var c = cues[selectedIdx];
      var mid = Math.round(((c.start + c.end) / 2) * 10) / 10;
      var origEnd = c.end;
      c.end = mid;
      cues.splice(selectedIdx + 1, 0, { id: cues.length + 1, start: mid, end: origEnd, text: c.text });
      for (var i = 0; i < cues.length; i++) cues[i].id = i + 1;
      saveCues(cues);
      refreshList();
      statusText.text = "Split cue #" + (selectedIdx + 1);
    }
  };

  btnMerge.onClick = function () {
    if (selectedIdx >= 0 && selectedIdx + 1 < cues.length) {
      var c1 = cues[selectedIdx];
      var c2 = cues[selectedIdx + 1];
      c1.end = c2.end;
      c1.text = c1.text + "\n" + c2.text;
      cues.splice(selectedIdx + 1, 1);
      for (var i = 0; i < cues.length; i++) cues[i].id = i + 1;
      saveCues(cues);
      refreshList();
      statusText.text = "Merged cues #" + (selectedIdx + 1) + " and #" + (selectedIdx + 2);
    }
  };

  btnFixOverlaps.onClick = function () {
    var fixed = 0;
    for (var i = 0; i + 1 < cues.length; i++) {
      if (cues[i + 1].start < cues[i].end) {
        cues[i].end = Math.max(cues[i].start + 0.1, cues[i + 1].start - 0.05);
        fixed++;
      }
    }
    saveCues(cues);
    refreshList();
    statusText.text = "Resolved " + fixed + " overlap(s).";
  };

  btnImport.onClick = function () {
    var sampleSRT = prompt("Paste SRT or WebVTT content to import:", "");
    if (sampleSRT && sampleSRT.length > 5) {
      var parsed = sampleSRT.indexOf("WEBVTT") >= 0 ? parseVTT(sampleSRT) : parseSRT(sampleSRT);
      if (parsed.length > 0) {
        cues = parsed;
        saveCues(cues);
        selectedIdx = 0;
        refreshList();
        statusText.text = "Imported " + cues.length + " caption cues.";
      } else {
        statusText.text = "Could not parse subtitle format.";
      }
    }
  };

  btnExportSRT.onClick = function () {
    var srtStr = toSRT(cues);
    app.settings.saveSetting(SECTION, "lastExportedSrt", srtStr);
    statusText.text = "Exported " + cues.length + " cues as SRT.";
  };

  btnExportVTT.onClick = function () {
    var vttStr = toVTT(cues);
    app.settings.saveSetting(SECTION, "lastExportedVtt", vttStr);
    statusText.text = "Exported " + cues.length + " cues as WebVTT.";
  };

  // Generate Timeline Layers in Active Comp
  btnBurn.onClick = function () {
    var comp = app.project.activeItem;
    if (!(comp instanceof CompItem)) {
      statusText.text = "Error: Open a composition first.";
      return;
    }
    if (cues.length === 0) {
      statusText.text = "No cues to generate.";
      return;
    }

    var fontSize = parseFloat(txtSize.text) || 48;
    var strokeWidth = parseFloat(txtStroke.text) || 3.5;
    var posY = comp.height * 0.88; // Lower third (12% from bottom)
    var posX = comp.width / 2;

    app.beginUndoGroup("CaptionCraft: Generate Subtitle Layers");
    var createdCount = 0;
    try {
      for (var i = 0; i < cues.length; i++) {
        var c = cues[i];
        var boxWidth = comp.width * 0.8;
        var boxHeight = fontSize * 3.5;
        // Create box text for auto wrapping
        var layer = comp.layers.addBoxText([boxWidth, boxHeight], c.text);
        if (layer) {
          layer.name = "[Sub #" + c.id + "] " + c.text.replace(/\n/g, " ").substring(0, 20);
          layer.inPoint = c.start;
          layer.outPoint = c.end;
          // Set layer transform position
          layer.transform.position.setValue([posX, posY, 0]);

          // Set sourceText typography
          var doc = new TextDocument(c.text);
          doc.fontSize = fontSize;
          doc.fillColor = [1.0, 1.0, 1.0, 1.0];
          doc.strokeColor = [0.0, 0.0, 0.0, 1.0];
          doc.strokeWidth = strokeWidth;
          doc.applyFill = true;
          doc.applyStroke = strokeWidth > 0;
          var alignIdx = dropAlign.selection ? dropAlign.selection.index : 0;
          if (alignIdx === 1) {
            doc.justification = ParagraphJustification.LEFT_JUSTIFY;
          } else if (alignIdx === 2) {
            doc.justification = ParagraphJustification.RIGHT_JUSTIFY;
          } else {
            doc.justification = ParagraphJustification.CENTER_JUSTIFY;
          }
          if (layer.text && layer.text.sourceText) {
            layer.text.sourceText.setValue(doc);
          } else {
            var stProp = layer.property("ADBE Text Properties");
            if (stProp) stProp.property("ADBE Text Document").setValue(doc);
          }
          createdCount++;
        }
      }
    } finally {
      app.endUndoGroup();
    }
    statusText.text = "Generated " + createdCount + " caption layers on timeline.";
  };

  // Initial populate
  refreshList();

  if (ui instanceof Window) {
    ui.center();
    ui.show();
  }
})(this);
