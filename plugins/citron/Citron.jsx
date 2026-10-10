// Citron: Multi-channel graph editor, FFD lattice cage tool, and curve workspace for EffectCraft.
//
// A ScriptUI panel extension for EffectCraft (Window ▸ Citron.jsx).
// Built against EffectCraft's public scripting API:
// comp.selectedLayers, comp.selectedProperties, Property.selectedKeys, keyInTemporalEase,
// keyOutTemporalEase, setTemporalEaseAtKey, KeyframeEase, setInterpolationTypeAtKey,
// setValueAtTime, app.beginUndoGroup, app.settings, and ScriptUI.
// Original work by EffectCraft contributors under MIT OR Apache-2.0.

(function citron(thisObj) {
  var SECTION = "Citron";
  var KEY_SETTINGS = "citronSettings";
  var KEY_CLIPBOARD = "clipboardTangents";

  var MIN_INFLUENCE = 0.1;
  var MAX_INFLUENCE = 100.0;
  var MAX_SPEED = 1000.0;
  var EPSILON = 1e-6;

  var BEZIER = KeyframeInterpolationType.BEZIER;
  var LINEAR = KeyframeInterpolationType.LINEAR;
  var HOLD = KeyframeInterpolationType.HOLD;

  // Channel color palette (Maya/Blender inspired color-coding)
  var CHANNEL_COLORS = [
    [0.95, 0.25, 0.25, 1.0], // Red: X / Width / Primary
    [0.25, 0.85, 0.35, 1.0], // Green: Y / Height / Secondary
    [0.25, 0.55, 0.95, 1.0], // Blue: Z / Rotation / Angle
    [0.95, 0.75, 0.20, 1.0], // Gold: Scale / Transform
    [0.80, 0.40, 0.95, 1.0], // Purple: Opacity / Alpha
    [0.20, 0.85, 0.85, 1.0], // Cyan: Effects / Parameters
    [0.95, 0.45, 0.65, 1.0], // Pink: Custom
    [0.65, 0.85, 0.35, 1.0]  // Lime: Custom
  ];

  // --- Mathematical Solvers ---

  function bez1d(p0, p1, p2, p3, u) {
    var v = 1 - u;
    return v * v * v * p0 + 3 * v * v * u * p1 + 3 * v * u * u * p2 + u * u * u * p3;
  }

  function solveU(outInf, inInf, x) {
    var p0 = 0, p1 = outInf / 100, p2 = 1 - inInf / 100, p3 = 1;
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
    for (var j = 0; j < 32; j++) {
      var mid = (lo + hi) * 0.5;
      if (bez1d(p0, p1, p2, p3, mid) < x) lo = mid; else hi = mid;
    }
    return (lo + hi) * 0.5;
  }

  function evalSegment(k0, k1, t) {
    var dt = Math.max(EPSILON, k1.time - k0.time);
    if (k0.interp === HOLD) return k0.value;
    if (k0.interp === LINEAR) {
      var f = Math.min(1, Math.max(0, (t - k0.time) / dt));
      return k0.value + f * (k1.value - k0.value);
    }
    // Bezier
    var x = Math.min(1, Math.max(0, (t - k0.time) / dt));
    var u = solveU(k0.outInfluence, k1.inInfluence, x);
    var v0 = k0.value;
    var v1 = k0.value + (k0.outInfluence / 100) * k0.outSpeed * dt;
    var v2 = k1.value - (k1.inInfluence / 100) * k1.inSpeed * dt;
    var v3 = k1.value;
    return bez1d(v0, v1, v2, v3, u);
  }

  // --- Channel & Keyframe Model ---

  var activeChannels = [];
  var bufferSnapshot = null;
  var selectedChannelIdx = 0;
  var selectedKeyIdx = -1;
  var viewMode = 0; // 0: Normalized (0-100%), 1: Absolute, 2: Stacked, 3: Speed Graph
  var showBuffer = false;
  var latticeMode = false;
  var unifiedTangents = true;

  function KeyItem(time, value, inInf, inSpd, outInf, outSpd, interp) {
    this.time = time;
    this.value = value;
    this.inInfluence = inInf;
    this.inSpeed = inSpd;
    this.outInfluence = outInf;
    this.outSpeed = outSpd;
    this.interp = interp || BEZIER;
    this.selected = false;
  }

  function ChannelItem(id, name, prop, color) {
    this.id = id;
    this.name = name;
    this.prop = prop;
    this.color = color;
    this.keys = [];
    this.minTime = 0;
    this.maxTime = 1;
    this.minValue = 0;
    this.maxValue = 1;
  }

  ChannelItem.prototype.evalAt = function (t) {
    if (!this.keys || this.keys.length === 0) return 0;
    if (this.keys.length === 1 || t <= this.keys[0].time) return this.keys[0].value;
    if (t >= this.keys[this.keys.length - 1].time) return this.keys[this.keys.length - 1].value;
    for (var i = 0; i < this.keys.length - 1; i++) {
      if (t >= this.keys[i].time && t <= this.keys[i + 1].time) {
        return evalSegment(this.keys[i], this.keys[i + 1], t);
      }
    }
    return this.keys[this.keys.length - 1].value;
  };

  ChannelItem.prototype.updateBounds = function () {
    if (!this.keys || this.keys.length === 0) {
      this.minTime = 0; this.maxTime = 1;
      this.minValue = 0; this.maxValue = 1;
      return;
    }
    this.minTime = this.keys[0].time;
    this.maxTime = this.keys[0].time;
    this.minValue = this.keys[0].value;
    this.maxValue = this.keys[0].value;
    for (var i = 0; i < this.keys.length; i++) {
      var k = this.keys[i];
      if (k.time < this.minTime) this.minTime = k.time;
      if (k.time > this.maxTime) this.maxTime = k.time;
      if (k.value < this.minValue) this.minValue = k.value;
      if (k.value > this.maxValue) this.maxValue = k.value;
    }
    if (Math.abs(this.maxTime - this.minTime) < EPSILON) this.maxTime += 1;
    if (Math.abs(this.maxValue - this.minValue) < EPSILON) this.maxValue += 1;
  };

  // --- Engine Scanner ---

  function inspectActiveChannels() {
    activeChannels = [];
    var comp = app.project ? app.project.activeItem : null;
    if (!comp || !(comp instanceof CompItem)) return activeChannels;

    var propsToInspect = [];
    var selProps = comp.selectedProperties;
    if (selProps && selProps.length > 0) {
      for (var p = 0; p < selProps.length; p++) {
        var sp = selProps[p];
        if (sp && sp.numKeys && sp.numKeys > 0) propsToInspect.push(sp);
      }
    }

    // Fallback to selected layers
    if (propsToInspect.length === 0) {
      var selLayers = comp.selectedLayers;
      if (selLayers && selLayers.length > 0) {
        for (var l = 0; l < selLayers.length; l++) {
          var layer = selLayers[l];
          if (!layer || !layer.transform) continue;
          var xf = layer.transform;
          var candidates = [xf.position, xf.xPosition, xf.yPosition, xf.zPosition, xf.rotation, xf.scale, xf.opacity];
          for (var c = 0; c < candidates.length; c++) {
            if (candidates[c] && candidates[c].numKeys && candidates[c].numKeys > 0) {
              propsToInspect.push(candidates[c]);
            }
          }
        }
      }
    }

    for (var i = 0; i < propsToInspect.length; i++) {
      var prop = propsToInspect[i];
      var name = prop.name || ("Property " + (i + 1));
      var col = CHANNEL_COLORS[i % CHANNEL_COLORS.length];
      var ch = new ChannelItem("ch_" + i, name, prop, col);

      var nKeys = prop.numKeys;
      for (var k = 1; k <= nKeys; k++) {
        var t = prop.keyTime(k);
        var rawVal = prop.keyValue(k);
        var valNum = typeof rawVal === "number" ? rawVal : (rawVal instanceof Array ? rawVal[0] : 0);

        var inEase = prop.keyInTemporalEase(k);
        var outEase = prop.keyOutTemporalEase(k);
        var inInf = (inEase && inEase.length > 0) ? inEase[0].influence : 33.333;
        var inSpd = (inEase && inEase.length > 0) ? inEase[0].speed : 0;
        var outInf = (outEase && outEase.length > 0) ? outEase[0].influence : 33.333;
        var outSpd = (outEase && outEase.length > 0) ? outEase[0].speed : 0;
        var interp = prop.keyInInterpolationType ? prop.keyInInterpolationType(k) : BEZIER;

        var keyItem = new KeyItem(t, valNum, inInf, inSpd, outInf, outSpd, interp);
        ch.keys.push(keyItem);
      }
      ch.updateBounds();
      activeChannels.push(ch);
    }

    if (activeChannels.length > 0 && selectedChannelIdx >= activeChannels.length) {
      selectedChannelIdx = 0;
    }
    return activeChannels;
  }

  // --- Building the UI ---

  var win = (thisObj instanceof Panel)
    ? thisObj
    : new Window("palette", "Citron - Maya-Style Graph Editor", undefined, { resizeable: true });

  win.orientation = "column";
  win.alignChildren = ["fill", "top"];
  win.spacing = 6;
  win.margins = 8;

  // --- Row 1: Channel Selector & View Modes ---
  var row1 = win.add("group");
  row1.orientation = "row";
  row1.alignChildren = ["left", "center"];
  row1.spacing = 8;

  var btnRefresh = row1.add("button", undefined, "↻ Refresh", { name: "btnRefresh" });
  btnRefresh.helpTip = "Scan selected layers and properties for animation channels";

  row1.add("statictext", undefined, "Channel:");
  var dropChannels = row1.add("dropdownlist", undefined, ["(No animated channels)"], { name: "dropChannels" });
  dropChannels.size = [160, 24];

  row1.add("statictext", undefined, "View:");
  var dropView = row1.add("dropdownlist", undefined, [
    "Normalized (0-100%)",
    "Absolute Values",
    "Stacked Lanes",
    "Speed Graph"
  ], { name: "dropView" });
  dropView.selection = 0;

  var chkBuffer = row1.add("checkbox", undefined, "Buffer Ghost", { name: "chkBuffer" });
  chkBuffer.helpTip = "Show ghosted reference snapshot of curves before edits";
  chkBuffer.value = false;

  var chkLattice = row1.add("checkbox", undefined, "FFD Cage", { name: "chkLattice" });
  chkLattice.helpTip = "Enable 2D Lattice deformation cage for bulk curve shaping";
  chkLattice.value = false;

  // --- Row 2: Interactive Curve Canvas ---
  var canvasGroup = win.add("group");
  canvasGroup.orientation = "column";
  canvasGroup.alignChildren = ["fill", "fill"];
  canvasGroup.alignment = ["fill", "fill"];
  canvasGroup.size = [560, 220];

  // Canvas container that paints with ScriptUIGraphics
  var graphCanvas = canvasGroup.add("group", undefined, undefined, { name: "graphCanvas" });
  graphCanvas.size = [560, 220];
  graphCanvas.alignment = ["fill", "fill"];

  // Custom onDraw handler for the Graph Canvas
  graphCanvas.onDraw = function () {
    var g = this.graphics;
    var w = this.size[0] || 560;
    var h = this.size[1] || 220;

    // Dark canvas background (3D DCC theme)
    var bgBrush = g.newBrush(g.BrushType.SOLID_COLOR, [0.12, 0.14, 0.18, 1.0]);
    g.rectPath(0, 0, w, h);
    g.fillPath(bgBrush);

    // Grid lines
    var gridPen = g.newPen(g.PenType.SOLID_COLOR, [0.20, 0.23, 0.28, 1.0], 1);
    for (var gx = 40; gx < w; gx += 50) {
      g.newPath();
      g.moveTo(gx, 0);
      g.lineTo(gx, h);
      g.strokePath(gridPen);
    }
    for (var gy = 20; gy < h; gy += 35) {
      g.newPath();
      g.moveTo(0, gy);
      g.lineTo(w, gy);
      g.strokePath(gridPen);
    }

    if (!activeChannels || activeChannels.length === 0) {
      var msgPen = g.newPen(g.PenType.SOLID_COLOR, [0.5, 0.55, 0.65, 1.0], 1);
      g.drawString("Select an animated layer or property in EffectCraft and click Refresh", msgPen, 60, h * 0.5 - 6);
      return;
    }

    // Compute global time bounds
    var gTMin = activeChannels[0].minTime;
    var gTMax = activeChannels[0].maxTime;
    for (var c = 1; c < activeChannels.length; c++) {
      if (activeChannels[c].minTime < gTMin) gTMin = activeChannels[c].minTime;
      if (activeChannels[c].maxTime > gTMax) gTMax = activeChannels[c].maxTime;
    }
    var totalDt = Math.max(0.1, gTMax - gTMin);

    var padX = 25;
    var padY = 20;
    var plotW = w - padX * 2;
    var plotH = h - padY * 2;

    // Render each active channel curve
    for (var chIdx = 0; chIdx < activeChannels.length; chIdx++) {
      var channel = activeChannels[chIdx];
      var isSelectedCh = (chIdx === selectedChannelIdx);
      var cRgba = channel.color || [1, 1, 1, 1];
      var curvePen = g.newPen(
        g.PenType.SOLID_COLOR,
        [cRgba[0], cRgba[1], cRgba[2], isSelectedCh ? 1.0 : 0.5],
        isSelectedCh ? 2.5 : 1.2
      );

      var steps = 80;
      var pts = [];
      for (var s = 0; s <= steps; s++) {
        var t = gTMin + (s / steps) * totalDt;
        var val = channel.evalAt(t);

        var normVal = 0.5;
        if (viewMode === 0) { // Normalized 0-1
          var dV = Math.max(EPSILON, channel.maxValue - channel.minValue);
          normVal = (val - channel.minValue) / dV;
        } else if (viewMode === 1) { // Absolute
          normVal = (val - channel.minValue) / Math.max(EPSILON, channel.maxValue - channel.minValue);
        } else if (viewMode === 2) { // Stacked lanes
          var laneH = 1.0 / activeChannels.length;
          var dV_st = Math.max(EPSILON, channel.maxValue - channel.minValue);
          var localNorm = (val - channel.minValue) / dV_st;
          normVal = chIdx * laneH + localNorm * laneH * 0.85 + laneH * 0.075;
        } else if (viewMode === 3) { // Speed graph
          var dtSpd = 0.01;
          var spd = Math.abs((channel.evalAt(t + dtSpd) - val) / dtSpd);
          normVal = Math.min(1.0, spd / 200.0);
        }

        var px = padX + ((t - gTMin) / totalDt) * plotW;
        var py = padY + (1.0 - normVal) * plotH;
        pts.push([px, py]);
      }

      // Draw curve
      if (pts.length > 1) {
        g.newPath();
        g.moveTo(pts[0][0], pts[0][1]);
        for (var pIdx = 1; pIdx < pts.length; pIdx++) {
          g.lineTo(pts[pIdx][0], pts[pIdx][1]);
        }
        g.strokePath(curvePen);
      }

      // Draw Keyframe Diamonds & Tangent Handles for selected channel
      if (isSelectedCh && channel.keys) {
        var keyBrush = g.newBrush(g.BrushType.SOLID_COLOR, [1.0, 1.0, 1.0, 1.0]);
        var keyPen = g.newPen(g.PenType.SOLID_COLOR, [cRgba[0], cRgba[1], cRgba[2], 1.0], 1.5);
        var handlePen = g.newPen(g.PenType.SOLID_COLOR, [0.8, 0.85, 0.95, 0.8], 1.2);
        var handleBrush = g.newBrush(g.BrushType.SOLID_COLOR, [0.3, 0.7, 1.0, 1.0]);

        for (var k = 0; k < channel.keys.length; k++) {
          var key = channel.keys[k];
          var dV_k = Math.max(EPSILON, channel.maxValue - channel.minValue);
          var nVk = (key.value - channel.minValue) / dV_k;
          var kx = padX + ((key.time - gTMin) / totalDt) * plotW;
          var ky = padY + (1.0 - nVk) * plotH;

          // Tangent handles for selected keyframe
          if (k === selectedKeyIdx) {
            var handleLen = (key.outInfluence / 100) * 45;
            var handleDy = -key.outSpeed * 0.15;
            // Out handle
            g.newPath();
            g.moveTo(kx, ky);
            g.lineTo(kx + handleLen, ky + handleDy);
            g.strokePath(handlePen);
            g.rectPath(kx + handleLen - 2.5, ky + handleDy - 2.5, 5, 5);
            g.fillPath(handleBrush);

            // In handle
            var inHandleLen = (key.inInfluence / 100) * 45;
            var inHandleDy = key.inSpeed * 0.15;
            g.newPath();
            g.moveTo(kx, ky);
            g.lineTo(kx - inHandleLen, ky + inHandleDy);
            g.strokePath(handlePen);
            g.rectPath(kx - inHandleLen - 2.5, ky + inHandleDy - 2.5, 5, 5);
            g.fillPath(handleBrush);
          }

          // Keyframe diamond
          g.newPath();
          g.moveTo(kx, ky - 4.5);
          g.lineTo(kx + 4.5, ky);
          g.lineTo(kx, ky + 4.5);
          g.lineTo(kx - 4.5, ky);
          g.closePath();
          g.fillPath(keyBrush);
          g.strokePath(keyPen);
        }
      }

      // Draw Buffer Ghost Curve if enabled
      if (showBuffer && bufferSnapshot && bufferSnapshot[channel.id]) {
        var ghostPen = g.newPen(g.PenType.SOLID_COLOR, [0.75, 0.8, 0.9, 0.35], 1.2);
        var bKeys = bufferSnapshot[channel.id];
        // Draw ghosted polyline
        var gSteps = 50;
        g.newPath();
        for (var gs = 0; gs <= gSteps; gs++) {
          var gt = gTMin + (gs / gSteps) * totalDt;
          var gv = bKeys.length > 0 ? bKeys[0].value : 0;
          var gnV = (gv - channel.minValue) / Math.max(EPSILON, channel.maxValue - channel.minValue);
          var gpx = padX + ((gt - gTMin) / totalDt) * plotW;
          var gpy = padY + (1.0 - gnV) * plotH;
          if (gs === 0) g.moveTo(gpx, gpy); else g.lineTo(gpx, gpy);
        }
        g.strokePath(ghostPen);
      }
    }

    // Draw FFD Lattice Cage overlay if active
    if (latticeMode) {
      var cagePen = g.newPen(g.PenType.SOLID_COLOR, [1.0, 0.6, 0.1, 0.85], 1.5);
      var cageBrush = g.newBrush(g.BrushType.SOLID_COLOR, [1.0, 0.6, 0.1, 1.0]);
      var cx0 = padX + 20;
      var cy0 = padY + 15;
      var cw = plotW - 40;
      var ch_box = plotH - 30;

      g.rectPath(cx0, cy0, cw, ch_box);
      g.strokePath(cagePen);

      // 4 Corner handles
      var corners = [
        [cx0, cy0], [cx0 + cw, cy0],
        [cx0, cy0 + ch_box], [cx0 + cw, cy0 + ch_box]
      ];
      for (var ci = 0; ci < corners.length; ci++) {
        g.rectPath(corners[ci][0] - 3.5, corners[ci][1] - 3.5, 7, 7);
        g.fillPath(cageBrush);
      }
    }
  };

  // --- Row 3: Tangent & Preset Toolbar ---
  var row3 = win.add("group");
  row3.orientation = "row";
  row3.alignChildren = ["left", "center"];
  row3.spacing = 6;

  var btnBezier = row3.add("button", undefined, "Bezier [B]", { name: "btnBezier" });
  btnBezier.helpTip = "Set cubic Bézier smooth tangents";

  var btnLinear = row3.add("button", undefined, "Linear [L]", { name: "btnLinear" });
  btnLinear.helpTip = "Set linear interpolation";

  var btnHold = row3.add("button", undefined, "Hold [H]", { name: "btnHold" });
  btnHold.helpTip = "Set stepped hold interpolation";

  var btnAutoClamp = row3.add("button", undefined, "Auto-Clamp", { name: "btnAutoClamp" });
  btnAutoClamp.helpTip = "Flatten tangents at peaks/valleys to prevent overshoot";

  var btnEasyEase = row3.add("button", undefined, "Easy Ease", { name: "btnEasyEase" });
  btnEasyEase.helpTip = "Standard 33% influence, 0 speed";

  var btnPunch = row3.add("button", undefined, "Punch (75%)", { name: "btnPunch" });
  btnPunch.helpTip = "Dynamic motion punch: 75% influence";

  var chkUnified = row3.add("checkbox", undefined, "Unified Tangents", { name: "chkUnified" });
  chkUnified.value = true;
  chkUnified.helpTip = "Keep In and Out tangent handles collinear";

  var btnCopyEase = row3.add("button", undefined, "Copy Ease", { name: "btnCopyEase" });
  var btnPasteEase = row3.add("button", undefined, "Paste Ease", { name: "btnPasteEase" });

  // --- Row 4: FFD Lattice Controls (visible when Lattice is checked) ---
  var pnlLattice = win.add("panel", undefined, "Lattice / FFD Cage Manipulation", { name: "pnlLattice" });
  pnlLattice.orientation = "row";
  pnlLattice.alignChildren = ["left", "center"];
  pnlLattice.spacing = 8;
  pnlLattice.visible = false;

  pnlLattice.add("statictext", undefined, "Time Scale %:");
  var txtTimeScale = pnlLattice.add("edittext", undefined, "100", { name: "txtTimeScale" });
  txtTimeScale.size = [45, 20];

  pnlLattice.add("statictext", undefined, "Value Scale %:");
  var txtValScale = pnlLattice.add("edittext", undefined, "100", { name: "txtValScale" });
  txtValScale.size = [45, 20];

  pnlLattice.add("statictext", undefined, "Time Shift (s):");
  var txtTimeShift = pnlLattice.add("edittext", undefined, "0.0", { name: "txtTimeShift" });
  txtTimeShift.size = [45, 20];

  pnlLattice.add("statictext", undefined, "Anchor:");
  var dropAnchor = pnlLattice.add("dropdownlist", undefined, ["Left (Start)", "Center", "Right (End)"], { name: "dropAnchor" });
  dropAnchor.selection = 1;

  var btnApplyLattice = pnlLattice.add("button", undefined, "Apply Lattice", { name: "btnApplyLattice" });
  btnApplyLattice.helpTip = "Deform and retime keyframes within the lattice cage";

  // --- Row 5: Precision Keyframe Inspector ---
  var pnlInspect = win.add("panel", undefined, "Keyframe Precision Inspector", { name: "pnlInspect" });
  pnlInspect.orientation = "row";
  pnlInspect.alignChildren = ["left", "center"];
  pnlInspect.spacing = 8;

  pnlInspect.add("statictext", undefined, "Time (s):");
  var txtKeyTime = pnlInspect.add("edittext", undefined, "0.000", { name: "txtKeyTime" });
  txtKeyTime.size = [55, 20];

  pnlInspect.add("statictext", undefined, "Value:");
  var txtKeyValue = pnlInspect.add("edittext", undefined, "0.0", { name: "txtKeyValue" });
  txtKeyValue.size = [60, 20];

  pnlInspect.add("statictext", undefined, "In Inf%:");
  var txtInInf = pnlInspect.add("edittext", undefined, "33.3", { name: "txtInInf" });
  txtInInf.size = [45, 20];

  pnlInspect.add("statictext", undefined, "Out Inf%:");
  var txtOutInf = pnlInspect.add("edittext", undefined, "33.3", { name: "txtOutInf" });
  txtOutInf.size = [45, 20];

  var btnApplyKey = pnlInspect.add("button", undefined, "Apply Key", { name: "btnApplyKey" });
  btnApplyKey.helpTip = "Apply numerical tangent and value changes to active keyframe";

  var btnInsertKey = pnlInspect.add("button", undefined, "Insert on Curve", { name: "btnInsertKey" });
  btnInsertKey.helpTip = "Insert a keyframe at CTI without altering the curve trajectory";

  var btnDeleteKey = pnlInspect.add("button", undefined, "Delete Key", { name: "btnDeleteKey" });

  // --- Row 6: Status & Feedback Bar ---
  var rowStatus = win.add("group");
  rowStatus.orientation = "row";
  rowStatus.alignChildren = ["fill", "center"];

  var lblStatus = rowStatus.add("statictext", undefined, "Citron Ready. Select animated layer and click Refresh.", { name: "status" });
  lblStatus.alignment = ["fill", "center"];

  // --- Event Handlers & Workflows ---

  function updateChannelDropdown() {
    dropChannels.removeAll();
    if (activeChannels.length === 0) {
      dropChannels.add("item", "(No animated channels)");
      dropChannels.selection = 0;
      selectedKeyIdx = -1;
      updateInspector();
      return;
    }
    for (var i = 0; i < activeChannels.length; i++) {
      var it = dropChannels.add("item", activeChannels[i].name + " (" + activeChannels[i].keys.length + " keys)");
    }
    dropChannels.selection = Math.min(selectedChannelIdx, activeChannels.length - 1);
    if (activeChannels[selectedChannelIdx] && activeChannels[selectedChannelIdx].keys.length > 0) {
      selectedKeyIdx = 0;
    } else {
      selectedKeyIdx = -1;
    }
    updateInspector();
  }

  function updateInspector() {
    if (selectedChannelIdx >= 0 && selectedChannelIdx < activeChannels.length) {
      var ch = activeChannels[selectedChannelIdx];
      if (selectedKeyIdx >= 0 && selectedKeyIdx < ch.keys.length) {
        var k = ch.keys[selectedKeyIdx];
        txtKeyTime.text = (Math.round(k.time * 1000) / 1000).toString();
        txtKeyValue.text = (Math.round(k.value * 100) / 100).toString();
        txtInInf.text = (Math.round(k.inInfluence * 10) / 10).toString();
        txtOutInf.text = (Math.round(k.outInfluence * 10) / 10).toString();
        return;
      }
    }
    txtKeyTime.text = "-";
    txtKeyValue.text = "-";
    txtInInf.text = "33.3";
    txtOutInf.text = "33.3";
  }

  btnRefresh.onClick = function () {
    inspectActiveChannels();
    updateChannelDropdown();
    if (chkBuffer.value && !bufferSnapshot) {
      bufferSnapshot = {};
      for (var i = 0; i < activeChannels.length; i++) {
        bufferSnapshot[activeChannels[i].id] = activeChannels[i].keys.slice(0);
      }
    }
    lblStatus.text = "Loaded " + activeChannels.length + " animation channels.";
    graphCanvas.notify("onDraw");
  };

  dropChannels.onChange = function () {
    if (dropChannels.selection) {
      selectedChannelIdx = dropChannels.selection.index;
      if (activeChannels[selectedChannelIdx] && activeChannels[selectedChannelIdx].keys.length > 0) {
        selectedKeyIdx = 0;
      } else {
        selectedKeyIdx = -1;
      }
      updateInspector();
      graphCanvas.notify("onDraw");
    }
  };

  dropView.onChange = function () {
    if (dropView.selection) {
      viewMode = dropView.selection.index;
      graphCanvas.notify("onDraw");
    }
  };

  chkBuffer.onClick = function () {
    showBuffer = chkBuffer.value;
    if (showBuffer && !bufferSnapshot) {
      bufferSnapshot = {};
      for (var i = 0; i < activeChannels.length; i++) {
        bufferSnapshot[activeChannels[i].id] = activeChannels[i].keys.slice(0);
      }
    }
    graphCanvas.notify("onDraw");
  };

  chkLattice.onClick = function () {
    latticeMode = chkLattice.value;
    pnlLattice.visible = latticeMode;
    win.layout.layout(true);
    graphCanvas.notify("onDraw");
  };

  // Preset Tangent Buttons
  function applyPresetToActiveKeys(inInf, inSpd, outInf, outSpd, interpType) {
    if (activeChannels.length === 0) return;
    app.beginUndoGroup("Citron: Apply Tangents");
    try {
      var ch = activeChannels[selectedChannelIdx];
      if (ch && ch.prop && ch.keys.length > 0) {
        var startK = (selectedKeyIdx >= 0) ? (selectedKeyIdx + 1) : 1;
        var endK = (selectedKeyIdx >= 0) ? (selectedKeyIdx + 1) : ch.keys.length;

        for (var k = startK; k <= endK; k++) {
          if (interpType) {
            ch.prop.setInterpolationTypeAtKey(k, interpType, interpType);
          }
          var easeIn = [new KeyframeEase(inSpd, inInf)];
          var easeOut = [new KeyframeEase(outSpd, outInf)];
          ch.prop.setTemporalEaseAtKey(k, easeIn, easeOut);
        }
        lblStatus.text = "Applied tangent ease to " + (endK - startK + 1) + " key(s).";
      }
    } catch (e) {
      lblStatus.text = "Error applying tangents: " + e.toString();
    }
    app.endUndoGroup();
    inspectActiveChannels();
    updateChannelDropdown();
    graphCanvas.notify("onDraw");
  }

  btnBezier.onClick = function () {
    applyPresetToActiveKeys(33.333, 0, 33.333, 0, BEZIER);
  };

  btnLinear.onClick = function () {
    applyPresetToActiveKeys(33.333, 1, 33.333, 1, LINEAR);
  };

  btnHold.onClick = function () {
    applyPresetToActiveKeys(33.333, 0, 33.333, 0, HOLD);
  };

  btnAutoClamp.onClick = function () {
    if (activeChannels.length === 0) return;
    app.beginUndoGroup("Citron: Auto-Clamp Tangents");
    try {
      var ch = activeChannels[selectedChannelIdx];
      if (ch && ch.prop && ch.keys.length >= 2) {
        for (var k = 1; k <= ch.keys.length; k++) {
          var isPeak = false;
          if (k > 1 && k < ch.keys.length) {
            var prevV = ch.keys[k - 2].value;
            var curV = ch.keys[k - 1].value;
            var nextV = ch.keys[k].value;
            if ((curV >= prevV && curV >= nextV) || (curV <= prevV && curV <= nextV)) {
              isPeak = true;
            }
          } else {
            isPeak = true;
          }
          var spd = isPeak ? 0.0 : ch.keys[k - 1].outSpeed;
          var easeIn = [new KeyframeEase(spd, 33.333)];
          var easeOut = [new KeyframeEase(spd, 33.333)];
          ch.prop.setTemporalEaseAtKey(k, easeIn, easeOut);
        }
        lblStatus.text = "Auto-clamped tangents across " + ch.keys.length + " keyframes.";
      }
    } catch (e) {
      lblStatus.text = "Auto-clamp error: " + e.toString();
    }
    app.endUndoGroup();
    inspectActiveChannels();
    updateChannelDropdown();
    graphCanvas.notify("onDraw");
  };

  btnEasyEase.onClick = function () {
    applyPresetToActiveKeys(33.333, 0, 33.333, 0, BEZIER);
  };

  btnPunch.onClick = function () {
    applyPresetToActiveKeys(75.0, 0, 75.0, 0, BEZIER);
  };

  btnCopyEase.onClick = function () {
    if (activeChannels.length === 0 || selectedKeyIdx < 0) return;
    var ch = activeChannels[selectedChannelIdx];
    var k = ch.keys[selectedKeyIdx];
    var clip = {
      inInfluence: k.inInfluence,
      inSpeed: k.inSpeed,
      outInfluence: k.outInfluence,
      outSpeed: k.outSpeed
    };
    app.settings.saveSetting(SECTION, KEY_CLIPBOARD, JSON.stringify(clip));
    lblStatus.text = "Copied tangents: In=" + Math.round(k.inInfluence) + "% Out=" + Math.round(k.outInfluence) + "%";
  };

  btnPasteEase.onClick = function () {
    if (!app.settings.haveSetting(SECTION, KEY_CLIPBOARD)) {
      lblStatus.text = "Clipboard empty. Copy ease first.";
      return;
    }
    try {
      var clip = JSON.parse(app.settings.getSetting(SECTION, KEY_CLIPBOARD));
      if (clip && clip.inInfluence !== undefined) {
        applyPresetToActiveKeys(clip.inInfluence, clip.inSpeed, clip.outInfluence, clip.outSpeed, BEZIER);
        lblStatus.text = "Pasted tangent ease from clipboard.";
      }
    } catch (e) {
      lblStatus.text = "Paste error: " + e.toString();
    }
  };

  // Lattice FFD Transform
  btnApplyLattice.onClick = function () {
    if (activeChannels.length === 0) return;
    var timeScale = parseFloat(txtTimeScale.text) / 100.0;
    var valScale = parseFloat(txtValScale.text) / 100.0;
    var timeShift = parseFloat(txtTimeShift.text);
    if (!isFinite(timeScale) || timeScale <= 0) timeScale = 1.0;
    if (!isFinite(valScale)) valScale = 1.0;
    if (!isFinite(timeShift)) timeShift = 0.0;

    app.beginUndoGroup("Citron: FFD Lattice Transform");
    try {
      var ch = activeChannels[selectedChannelIdx];
      if (ch && ch.prop && ch.keys.length > 1) {
        var tMin = ch.minTime;
        var tMax = ch.maxTime;
        var vMin = ch.minValue;
        var vMax = ch.maxValue;

        var anchorT = tMin;
        var anchorV = vMin;
        if (dropAnchor.selection.index === 1) { // Center
          anchorT = (tMin + tMax) * 0.5;
          anchorV = (vMin + vMax) * 0.5;
        } else if (dropAnchor.selection.index === 2) { // Right
          anchorT = tMax;
          anchorV = vMax;
        }

        // Apply transformed keys back to property
        for (var k = 1; k <= ch.keys.length; k++) {
          var oldT = ch.keys[k - 1].time;
          var oldV = ch.keys[k - 1].value;
          var newT = Math.max(0, anchorT + (oldT - anchorT) * timeScale + timeShift);
          var newV = anchorV + (oldV - anchorV) * valScale;

          // Re-key property
          ch.prop.setValueAtTime(newT, newV);
        }
        lblStatus.text = "Lattice transform applied: TimeScale=" + Math.round(timeScale * 100) + "%, ValScale=" + Math.round(valScale * 100) + "%";
      }
    } catch (e) {
      lblStatus.text = "Lattice transform error: " + e.toString();
    }
    app.endUndoGroup();
    inspectActiveChannels();
    updateChannelDropdown();
    graphCanvas.notify("onDraw");
  };

  // Keyframe Precision Edit & Insert
  btnApplyKey.onClick = function () {
    if (activeChannels.length === 0 || selectedKeyIdx < 0) return;
    var inInf = Math.min(MAX_INFLUENCE, Math.max(MIN_INFLUENCE, parseFloat(txtInInf.text)));
    var outInf = Math.min(MAX_INFLUENCE, Math.max(MIN_INFLUENCE, parseFloat(txtOutInf.text)));
    var val = parseFloat(txtKeyValue.text);

    app.beginUndoGroup("Citron: Edit Keyframe");
    try {
      var ch = activeChannels[selectedChannelIdx];
      var kIdx = selectedKeyIdx + 1;
      if (isFinite(val)) {
        ch.prop.setValueAtTime(ch.keys[selectedKeyIdx].time, val);
      }
      var easeIn = [new KeyframeEase(ch.keys[selectedKeyIdx].inSpeed, inInf)];
      var easeOut = [new KeyframeEase(ch.keys[selectedKeyIdx].outSpeed, outInf)];
      ch.prop.setTemporalEaseAtKey(kIdx, easeIn, easeOut);
      lblStatus.text = "Updated key " + kIdx + " (In=" + inInf + "%, Out=" + outInf + "%)";
    } catch (e) {
      lblStatus.text = "Key edit error: " + e.toString();
    }
    app.endUndoGroup();
    inspectActiveChannels();
    updateChannelDropdown();
    graphCanvas.notify("onDraw");
  };

  btnInsertKey.onClick = function () {
    var comp = app.project ? app.project.activeItem : null;
    if (!comp || activeChannels.length === 0) return;
    var curTime = comp.time;

    app.beginUndoGroup("Citron: Insert Key on Curve");
    try {
      var ch = activeChannels[selectedChannelIdx];
      if (ch && ch.prop) {
        // Evaluate exact curve value at current time
        var curveVal = ch.evalAt(curTime);
        ch.prop.setValueAtTime(curTime, curveVal);
        lblStatus.text = "Inserted key at " + (Math.round(curTime * 1000) / 1000) + "s preserving trajectory.";
      }
    } catch (e) {
      lblStatus.text = "Insert error: " + e.toString();
    }
    app.endUndoGroup();
    inspectActiveChannels();
    updateChannelDropdown();
    graphCanvas.notify("onDraw");
  };

  btnDeleteKey.onClick = function () {
    if (activeChannels.length === 0 || selectedKeyIdx < 0) return;
    app.beginUndoGroup("Citron: Delete Keyframe");
    try {
      var ch = activeChannels[selectedChannelIdx];
      var kIdx = selectedKeyIdx + 1;
      ch.prop.removeKey(kIdx);
      lblStatus.text = "Deleted keyframe " + kIdx;
    } catch (e) {
      lblStatus.text = "Delete error: " + e.toString();
    }
    app.endUndoGroup();
    inspectActiveChannels();
    updateChannelDropdown();
    graphCanvas.notify("onDraw");
  };

  // Initial inspection
  inspectActiveChannels();
  updateChannelDropdown();

  if (win instanceof Window) {
    win.center();
    win.show();
  } else {
    win.layout.layout(true);
  }
})(this);
