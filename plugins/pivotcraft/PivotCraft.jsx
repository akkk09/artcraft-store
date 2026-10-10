// PivotCraft: Anchor-point placement, layer alignment, and transform compensation tool for EffectCraft.
//
// A ScriptUI panel extension for EffectCraft (Window ▸ PivotCraft.jsx).
// Built against EffectCraft's public scripting API:
// comp.selectedLayers, layer.transform, layer.sourceRectAtTime, dimensionsSeparated,
// threeDLayer, app.beginUndoGroup, app.settings, and ScriptUI.
// Original work by EffectCraft contributors under MIT OR Apache-2.0.

(function pivotCraft(thisObj) {
  var SECTION = "PivotCraft";
  var KEY_USER_PRESETS = "userPresets";
  var KEY_SETTINGS = "pivotSettings";

  // Built-in standard presets
  var BUILT_IN_PRESETS = [
    { name: "Center (Default)", point: "center", norm: [0.5, 0.5, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Top-Left (Corner)", point: "topLeft", norm: [0, 0, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Top-Center (Edge)", point: "topCenter", norm: [0.5, 0, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Top-Right (Corner)", point: "topRight", norm: [1, 0, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Middle-Left (Edge)", point: "middleLeft", norm: [0, 0.5, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Middle-Right (Edge)", point: "middleRight", norm: [1, 0.5, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Bottom-Left (Corner)", point: "bottomLeft", norm: [0, 1, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Bottom-Center (Edge)", point: "bottomCenter", norm: [0.5, 1, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Bottom-Right (Corner)", point: "bottomRight", norm: [1, 1, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Lower-Third Anchor (0%, 80%)", point: "custom", norm: [0, 0.8, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Title Baseline (50%, 100%)", point: "custom", norm: [0.5, 1, 0], mode: 0, thresh: 5, off: [0, 0, 0] },
    { name: "Alpha-Aware Center (5% Threshold)", point: "center", norm: [0.5, 0.5, 0], mode: 1, thresh: 5, off: [0, 0, 0] }
  ];

  function loadUserPresets() {
    var out = [];
    if (!app.settings.haveSetting(SECTION, KEY_USER_PRESETS)) return out;
    try {
      var doc = JSON.parse(app.settings.getSetting(SECTION, KEY_USER_PRESETS));
      var list = doc && doc.presets instanceof Array ? doc.presets : [];
      for (var i = 0; i < list.length; i++) {
        var p = list[i];
        if (p && p.name) out.push(p);
      }
    } catch (e) {}
    return out;
  }

  function saveUserPresets(list) {
    var doc = {
      format: "pivotcraft-presets",
      version: 1,
      presets: list
    };
    app.settings.saveSetting(SECTION, KEY_USER_PRESETS, JSON.stringify(doc));
  }

  var userPresets = loadUserPresets();

  // --- Geometry and Transform Math Helpers ---

  function getLayerBounds(layer, isAlphaMode, threshold) {
    var time = 0;
    try {
      var comp = layer.containingComp || app.project.activeItem;
      if (comp) time = comp.time;
    } catch (e) {}

    var left = 0, top = 0, width = 0, height = 0;

    // First attempt: sourceRectAtTime
    try {
      if (typeof layer.sourceRectAtTime === "function") {
        var rect = layer.sourceRectAtTime(time, false);
        if (rect && rect.width > 0 && rect.height > 0) {
          left = rect.left;
          top = rect.top;
          width = rect.width;
          height = rect.height;
        }
      }
    } catch (e) {}

    // Fallback for solids/footage if sourceRect was empty or zero
    if (width <= 0 || height <= 0) {
      if (layer.width && layer.height) {
        left = 0;
        top = 0;
        width = layer.width;
        height = layer.height;
      } else {
        left = 0;
        top = 0;
        width = 100;
        height = 100;
      }
    }

    // Alpha mode adjustment: if alpha threshold is set (> 0) on vector/shape/masked layers,
    // bounds are adjusted inward proportionally by threshold factor
    if (isAlphaMode && threshold > 0) {
      var inset = Math.min(width * 0.45, Math.min(height * 0.45, (threshold / 100) * Math.min(width, height) * 0.1));
      left += inset;
      top += inset;
      width = Math.max(1, width - inset * 2);
      height = Math.max(1, height - inset * 2);
    }

    return {
      left: left,
      top: top,
      width: width,
      height: height,
      right: left + width,
      bottom: top + height
    };
  }

  // 2D Rotation & Scale Matrix Shift
  function computeShift2D(deltaA, scale, rotationDeg) {
    var rad = rotationDeg * Math.PI / 180.0;
    var cs = Math.cos(rad);
    var sn = Math.sin(rad);
    var dx = deltaA[0] * (scale[0] / 100.0);
    var dy = deltaA[1] * (scale[1] / 100.0);
    return [
      dx * cs - dy * sn,
      dx * sn + dy * cs,
      0
    ];
  }

  // 3D Matrix Multiplication & Transformation Shift
  function computeShift3D(deltaA, scale, orient, rot) {
    var rad = Math.PI / 180.0;
    // Orientation angles: Z, Y, X
    // Rotations: rx, ry, rz
    var ox = orient[0] * rad, oy = orient[1] * rad, oz = orient[2] * rad;
    var rx = rot[0] * rad, ry = rot[1] * rad, rz = rot[2] * rad;

    // Helper to multiply 3x3 matrices
    function m3Mul(a, b) {
      var r = [[0,0,0],[0,0,0],[0,0,0]];
      for (var i = 0; i < 3; i++) {
        for (var j = 0; j < 3; j++) {
          r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
      }
      return r;
    }

    function rotX(a) {
      var c = Math.cos(a), s = Math.sin(a);
      return [[1, 0, 0], [0, c, -s], [0, s, c]];
    }
    function rotY(a) {
      var c = Math.cos(a), s = Math.sin(a);
      return [[c, 0, s], [0, 1, 0], [-s, 0, c]];
    }
    function rotZ(a) {
      var c = Math.cos(a), s = Math.sin(a);
      return [[c, -s, 0], [s, c, 0], [0, 0, 1]];
    }

    var mOrient = m3Mul(rotZ(oz), m3Mul(rotY(oy), rotX(ox)));
    var mRot = m3Mul(rotZ(rz), m3Mul(rotY(ry), rotX(rx)));
    var mScale = [
      [scale[0] / 100.0, 0, 0],
      [0, scale[1] / 100.0, 0],
      [0, 0, (scale[2] || 100.0) / 100.0]
    ];

    var linear = m3Mul(mOrient, m3Mul(mRot, mScale));
    return [
      linear[0][0] * deltaA[0] + linear[0][1] * deltaA[1] + linear[0][2] * deltaA[2],
      linear[1][0] * deltaA[0] + linear[1][1] * deltaA[1] + linear[1][2] * deltaA[2],
      linear[2][0] * deltaA[0] + linear[2][1] * deltaA[1] + linear[2][2] * deltaA[2]
    ];
  }

  // --- Main Pivot Application Logic ---

  function applyPivotToLayer(layer, nx, ny, nz, offX, offY, offZ, isAlpha, thresh, preservePos, targetMode) {
    var is3D = !!layer.threeDLayer;
    var b = getLayerBounds(layer, isAlpha, thresh);

    // Target anchor in layer local space
    var targetX = b.left + b.width * nx + offX;
    var targetY = b.top + b.height * ny + offY;

    // Handle Transform Effect targeting
    if (targetMode === "effect") {
      var fx = null;
      try {
        fx = layer.property("ADBE Effect Parade").property("ADBE Transform") || layer.effect("Transform");
      } catch (e) {}
      if (fx) {
        var fxAnchor = fx.property("Anchor Point") || fx.property(1);
        var fxPos = fx.property("Position") || fx.property(2);
        var fxScale = fx.property("Scale") || fx.property(4);
        var fxRot = fx.property("Rotation") || fx.property(6);

        if (fxAnchor && fxPos) {
          var oldA = fxAnchor.value;
          var curScale = fxScale ? (typeof fxScale.value === "number" ? [fxScale.value, fxScale.value] : fxScale.value) : [100, 100];
          var curRot = fxRot ? fxRot.value : 0;
          var deltaA = [targetX - oldA[0], targetY - oldA[1], 0];

          if (fxAnchor.numKeys > 0) {
            for (var k = 1; k <= fxAnchor.numKeys; k++) {
              var kv = fxAnchor.keyValue(k);
              fxAnchor.setValueAtKey(k, [kv[0] + deltaA[0], kv[1] + deltaA[1]]);
            }
          } else {
            fxAnchor.setValue([targetX, targetY]);
          }

          if (preservePos) {
            var shift = computeShift2D(deltaA, curScale, curRot);
            if (fxPos.numKeys > 0) {
              for (var j = 1; j <= fxPos.numKeys; j++) {
                var pv = fxPos.keyValue(j);
                fxPos.setValueAtKey(j, [pv[0] + shift[0], pv[1] + shift[1]]);
              }
            } else {
              var curP = fxPos.value;
              fxPos.setValue([curP[0] + shift[0], curP[1] + shift[1]]);
            }
          }
          return true;
        }
      }
    }

    // Native Layer Transform targeting
    var tr = layer.transform;
    if (!tr || !tr.anchorPoint) return false;

    var curA = tr.anchorPoint.value;
    var oldAx = curA[0], oldAy = curA[1], oldAz = is3D ? (curA[2] || 0) : 0;
    var targetZ = is3D ? (oldAz + offZ) : 0;
    var delta = [targetX - oldAx, targetY - oldAy, targetZ - oldAz];

    // Compute shift vector to preserve layer's visible position
    var shift = [0, 0, 0];
    if (preservePos) {
      var curScale = tr.scale ? tr.scale.value : [100, 100, 100];
      if (is3D) {
        var orient = tr.orientation ? tr.orientation.value : [0, 0, 0];
        var rx = tr.xRotation ? tr.xRotation.value : 0;
        var ry = tr.yRotation ? tr.yRotation.value : 0;
        var rz = tr.zRotation ? tr.zRotation.value : (tr.rotation ? tr.rotation.value : 0);
        shift = computeShift3D(delta, curScale, orient, [rx, ry, rz]);
      } else {
        var rot = tr.rotation ? tr.rotation.value : 0;
        shift = computeShift2D(delta, curScale, rot);
      }
    }

    // 1. Update Anchor Point
    var ap = tr.anchorPoint;
    if (ap.numKeys > 0) {
      for (var ak = 1; ak <= ap.numKeys; ak++) {
        var val = ap.keyValue(ak);
        if (is3D) {
          ap.setValueAtKey(ak, [val[0] + delta[0], val[1] + delta[1], (val[2] || 0) + delta[2]]);
        } else {
          ap.setValueAtKey(ak, [val[0] + delta[0], val[1] + delta[1]]);
        }
      }
    } else {
      if (is3D) {
        ap.setValue([targetX, targetY, targetZ]);
      } else {
        ap.setValue([targetX, targetY]);
      }
    }

    // 2. Compensate Position (if preservePos enabled)
    if (preservePos) {
      var posProp = tr.position;
      var isSeparated = false;
      try {
        isSeparated = !!(posProp && posProp.dimensionsSeparated);
      } catch (e) {}

      if (isSeparated) {
        var px = tr.xPosition;
        var py = tr.yPosition;
        var pz = is3D ? tr.zPosition : null;

        if (px) {
          if (px.numKeys > 0) {
            for (var kx = 1; kx <= px.numKeys; kx++) px.setValueAtKey(kx, px.keyValue(kx) + shift[0]);
          } else {
            px.setValue(px.value + shift[0]);
          }
        }
        if (py) {
          if (py.numKeys > 0) {
            for (var ky = 1; ky <= py.numKeys; ky++) py.setValueAtKey(ky, py.keyValue(ky) + shift[1]);
          } else {
            py.setValue(py.value + shift[1]);
          }
        }
        if (pz) {
          if (pz.numKeys > 0) {
            for (var kz = 1; kz <= pz.numKeys; kz++) pz.setValueAtKey(kz, pz.keyValue(kz) + shift[2]);
          } else {
            pz.setValue(pz.value + shift[2]);
          }
        }
      } else if (posProp) {
        if (posProp.numKeys > 0) {
          for (var pk = 1; pk <= posProp.numKeys; pk++) {
            var curP = posProp.keyValue(pk);
            if (is3D) {
              posProp.setValueAtKey(pk, [curP[0] + shift[0], curP[1] + shift[1], (curP[2] || 0) + shift[2]]);
            } else {
              posProp.setValueAtKey(pk, [curP[0] + shift[0], curP[1] + shift[1]]);
            }
          }
        } else {
          var curPos = posProp.value;
          if (is3D) {
            posProp.setValue([curPos[0] + shift[0], curPos[1] + shift[1], (curPos[2] || 0) + shift[2]]);
          } else {
            posProp.setValue([curPos[0] + shift[0], curPos[1] + shift[1]]);
          }
        }
      }
    }

    return true;
  }

  // --- UI Creation ---

  var ui = thisObj instanceof Panel ? thisObj : new Window("palette", "PivotCraft", undefined, { resizeable: true });
  ui.orientation = "column";
  ui.alignChildren = ["fill", "top"];
  ui.spacing = 6;
  ui.margins = 8;

  // 1. Bounds Mode & Alpha Threshold Group
  var grpMode = ui.add("group", undefined, { name: "grpMode" });
  grpMode.orientation = "row";
  grpMode.alignChildren = ["left", "center"];
  grpMode.spacing = 6;

  grpMode.add("statictext", undefined, "Bounds:");
  var dropMode = grpMode.add("dropdownlist", undefined, ["Layer Bounds", "Visible Alpha Bounds"], { name: "dropMode" });
  dropMode.selection = 0;
  dropMode.preferredSize = [140, 24];

  grpMode.add("statictext", undefined, "Alpha %:");
  var sliderThreshold = grpMode.add("slider", undefined, 5, 0, 100, { name: "sliderThreshold" });
  sliderThreshold.preferredSize = [70, 20];
  var txtThreshold = grpMode.add("edittext", undefined, "5", { name: "txtThreshold" });
  txtThreshold.preferredSize = [35, 20];

  sliderThreshold.onChanging = function () {
    txtThreshold.text = String(Math.round(sliderThreshold.value));
    updatePreview();
  };
  txtThreshold.onChange = function () {
    var v = Math.min(100, Math.max(0, parseFloat(txtThreshold.text) || 0));
    sliderThreshold.value = v;
    txtThreshold.text = String(Math.round(v));
    updatePreview();
  };
  dropMode.onChange = function () {
    updatePreview();
  };

  // 2. 3x3 Grid Buttons & Interactive Canvas
  var grpMain = ui.add("group", undefined, { name: "grpMain" });
  grpMain.orientation = "row";
  grpMain.alignChildren = ["fill", "top"];
  grpMain.spacing = 8;

  // 3x3 Grid Panel
  var pnlGrid = grpMain.add("panel", undefined, "Pivot Grid", { name: "pnlGrid" });
  pnlGrid.orientation = "column";
  pnlGrid.spacing = 4;
  pnlGrid.margins = 6;

  var row1 = pnlGrid.add("group", undefined);
  row1.spacing = 4;
  var btnTL = row1.add("button", undefined, "TL", { name: "btnTL" });
  var btnTC = row1.add("button", undefined, "TC", { name: "btnTC" });
  var btnTR = row1.add("button", undefined, "TR", { name: "btnTR" });
  btnTL.preferredSize = [36, 28];
  btnTC.preferredSize = [36, 28];
  btnTR.preferredSize = [36, 28];

  var row2 = pnlGrid.add("group", undefined);
  row2.spacing = 4;
  var btnML = row2.add("button", undefined, "ML", { name: "btnML" });
  var btnC  = row2.add("button", undefined, "C",  { name: "btnC" });
  var btnMR = row2.add("button", undefined, "MR", { name: "btnMR" });
  btnML.preferredSize = [36, 28];
  btnC.preferredSize  = [36, 28];
  btnMR.preferredSize = [36, 28];

  var row3 = pnlGrid.add("group", undefined);
  row3.spacing = 4;
  var btnBL = row3.add("button", undefined, "BL", { name: "btnBL" });
  var btnBC = row3.add("button", undefined, "BC", { name: "btnBC" });
  var btnBR = row3.add("button", undefined, "BR", { name: "btnBR" });
  btnBL.preferredSize = [36, 28];
  btnBC.preferredSize = [36, 28];
  btnBR.preferredSize = [36, 28];

  // Visual Preview Box
  var pnlPreview = grpMain.add("panel", undefined, "Pivot Preview", { name: "pnlPreview" });
  pnlPreview.orientation = "column";
  pnlPreview.alignChildren = ["center", "center"];
  pnlPreview.margins = 6;

  var previewCanvas = pnlPreview.add("group", undefined, undefined, { name: "preview" });
  previewCanvas.preferredSize = [130, 94];

  // Current preview parameters
  var previewState = {
    nx: 0.5,
    ny: 0.5,
    curAnchor: [50, 50],
    targetAnchor: [50, 50],
    bounds: { width: 100, height: 100 },
    activePoint: "center"
  };

  previewCanvas.onDraw = function () {
    var g = this.graphics;
    if (!g) return;
    var w = this.size.width;
    var h = this.size.height;

    // Draw background
    var bgBrush = g.newBrush(g.BrushType.SOLID_COLOR, [0.15, 0.16, 0.18, 1]);
    g.rectPath(0, 0, w, h);
    g.fillPath(bgBrush);

    // Draw Layer Bounds box
    var pad = 12;
    var bw = w - pad * 2;
    var bh = h - pad * 2;
    var boxPen = g.newPen(g.PenType.SOLID_COLOR, [0.4, 0.45, 0.55, 1], 1.5);
    g.rectPath(pad, pad, bw, bh);
    g.strokePath(boxPen);

    // Draw 9 grid reference dots
    var dotBrush = g.newBrush(g.BrushType.SOLID_COLOR, [0.35, 0.38, 0.42, 1]);
    for (var gx = 0; gx <= 2; gx++) {
      for (var gy = 0; gy <= 2; gy++) {
        var dx = pad + (bw * gx) / 2;
        var dy = pad + (bh * gy) / 2;
        g.rectPath(dx - 1.5, dy - 1.5, 3, 3);
        g.fillPath(dotBrush);
      }
    }

    // Draw Target Pivot Point (Emerald Green Marker)
    var tx = pad + bw * previewState.nx;
    var ty = pad + bh * previewState.ny;
    var targetPen = g.newPen(g.PenType.SOLID_COLOR, [0.1, 0.85, 0.4, 1], 2);
    // Draw crosshair
    g.newPath();
    g.moveTo(tx - 6, ty);
    g.lineTo(tx + 6, ty);
    g.moveTo(tx, ty - 6);
    g.lineTo(tx, ty + 6);
    g.strokePath(targetPen);

    var targetBrush = g.newBrush(g.BrushType.SOLID_COLOR, [0.1, 0.85, 0.4, 0.7]);
    g.rectPath(tx - 3, ty - 3, 6, 6);
    g.fillPath(targetBrush);
  };

  function updatePreview() {
    var comp = app.project.activeItem;
    if (comp instanceof CompItem && comp.selectedLayers.length > 0) {
      var l = comp.selectedLayers[0];
      var isAlpha = dropMode.selection ? dropMode.selection.index === 1 : false;
      var thresh = sliderThreshold.value;
      var b = getLayerBounds(l, isAlpha, thresh);
      previewState.bounds = b;
      if (l.transform && l.transform.anchorPoint) {
        previewState.curAnchor = l.transform.anchorPoint.value;
      }
    }
    if (typeof previewCanvas.notify === "function") {
      previewCanvas.notify("onDraw");
    }
  }

  // 3. Custom Position & Offset Controls
  var grpCustom = ui.add("group", undefined, { name: "grpCustom" });
  grpCustom.orientation = "row";
  grpCustom.alignChildren = ["left", "center"];
  grpCustom.spacing = 6;

  grpCustom.add("statictext", undefined, "Custom X%:");
  var sliderCustomX = grpCustom.add("slider", undefined, 50, 0, 100, { name: "sliderCustomX" });
  sliderCustomX.preferredSize = [60, 20];
  var txtCustomX = grpCustom.add("edittext", undefined, "50", { name: "txtCustomX" });
  txtCustomX.preferredSize = [35, 20];

  grpCustom.add("statictext", undefined, "Y%:");
  var sliderCustomY = grpCustom.add("slider", undefined, 50, 0, 100, { name: "sliderCustomY" });
  sliderCustomY.preferredSize = [60, 20];
  var txtCustomY = grpCustom.add("edittext", undefined, "50", { name: "txtCustomY" });
  txtCustomY.preferredSize = [35, 20];

  sliderCustomX.onChanging = function () {
    txtCustomX.text = String(Math.round(sliderCustomX.value));
    previewState.nx = sliderCustomX.value / 100.0;
    updatePreview();
  };
  txtCustomX.onChange = function () {
    var v = Math.min(100, Math.max(0, parseFloat(txtCustomX.text) || 0));
    sliderCustomX.value = v;
    txtCustomX.text = String(Math.round(v));
    previewState.nx = v / 100.0;
    updatePreview();
  };

  sliderCustomY.onChanging = function () {
    txtCustomY.text = String(Math.round(sliderCustomY.value));
    previewState.ny = sliderCustomY.value / 100.0;
    updatePreview();
  };
  txtCustomY.onChange = function () {
    var v = Math.min(100, Math.max(0, parseFloat(txtCustomY.text) || 0));
    sliderCustomY.value = v;
    txtCustomY.text = String(Math.round(v));
    previewState.ny = v / 100.0;
    updatePreview();
  };

  // 4. Options Row (Preserve Position, Separated Dimensions, Target)
  var grpOpts = ui.add("group", undefined, { name: "grpOpts" });
  grpOpts.orientation = "row";
  grpOpts.alignChildren = ["left", "center"];
  grpOpts.spacing = 8;

  var chkPreservePos = grpOpts.add("checkbox", undefined, "Preserve Visible Position", { name: "chkPreservePos" });
  chkPreservePos.value = true;

  var chkSeparated = grpOpts.add("checkbox", undefined, "Separated Dims", { name: "chkSeparated" });
  chkSeparated.value = true;

  grpOpts.add("statictext", undefined, "Target:");
  var dropTarget = grpOpts.add("dropdownlist", undefined, ["Layer Transform", "Transform Effect"], { name: "dropTarget" });
  dropTarget.selection = 0;
  dropTarget.preferredSize = [120, 22];

  // 5. Presets & Actions Row
  var grpPresets = ui.add("group", undefined, { name: "grpPresets" });
  grpPresets.orientation = "row";
  grpPresets.alignChildren = ["fill", "center"];
  grpPresets.spacing = 6;

  grpPresets.add("statictext", undefined, "Preset:");
  var dropPresets = grpPresets.add("dropdownlist", undefined, [], { name: "presets" });
  dropPresets.preferredSize = [150, 24];

  var btnApply = grpPresets.add("button", undefined, "Apply", { name: "apply" });
  btnApply.preferredSize = [55, 24];
  var btnSave = grpPresets.add("button", undefined, "Save...", { name: "save" });
  btnSave.preferredSize = [55, 24];
  var btnExport = grpPresets.add("button", undefined, "Export", { name: "export" });
  btnExport.preferredSize = [55, 24];

  function refreshPresetsDropdown() {
    dropPresets.removeAll();
    for (var i = 0; i < BUILT_IN_PRESETS.length; i++) {
      dropPresets.add("item", BUILT_IN_PRESETS[i].name);
    }
    for (var j = 0; j < userPresets.length; j++) {
      dropPresets.add("item", "[User] " + userPresets[j].name);
    }
    dropPresets.selection = 0;
  }
  refreshPresetsDropdown();

  // 6. Status Text
  var statusText = ui.add("statictext", undefined, "Ready. Select layers to align.", { name: "status" });
  statusText.preferredSize = [-1, 20];

  // --- Button Handlers ---

  function applyPivotGrid(pointName, nx, ny) {
    previewState.nx = nx;
    previewState.ny = ny;
    previewState.activePoint = pointName;
    sliderCustomX.value = nx * 100;
    txtCustomX.text = String(Math.round(nx * 100));
    sliderCustomY.value = ny * 100;
    txtCustomY.text = String(Math.round(ny * 100));
    updatePreview();

    var comp = app.project.activeItem;
    if (!(comp instanceof CompItem)) {
      statusText.text = "Error: Open a composition first.";
      return;
    }
    var layers = comp.selectedLayers;
    if (!layers || layers.length === 0) {
      statusText.text = "Select one or more layers.";
      return;
    }

    var isAlpha = dropMode.selection ? dropMode.selection.index === 1 : false;
    var thresh = sliderThreshold.value;
    var preserve = chkPreservePos.value;
    var targetMode = (dropTarget.selection && dropTarget.selection.index === 1) ? "effect" : "transform";

    app.beginUndoGroup("PivotCraft: Set Anchor Point");
    var count = 0;
    try {
      for (var i = 0; i < layers.length; i++) {
        if (applyPivotToLayer(layers[i], nx, ny, 0, 0, 0, 0, isAlpha, thresh, preserve, targetMode)) {
          count++;
        }
      }
    } finally {
      app.endUndoGroup();
    }

    statusText.text = "Aligned " + count + " layer(s) to " + pointName + ".";
    updatePreview();
  }

  // Wire 9-grid buttons
  btnTL.onClick = function () { applyPivotGrid("Top-Left", 0.0, 0.0); };
  btnTC.onClick = function () { applyPivotGrid("Top-Center", 0.5, 0.0); };
  btnTR.onClick = function () { applyPivotGrid("Top-Right", 1.0, 0.0); };
  btnML.onClick = function () { applyPivotGrid("Middle-Left", 0.0, 0.5); };
  btnC.onClick  = function () { applyPivotGrid("Center", 0.5, 0.5); };
  btnMR.onClick = function () { applyPivotGrid("Middle-Right", 1.0, 0.5); };
  btnBL.onClick = function () { applyPivotGrid("Bottom-Left", 0.0, 1.0); };
  btnBC.onClick = function () { applyPivotGrid("Bottom-Center", 0.5, 1.0); };
  btnBR.onClick = function () { applyPivotGrid("Bottom-Right", 1.0, 1.0); };

  btnApply.onClick = function () {
    var idx = dropPresets.selection ? dropPresets.selection.index : 0;
    var preset = idx < BUILT_IN_PRESETS.length ? BUILT_IN_PRESETS[idx] : userPresets[idx - BUILT_IN_PRESETS.length];
    if (preset && preset.norm) {
      if (preset.mode !== undefined) dropMode.selection = preset.mode;
      if (preset.thresh !== undefined) {
        sliderThreshold.value = preset.thresh;
        txtThreshold.text = String(preset.thresh);
      }
      applyPivotGrid(preset.name, preset.norm[0], preset.norm[1]);
    } else {
      applyPivotGrid("Custom", previewState.nx, previewState.ny);
    }
  };

  btnSave.onClick = function () {
    var name = prompt("Enter a name for the new preset:", "Custom Preset " + (userPresets.length + 1));
    if (name && name.length > 0) {
      var isAlpha = dropMode.selection ? dropMode.selection.index === 1 : false;
      var newP = {
        name: name,
        point: "custom",
        norm: [previewState.nx, previewState.ny, 0],
        mode: isAlpha ? 1 : 0,
        thresh: sliderThreshold.value,
        off: [0, 0, 0]
      };
      userPresets.push(newP);
      saveUserPresets(userPresets);
      refreshPresetsDropdown();
      dropPresets.selection = BUILT_IN_PRESETS.length + userPresets.length - 1;
      statusText.text = "Saved preset: " + name;
    }
  };

  btnExport.onClick = function () {
    var all = BUILT_IN_PRESETS.concat(userPresets);
    var doc = {
      format: "pivotcraft-presets",
      version: 1,
      presets: all
    };
    app.settings.saveSetting(SECTION, "lastExported", JSON.stringify(doc));
    statusText.text = "Exported " + all.length + " presets to settings.";
  };

  if (ui instanceof Window) ui.show(); else ui.layout.layout(true);
})(this);
