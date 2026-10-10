# PasteCraft

**PasteCraft** is an original EffectCraft workflow utility and ScriptUI panel for direct clipboard asset importing, vector shape conversion, and in-place layer replacement.

## Key Features

1. **Multi-Format Clipboard Detection**:
   - **Bitmaps & Images**: Decodes PNG, JPEG, GIF, and WebP data URIs or raw clipboard data.
   - **SVG Vectors**: Direct SVG markup detection with option to import as vector footage or convert directly into editable Shape Layers (`layer.create shapesFromVector`).
   - **Web Asset URLs**: Automatically downloads `http(s)://` images and links directly into the project assets folder.
   - **Local File Paths**: Detects single files or batch multi-line file paths from file managers.

2. **Paste as New Layer**:
   - Imports assets into the project folder (`(Assets)/`).
   - Automatically adds footage or shapes to the active composition at the current playhead time, centered.

3. **Paste & Replace (Destructive Change Safety)**:
   - In-place source footage replacement for one or more selected layers.
   - **100% Transform & Animation Preservation**: Retains all position, scale, rotation, anchor, opacity keyframes, expressions, effects stack, masks, timing (in-point/out-point/start-time), and parent links.
   - **Confirmation Step**: Generates an explicit destructive preview detailing target layers, incoming asset metadata, and preserved properties before committing changes.

4. **Batch Operations**:
   - Paste multiple files or URLs simultaneously as stacked layers.
   - Batch replace multiple selected layers with pasted assets.

5. **Full Undo Support**:
   - Every operation is wrapped in a single atomic undo group (`PasteCraft: ...`), allowing instant one-step revert.

## Usage

### In EffectCraft (ScriptUI Panel)
1. Open **Window ▸ PasteCraft.jsx**.
2. Paste or copy your asset (image, SVG, URL, or file path).
3. Select your mode:
   - **Paste as New Layer** (default)
   - **Paste & Replace Selected Layers**
4. Click **Paste to Timeline** or **Paste & Replace**.
