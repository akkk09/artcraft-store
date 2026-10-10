// EaseCraft: Visual graph editor, mathematical curve engine, and preset manager for EffectCraft.
//
// A ScriptUI panel extension for EffectCraft (Window ▸ EaseCraft.jsx).
// Built against EffectCraft's public scripting API:
// comp.selectedProperties, Property.selectedKeys, keyInTemporalEase, keyOutTemporalEase,
// setTemporalEaseAtKey, KeyframeEase, setInterpolationTypeAtKey, app.settings, and ScriptUI.
// Original work by EffectCraft contributors under MIT OR Apache-2.0.

(function easeCraft(thisObj) {
  var SECTION = "EaseCraft";
  var KEY_USER_PRESETS = "userPresets";
  var KEY_CLIPBOARD = "clipboardCurve";
  var MAX_NAME = 64;
  var MIN_INFLUENCE = 0.1;
  var MAX_INFLUENCE = 100.0;
  var MAX_SPEED = 100.0;
  var BEZIER = KeyframeInterpolationType.BEZIER;
  var HOLD = KeyframeInterpolationType.HOLD;

  function curve(outInf, outSpd, inInf, inSpd) {
    return {
      outInfluence: outInf,
      outSpeed: outSpd,
      inInfluence: inInf,
      inSpeed: inSpd
    };
  }

  // Built-in standard and mathematical presets
  var BUILT_IN = [
    { name: "Linear", category: "Basic", curve: curve(100 / 3, 1, 100 / 3, 1) },
    { name: "Ease In", category: "Basic", curve: curve(33.333, 0, 33.333, 1) },
    { name: "Ease Out", category: "Basic", curve: curve(33.333, 1, 33.333, 0) },
    { name: "Ease In-Out Soft", category: "Standard", curve: curve(25, 0, 25, 0) },
    { name: "Ease In-Out", category: "Standard", curve: curve(50, 0, 50, 0) },
    { name: "Ease In-Out Strong", category: "Standard", curve: curve(75, 0, 75, 0) },
    { name: "Ease In-Out Extreme", category: "Standard", curve: curve(88, 0, 88, 0) },
    { name: "Back In (Anticipate)", category: "Dynamic", curve: curve(45, -0.85, 33.333, 1.2) },
    { name: "Back Out (Overshoot)", category: "Dynamic", curve: curve(33.333, 1.2, 45, -0.85) },
    { name: "Back In-Out", category: "Dynamic", curve: curve(50, -0.75, 50, -0.75) },
    { name: "Elastic Snap", category: "Dynamic", curve: curve(70, 0, 20, -1.5) },
    { name: "Bounce Settle", category: "Dynamic", curve: curve(35, 1.8, 65, -0.4) },
    { name: "Smooth S-Curve", category: "Standard", curve: curve(40, 0.25, 40, 0.25) }
  ];

  function finite(v) {
    return typeof v === "number" && isFinite(v);
  }

  function sanitize(c) {
    if (!c || !finite(c.outInfluence) || !finite(c.outSpeed) || !finite(c.inInfluence) || !finite(c.inSpeed)) {
      return null;
    }
    var inf = function (v) { return Math.min(MAX_INFLUENCE, Math.max(MIN_INFLUENCE, v)); };
    var spd = function (v) { return Math.min(MAX_SPEED, Math.max(-MAX_SPEED, v)); };
    return curve(inf(c.outInfluence), spd(c.outSpeed), inf(c.inInfluence), spd(c.inSpeed));
  }

  function cleanName(n) {
    var s = String(n === undefined || n === null ? "" : n).replace(/^\s+|\s+$/g, "");
    return s.length > 0 && s.length <= MAX_NAME ? s : null;
  }

  // --- Mathematical Curve Solver ---

  function bez1d(p0, p1, p2, p3, u) {
    var v = 1 - u;
    return v * v * v * p0 + 3 * v * v * u * p1 + 3 * v * u * u * p2 + u * u * u * p3;
  }

  function solveU(c, x) {
    var p0 = 0, p1 = c.outInfluence / 100, p2 = 1 - c.inInfluence / 100, p3 = 1;
    var u = Math.min(1, Math.max(0, x));
    for (var i = 0; i < 12; i++) {
      var f = bez1d(p0, p1, p2, p3, u) - x;
      if (Math.abs(f) < 1e-6) return u;
      var v = 1 - u;
      var d = 3 * v * v * (p1 - p0) + 6 * v * u * (p2 - p1) + 3 * u * u * (p3 - p2);
      if (Math.abs(d) < 1e-6) break;
      var next = u - f / d;
      if (next < 0 || next > 1) break;
      u = next;
    }
    var lo = 0, hi = 1;
    for (var j = 0; j < 40; j++) {
      var mid = (lo + hi) * 0.5;
      if (bez1d(p0, p1, p2, p3, mid) < x) lo = mid; else hi = mid;
    }
    return (lo + hi) * 0.5;
  }

  function evalCurve(c, x) {
    if (x <= 0) return 0;
    if (x >= 1) return 1;
    var u = solveU(c, x);
    var y0 = 0;
    var y1 = (c.outInfluence / 100) * c.outSpeed;
    var y2 = 1 - (c.inInfluence / 100) * c.inSpeed;
    var y3 = 1;
    return bez1d(y0, y1, y2, y3, u);
  }

  // --- Preset Persistence & Open Format ---

  function loadUserPresets() {
    var out = [];
    if (!app.settings.haveSetting(SECTION, KEY_USER_PRESETS)) return out;
    try {
      var doc = JSON.parse(app.settings.getSetting(SECTION, KEY_USER_PRESETS));
      var list = doc && doc.presets instanceof Array ? doc.presets : [];
      for (var i = 0; i < list.length; i++) {
        var name = cleanName(list[i] && list[i].name);
        var c = sanitize(list[i] && list[i].curve);
        if (name && c) out.push({ name: name, category: "Custom", curve: c });
      }
    } catch (e) {}
    return out;
  }

  function saveUserPresets(list) {
    var doc = {
      format: "easecraft-presets",
      version: 1,
      presets: list
    };
    app.settings.saveSetting(SECTION, KEY_USER_PRESETS, JSON.stringify(doc));
  }

  // --- Keyframe Detection & Easing Application ---

  function selectedPairs() {
    var comp = app.project.activeItem;
    var out = [];
    if (!(comp instanceof CompItem)) return out;
    var props = comp.selectedProperties;
    for (var i = 0; i < props.length; i++) {
      var p = props[i];
      if (p.propertyType !== PropertyType.PROPERTY || p.numKeys < 2 || !p.isInterpolationTypeValid(BEZIER)) continue;
      var keys = p.selectedKeys;
      for (var k = 0; k + 1 < keys.length; k++) {
        if (keys[k + 1] === keys[k] + 1) out.push({ prop: p, key: keys[k] });
      }
    }
    return out;
  }

  function pathLength(a, outTan, inTan, b) {
    var n = Math.max(a.length, b.length), p = [], len = 0, prev = null;
    for (var d = 0; d < n; d++) {
      p.push([a[d] || 0, (a[d] || 0) + (outTan[d] || 0), (b[d] || 0) + (inTan[d] || 0), b[d] || 0]);
    }
    var steps = 64;
    for (var s = 0; s <= steps; s++) {
      var t = s / steps, u = 1 - t, pt = [];
      for (var e = 0; e < n; e++) {
        pt.push(u * u * u * p[e][0] + 3 * u * u * t * p[e][1] + 3 * u * t * t * p[e][2] + t * t * t * p[e][3]);
      }
      if (prev) {
        var sq = 0;
        for (var f = 0; f < n; f++) sq += (pt[f] - prev[f]) * (pt[f] - prev[f]);
        len += Math.sqrt(sq);
      }
      prev = pt;
    }
    return len;
  }

  function layerOf(p) {
    return p.propertyGroup(p.propertyDepth);
  }

  function slopes(p, a) {
    var stretch = layerOf(p).stretch || 100;
    var dur = (p.keyTime(a + 1) - p.keyTime(a)) * 100 / stretch;
    if (!(dur > 0) || !isFinite(dur)) return null;
    var n = Math.max(1, p.keyOutTemporalEase(a).length);
    var v0 = p.keyValue(a), v1 = p.keyValue(a + 1), out = [];
    var vt = p.propertyValueType;
    if (vt === PropertyValueType.TwoD_SPATIAL || vt === PropertyValueType.ThreeD_SPATIAL) {
      var spd = pathLength(v0, p.keyOutSpatialTangent(a), p.keyInSpatialTangent(a + 1), v1) / dur;
      for (var i = 0; i < n; i++) out.push(spd);
    } else if (typeof v0 === "number" && typeof v1 === "number") {
      out.push((v1 - v0) / dur);
    } else if (v0 instanceof Array && v1 instanceof Array && v0.length === v1.length) {
      for (var d = 0; d < n; d++) out.push(((v1[d] || 0) - (v0[d] || 0)) / dur);
    } else {
      for (var j = 0; j < n; j++) out.push(1 / dur);
    }
    return out;
  }

  function easeSide(p, k, out, eases) {
    var inType = p.keyInInterpolationType(k), outType = p.keyOutInterpolationType(k);
    var inEase = p.keyInTemporalEase(k), outEase = p.keyOutTemporalEase(k);
    if (p.keyTemporalContinuous(k)) p.setTemporalContinuousAtKey(k, false);
    if (out) {
      p.setTemporalEaseAtKey(k, inEase, eases);
      p.setInterpolationTypeAtKey(k, inType, BEZIER);
    } else {
      p.setTemporalEaseAtKey(k, eases, outEase);
      p.setInterpolationTypeAtKey(k, BEZIER, outType);
    }
  }

  function applyCurve(c) {
    var pairs = selectedPairs(), n = 0;
    if (!pairs.length) return 0;
    app.beginUndoGroup("EaseCraft: Apply Curve");
    try {
      for (var i = 0; i < pairs.length; i++) {
        var p = pairs[i].prop, a = pairs[i].key, s = slopes(p, a);
        if (!s) continue;
        var outE = [], inE = [];
        for (var d = 0; d < s.length; d++) {
          outE.push(new KeyframeEase(c.outSpeed * s[d], c.outInfluence));
          inE.push(new KeyframeEase(c.inSpeed * s[d], c.inInfluence));
        }
        easeSide(p, a, true, outE);
        easeSide(p, a + 1, false, inE);
        n++;
      }
    } finally {
      app.endUndoGroup();
    }
    return n;
  }

  function captureCurve() {
    var pairs = selectedPairs();
    if (!pairs.length) return null;
    var p = pairs[0].prop, a = pairs[0].key;
    if (p.keyOutInterpolationType(a) === HOLD) return null;
    var s = slopes(p, a);
    if (!s) return null;
    var d = 0;
    for (var i = 1; i < s.length; i++) if (Math.abs(s[i]) > Math.abs(s[d])) d = i;
    if (Math.abs(s[d]) < 1e-12) return null;
    var o = p.keyOutTemporalEase(a), n = p.keyInTemporalEase(a + 1);
    var oe = o[d] || o[0], ne = n[d] || n[0];
    if (!oe || !ne) return null;
    return sanitize(curve(oe.influence, oe.speed / s[d], ne.influence, ne.speed / s[d]));
  }

  // --- UI Layout & Controls ---

  var ui = thisObj instanceof Panel ? thisObj : new Window("palette", "EaseCraft", undefined, { resizeable: true });
  ui.orientation = "column";
  ui.alignChildren = ["fill", "top"];
  ui.spacing = 6;
  ui.margins = 8;

  var userPresets = loadUserPresets();
  var working = curve(50, 0, 50, 0); // Default: Ease In-Out
  var previewPos = 0.5;
  var updating = false;

  // Header / Status
  var statusText = ui.add("statictext", undefined, "Select keyframes and apply easing.", { name: "status" });
  statusText.preferredSize = [240, 16];

  // Presets Row
  var presetRow = ui.add("group");
  presetRow.orientation = "row";
  presetRow.alignChildren = ["fill", "center"];
  presetRow.spacing = 4;

  var presetList = presetRow.add("dropdownlist", undefined, [], { name: "presets" });
  presetList.preferredSize = [170, 24];

  var btnApply = presetRow.add("button", undefined, "Apply", { name: "apply" });
  btnApply.preferredSize = [65, 24];
  btnApply.helpTip = "Apply current curve to selected keyframes (one undo step)";

  function allPresets() {
    return BUILT_IN.concat(userPresets);
  }

  function refreshPresetDropdown(selectedName) {
    presetList.removeAll();
    var list = allPresets();
    var selIdx = 0;
    for (var i = 0; i < list.length; i++) {
      var item = presetList.add("item", list[i].name);
      if (selectedName && list[i].name.toLowerCase() === selectedName.toLowerCase()) {
        selIdx = i;
      }
    }
    if (list.length > 0) presetList.selection = selIdx;
  }

  // Graph Area
  var graph = ui.add("group", undefined, undefined, { name: "graph" });
  graph.preferredSize = [240, 120];
  graph.helpTip = "Bézier curve view (0..1 time, -0.35..1.35 value)";

  graph.onDraw = function () {
    var g = this.graphics, w = this.size.width, h = this.size.height;
    var lo = -0.35, hi = 1.35, m = 12;
    var at = function (x, v) {
      return [m + (w - 2 * m) * x, h - m - (h - 2 * m) * (v - lo) / (hi - lo)];
    };

    // Border
    var borderPen = g.newPen(g.PenType.SOLID_COLOR, [0.3, 0.3, 0.3, 0.8], 1);
    g.newPath();
    g.rectPath(0, 0, w, h);
    g.strokePath(borderPen);

    // Reference Grid (0.0, 0.5, 1.0)
    var gridPen = g.newPen(g.PenType.SOLID_COLOR, [0.22, 0.22, 0.22, 0.9], 1);
    var p0_0 = at(0, 0), p1_0 = at(1, 0);
    var p0_1 = at(0, 1), p1_1 = at(1, 1);
    g.newPath();
    g.moveTo(p0_0[0], p0_0[1]); g.lineTo(p1_0[0], p1_0[1]);
    g.moveTo(p0_1[0], p0_1[1]); g.lineTo(p1_1[0], p1_1[1]);
    g.strokePath(gridPen);

    // Linear diagonal reference
    var diagPen = g.newPen(g.PenType.SOLID_COLOR, [0.28, 0.28, 0.28, 0.6], 1);
    g.newPath();
    g.moveTo(p0_0[0], p0_0[1]); g.lineTo(p1_1[0], p1_1[1]);
    g.strokePath(diagPen);

    // Tangent Handles
    var x1 = working.outInfluence / 100, y1 = (working.outInfluence / 100) * working.outSpeed;
    var x2 = 1 - working.inInfluence / 100, y2 = 1 - (working.inInfluence / 100) * working.inSpeed;
    var h1 = at(x1, y1), h2 = at(x2, y2);

    var handlePen = g.newPen(g.PenType.SOLID_COLOR, [0.9, 0.5, 0.1, 0.7], 1.5);
    g.newPath();
    g.moveTo(p0_0[0], p0_0[1]); g.lineTo(h1[0], h1[1]);
    g.moveTo(p1_1[0], p1_1[1]); g.lineTo(h2[0], h2[1]);
    g.strokePath(handlePen);

    // Tangent Pins
    var pinBrush = g.newBrush(g.BrushType.SOLID_COLOR, [1.0, 0.6, 0.2, 0.9]);
    g.newPath();
    g.rectPath(h1[0] - 2.5, h1[1] - 2.5, 5, 5);
    g.rectPath(h2[0] - 2.5, h2[1] - 2.5, 5, 5);
    g.fillPath(pinBrush);

    // Main Bézier Curve (Cyan)
    var curvePen = g.newPen(g.PenType.SOLID_COLOR, [0.2, 0.8, 1.0, 1.0], 2);
    g.newPath();
    var startPt = at(0, 0);
    g.moveTo(startPt[0], startPt[1]);
    var steps = 48;
    for (var s = 1; s <= steps; s++) {
      var t = s / steps;
      var val = evalCurve(working, t);
      var pt = at(t, val);
      g.lineTo(pt[0], pt[1]);
    }
    g.strokePath(curvePen);

    // Animated Preview Marker (Yellow Dot)
    var prevVal = evalCurve(working, previewPos);
    var prevPt = at(previewPos, prevVal);
    var prevBrush = g.newBrush(g.BrushType.SOLID_COLOR, [1.0, 0.9, 0.2, 1.0]);
    g.newPath();
    g.rectPath(prevPt[0] - 3.5, prevPt[1] - 3.5, 7, 7);
    g.fillPath(prevBrush);
  };

  // Preview Motion Track & Slider
  var previewGroup = ui.add("group");
  previewGroup.orientation = "row";
  previewGroup.alignChildren = ["fill", "center"];
  previewGroup.spacing = 4;

  var prevLabel = previewGroup.add("statictext", undefined, "Preview:");
  prevLabel.preferredSize = [48, 16];

  var previewSlider = previewGroup.add("slider", undefined, 50, 0, 100, { name: "previewSlider" });
  previewSlider.preferredSize = [140, 16];
  previewSlider.helpTip = "Scrub preview marker along the curve";

  var prevReadout = previewGroup.add("statictext", undefined, "50%", { name: "previewReadout" });
  prevReadout.preferredSize = [42, 16];

  // Sliders for Out / In Influences and Speeds
  var paramsPanel = ui.add("panel", undefined, "Curve Parameters");
  paramsPanel.orientation = "column";
  paramsPanel.alignChildren = ["fill", "center"];
  paramsPanel.spacing = 4;
  paramsPanel.margins = [10, 18, 10, 10];

  function makeRow(parent, labelText, minVal, maxVal, curVal, sName, tName) {
    var row = parent.add("group");
    row.orientation = "row";
    row.alignChildren = ["fill", "center"];
    row.spacing = 4;
    var lbl = row.add("statictext", undefined, labelText);
    lbl.preferredSize = [75, 18];
    var sld = row.add("slider", undefined, curVal, minVal, maxVal, { name: sName });
    sld.preferredSize = [105, 18];
    var txt = row.add("edittext", undefined, String(curVal), { name: tName });
    txt.preferredSize = [42, 18];
    return { slider: sld, text: txt };
  }

  var rowOutInf = makeRow(paramsPanel, "Out Inf %:", 0.1, 100, 50, "outInfSlider", "outInfText");
  var rowOutSpd = makeRow(paramsPanel, "Out Speed:", -5, 5, 0, "outSpdSlider", "outSpdText");
  var rowInInf = makeRow(paramsPanel, "In Inf %:", 0.1, 100, 50, "inInfSlider", "inInfText");
  var rowInSpd = makeRow(paramsPanel, "In Speed:", -5, 5, 0, "inSpdSlider", "inSpdText");

  function syncWorkingToFields() {
    updating = true;
    rowOutInf.slider.value = working.outInfluence;
    rowOutInf.text.text = String(Math.round(working.outInfluence * 10) / 10);
    rowOutSpd.slider.value = working.outSpeed;
    rowOutSpd.text.text = String(Math.round(working.outSpeed * 100) / 100);
    rowInInf.slider.value = working.inInfluence;
    rowInInf.text.text = String(Math.round(working.inInfluence * 10) / 10);
    rowInSpd.slider.value = working.inSpeed;
    rowInSpd.text.text = String(Math.round(working.inSpeed * 100) / 100);
    updating = false;
    graph.onDraw();
  }

  function readFieldsToWorking() {
    if (updating) return;
    var oInf = parseFloat(rowOutInf.text.text);
    var oSpd = parseFloat(rowOutSpd.text.text);
    var iInf = parseFloat(rowInInf.text.text);
    var iSpd = parseFloat(rowInSpd.text.text);
    var c = sanitize(curve(oInf, oSpd, iInf, iSpd));
    if (c) {
      working = c;
      syncWorkingToFields();
    }
  }

  // Hook slider and text listeners
  rowOutInf.slider.onChanging = function () { working.outInfluence = this.value; syncWorkingToFields(); };
  rowOutSpd.slider.onChanging = function () { working.outSpeed = this.value; syncWorkingToFields(); };
  rowInInf.slider.onChanging = function () { working.inInfluence = this.value; syncWorkingToFields(); };
  rowInSpd.slider.onChanging = function () { working.inSpeed = this.value; syncWorkingToFields(); };

  rowOutInf.text.onChange = readFieldsToWorking;
  rowOutSpd.text.onChange = readFieldsToWorking;
  rowInInf.text.onChange = readFieldsToWorking;
  rowInSpd.text.onChange = readFieldsToWorking;

  previewSlider.onChanging = function () {
    previewPos = this.value / 100;
    var y = evalCurve(working, previewPos);
    prevReadout.text = Math.round(previewPos * 100) + "%";
    statusText.text = "t: " + Math.round(previewPos * 100) + "%  →  val: " + Math.round(y * 100) + "%";
    graph.onDraw();
  };

  // Button Toolbars: Actions
  var actionsGroup = ui.add("group");
  actionsGroup.orientation = "row";
  actionsGroup.alignChildren = ["fill", "center"];
  actionsGroup.spacing = 4;

  var btnMirror = actionsGroup.add("button", undefined, "Mirror", { name: "mirror" });
  btnMirror.helpTip = "Swap Out and In handles";
  btnMirror.onClick = function () {
    working = curve(working.inInfluence, working.inSpeed, working.outInfluence, working.outSpeed);
    syncWorkingToFields();
    statusText.text = "Mirrored curve.";
  };

  var btnReverse = actionsGroup.add("button", undefined, "Reverse", { name: "reverse" });
  btnReverse.helpTip = "Reverse curve velocity direction";
  btnReverse.onClick = function () {
    working = curve(working.inInfluence, 2 - working.inSpeed, working.outInfluence, 2 - working.outSpeed);
    syncWorkingToFields();
    statusText.text = "Reversed curve.";
  };

  var btnCapture = actionsGroup.add("button", undefined, "From Keys", { name: "capture" });
  btnCapture.helpTip = "Capture curve from first selected keyframe pair";
  btnCapture.onClick = function () {
    var c = captureCurve();
    if (c) {
      working = c;
      syncWorkingToFields();
      statusText.text = "Captured curve from selected keys.";
    } else {
      statusText.text = "No keyframe pair selected or non-eased.";
    }
  };

  // Second row: Copy, Paste, Save, Export
  var utilsGroup = ui.add("group");
  utilsGroup.orientation = "row";
  utilsGroup.alignChildren = ["fill", "center"];
  utilsGroup.spacing = 4;

  var btnCopy = utilsGroup.add("button", undefined, "Copy", { name: "copy" });
  btnCopy.helpTip = "Copy current curve parameters";
  btnCopy.onClick = function () {
    app.settings.saveSetting(SECTION, KEY_CLIPBOARD, JSON.stringify(working));
    statusText.text = "Copied curve to clipboard.";
  };

  var btnPaste = utilsGroup.add("button", undefined, "Paste", { name: "paste" });
  btnPaste.helpTip = "Paste curve parameters from clipboard";
  btnPaste.onClick = function () {
    if (app.settings.haveSetting(SECTION, KEY_CLIPBOARD)) {
      try {
        var c = sanitize(JSON.parse(app.settings.getSetting(SECTION, KEY_CLIPBOARD)));
        if (c) {
          working = c;
          syncWorkingToFields();
          statusText.text = "Pasted curve from clipboard.";
        }
      } catch (e) {
        statusText.text = "Invalid clipboard data.";
      }
    } else {
      statusText.text = "Clipboard is empty.";
    }
  };

  var btnSavePreset = utilsGroup.add("button", undefined, "Save...", { name: "savePreset" });
  btnSavePreset.helpTip = "Save current curve as a user preset";
  btnSavePreset.onClick = function () {
    var name = cleanName(prompt("Preset Name:", "My Ease"));
    if (!name) return;
    for (var i = 0; i < userPresets.length; i++) {
      if (userPresets[i].name.toLowerCase() === name.toLowerCase()) {
        userPresets[i].curve = working;
        saveUserPresets(userPresets);
        refreshPresetDropdown(name);
        statusText.text = "Updated preset: " + name;
        return;
      }
    }
    userPresets.push({ name: name, category: "Custom", curve: working });
    saveUserPresets(userPresets);
    refreshPresetDropdown(name);
    statusText.text = "Saved preset: " + name;
  };

  var btnExport = utilsGroup.add("button", undefined, "Export...", { name: "export" });
  btnExport.helpTip = "Export presets to open JSON format";
  btnExport.onClick = function () {
    var pack = {
      format: "easecraft-presets",
      version: 1,
      presets: allPresets()
    };
    var jsonStr = JSON.stringify(pack, null, 2);
    app.settings.saveSetting(SECTION, "lastExported", jsonStr);
    statusText.text = "Exported " + pack.presets.len + " presets to app settings.";
  };

  // Dropdown selection change
  presetList.onChange = function () {
    if (this.selection && this.selection.index >= 0) {
      var list = allPresets();
      var p = list[this.selection.index];
      if (p && p.curve) {
        working = p.curve;
        syncWorkingToFields();
        statusText.text = "Loaded preset: " + p.name;
      }
    }
  };

  // Apply button
  btnApply.onClick = function () {
    var count = applyCurve(working);
    if (count > 0) {
      statusText.text = "Eased " + count + " keyframe pair" + (count > 1 ? "s" : "") + ".";
    } else {
      statusText.text = "No keyframe pairs selected.";
    }
  };

  // Initial populate
  refreshPresetDropdown("Ease In-Out");
  syncWorkingToFields();

  if (ui instanceof Window) {
    ui.center();
    ui.show();
  }
})(this);
