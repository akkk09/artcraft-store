// PasteCraft: Universal clipboard asset importer and layer replacer for EffectCraft.
// Clean-room implementation for asset import, vector conversion, and in-place layer replacement.
// Dockable ScriptUI panel for EffectCraft (Window ▸ PasteCraft.jsx).

(function (thisObj) {
  "use strict";

  var SECTION = "PasteCraft";

  function getPref(key, fallback) {
    try {
      if (app.settings.haveSetting(SECTION, key)) {
        return app.settings.getSetting(SECTION, key);
      }
    } catch (e) {}
    return fallback;
  }

  function setPref(key, value) {
    try {
      app.settings.saveSetting(SECTION, key, String(value));
    } catch (e) {}
  }

  var ui = thisObj instanceof Panel ? thisObj : new Window("palette", "PasteCraft", undefined, { resizeable: true });
  ui.orientation = "column";
  ui.alignChildren = ["fill", "top"];
  ui.spacing = 6;
  ui.margins = 10;

  // Header
  var grpHeader = ui.add("group");
  grpHeader.orientation = "row";
  grpHeader.alignChildren = ["left", "center"];
  grpHeader.add("statictext", undefined, "PASTECRAFT").graphics.foregroundColor = ui.graphics.newPen(ui.graphics.PenType.SOLID_COLOR, [0.95, 0.95, 0.95, 1], 1);
  var lblVer = grpHeader.add("statictext", undefined, "v0.1.0 · Clipboard Asset Importer");
  lblVer.graphics.foregroundColor = ui.graphics.newPen(ui.graphics.PenType.SOLID_COLOR, [0.6, 0.6, 0.6, 1], 1);

  // Workflow Mode
  var pnlMode = ui.add("panel", undefined, "Mode & Options");
  pnlMode.orientation = "column";
  pnlMode.alignChildren = ["fill", "top"];
  pnlMode.spacing = 6;
  pnlMode.margins = 8;

  var grpMode = pnlMode.add("group");
  grpMode.orientation = "row";
  grpMode.add("statictext", undefined, "Action:");
  var dropMode = grpMode.add("dropdownlist", undefined, ["Paste as New Layer(s)", "Paste & Replace Selected"]);
  dropMode.selection = parseInt(getPref("modeIndex", "0"), 10) || 0;
  dropMode.name = "dropMode";

  var grpSvg = pnlMode.add("group");
  grpSvg.orientation = "row";
  grpSvg.add("statictext", undefined, "SVG Format:");
  var dropSvgMode = grpSvg.add("dropdownlist", undefined, ["Vector Footage", "Convert to Editable Shapes"]);
  dropSvgMode.selection = parseInt(getPref("svgIndex", "0"), 10) || 0;
  dropSvgMode.name = "dropSvgMode";

  var grpNaming = pnlMode.add("group");
  grpNaming.orientation = "row";
  grpNaming.add("statictext", undefined, "Asset Prefix:");
  var txtPrefix = grpNaming.add("edittext", undefined, getPref("assetPrefix", "Pasted_Asset"));
  txtPrefix.characters = 12;
  txtPrefix.name = "txtPrefix";

  grpNaming.add("statictext", undefined, "Folder:");
  var txtFolder = grpNaming.add("edittext", undefined, getPref("assetFolder", "(Assets)"));
  txtFolder.characters = 10;
  txtFolder.name = "txtFolder";

  // Clipboard Input & Inspector
  var pnlInput = ui.add("panel", undefined, "Clipboard Data / URL / SVG / File");
  pnlInput.orientation = "column";
  pnlInput.alignChildren = ["fill", "top"];
  pnlInput.spacing = 6;
  pnlInput.margins = 8;

  var txtInput = pnlInput.add("edittext", undefined, "", { multiline: true, scrollable: true });
  txtInput.preferredSize.height = 70;
  txtInput.name = "txtInput";

  var grpInspect = pnlInput.add("group");
  grpInspect.orientation = "row";
  var btnInspect = grpInspect.add("button", undefined, "Inspect Data");
  btnInspect.name = "btnInspect";
  var btnClear = grpInspect.add("button", undefined, "Clear");
  btnClear.name = "btnClear";
  var lblDetected = grpInspect.add("statictext", undefined, "[Format: Ready]");
  lblDetected.name = "lblDetected";
  lblDetected.graphics.foregroundColor = ui.graphics.newPen(ui.graphics.PenType.SOLID_COLOR, [0.7, 0.7, 0.7, 1], 1);

  // Destructive Replace Preview & Confirmation
  var pnlConfirm = ui.add("panel", undefined, "Destructive Change Safety");
  pnlConfirm.orientation = "column";
  pnlConfirm.alignChildren = ["fill", "top"];
  pnlConfirm.spacing = 4;
  pnlConfirm.margins = 8;
  pnlConfirm.name = "pnlConfirm";

  var lblSafetyNote = pnlConfirm.add("statictext", undefined, "Replace mode swaps footage while preserving keyframes, effects & timing.");
  lblSafetyNote.graphics.foregroundColor = ui.graphics.newPen(ui.graphics.PenType.SOLID_COLOR, [0.65, 0.65, 0.65, 1], 1);

  var chkConfirm = pnlConfirm.add("checkbox", undefined, "Confirm replacement of selected layers");
  chkConfirm.value = false;
  chkConfirm.name = "chkConfirm";

  // Actions
  var grpActions = ui.add("group");
  grpActions.orientation = "row";
  grpActions.alignChildren = ["fill", "center"];
  grpActions.spacing = 8;

  var btnPaste = grpActions.add("button", undefined, "Paste to Timeline");
  btnPaste.name = "btnPaste";

  var btnReplace = grpActions.add("button", undefined, "Paste & Replace");
  btnReplace.name = "btnReplace";

  // Status Bar
  var grpStatus = ui.add("group");
  grpStatus.orientation = "row";
  grpStatus.alignChildren = ["fill", "center"];
  var status = grpStatus.add("statictext", undefined, "Ready. Paste data above or copy an asset to clipboard.");
  status.name = "status";
  status.graphics.foregroundColor = ui.graphics.newPen(ui.graphics.PenType.SOLID_COLOR, [0.5, 0.8, 0.5, 1], 1);

  // Helper Functions
  function setStatus(msg, isError) {
    status.text = msg;
    if (isError) {
      status.graphics.foregroundColor = ui.graphics.newPen(ui.graphics.PenType.SOLID_COLOR, [1, 0.4, 0.4, 1], 1);
    } else {
      status.graphics.foregroundColor = ui.graphics.newPen(ui.graphics.PenType.SOLID_COLOR, [0.4, 0.9, 0.4, 1], 1);
    }
  }

  function inspectClipboard() {
    var raw = txtInput.text;
    if (!raw || !raw.replace(/\s+/g, "")) {
      lblDetected.text = "[Empty / No data]";
      setStatus("Input is empty. Paste image data, SVG code, URL, or file path.", true);
      return;
    }
    try {
      var r = app.pasteCraftPreview({ data: raw });
      if (r && r.inspection) {
        lblDetected.text = "[" + (r.inspection.format || r.inspection.type || "Detected") + ": " + (r.inspection.description || "") + "]";
        setStatus("Detected: " + (r.inspection.description || r.inspection.type), false);
      }
    } catch (e) {
      lblDetected.text = "[Detected: Text]";
    }
  }

  function executePaste(forceReplace) {
    var raw = txtInput.text;
    if (!raw || !raw.replace(/\s+/g, "")) {
      setStatus("Clipboard is empty. Copy an image, vector SVG, file path, or URL first.", true);
      return;
    }

    var isReplace = forceReplace || (dropMode.selection && dropMode.selection.index === 1);
    var svgMode = (dropSvgMode.selection && dropSvgMode.selection.index === 1) ? "shapes" : "footage";
    var prefix = txtPrefix.text || "Pasted_Asset";
    var folder = txtFolder.text || "(Assets)";

    // Verify confirmation for destructive replace
    if (isReplace && !chkConfirm.value && !forceReplace) {
      setStatus("Destructive change: Please check 'Confirm replacement' before proceeding.", true);
      return;
    }

    app.beginUndoGroup(isReplace ? "PasteCraft: Replace Layer" : "PasteCraft: Paste Asset");
    try {
      var params = {
        data: raw,
        mode: isReplace ? "replace" : "newLayer",
        svgMode: svgMode,
        assetName: prefix,
        assetDir: folder,
        confirm: chkConfirm.value || forceReplace
      };

      var res = app.pasteCraft(params);
      if (res && res.requiresConfirmation) {
        setStatus("Destructive preview: " + res.message + " Check confirm and re-run.", true);
      } else if (res && res.status === "success") {
        setStatus("✓ " + (res.message || "Operation completed successfully."), false);
        setPref("assetPrefix", prefix);
        setPref("assetFolder", folder);
      } else {
        setStatus("✓ Pasted asset successfully.", false);
      }
    } catch (err) {
      setStatus("Error: " + (err.message || String(err)), true);
    } finally {
      app.endUndoGroup();
    }
  }

  // Event Handlers
  btnInspect.onClick = inspectClipboard;

  btnClear.onClick = function () {
    txtInput.text = "";
    lblDetected.text = "[Format: Ready]";
    setStatus("Cleared input.", false);
  };

  btnPaste.onClick = function () {
    executePaste(false);
  };

  btnReplace.onClick = function () {
    executePaste(true);
  };

  dropMode.onChange = function () {
    setPref("modeIndex", dropMode.selection.index);
    if (dropMode.selection.index === 1) {
      pnlConfirm.visible = true;
    }
  };

  dropSvgMode.onChange = function () {
    setPref("svgIndex", dropSvgMode.selection.index);
  };

  txtInput.onChanging = function () {
    if (txtInput.text.length > 5 && lblDetected.text === "[Format: Ready]") {
      inspectClipboard();
    }
  };

  if (ui instanceof Window) {
    ui.center();
    ui.show();
  } else {
    ui.layout.layout(true);
  }

})(this);
