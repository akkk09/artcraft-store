use eframe::egui::{self, Align, Color32, FontId, Frame, Layout, RichText, Stroke, Vec2};
use serde::Deserialize;

const BASE_URL: &str = "https://akkk09.github.io/artcraft-store/";
const REPO_URL: &str = "https://github.com/akkk09/artcraft-store";
const REVIEW_URL: &str = "https://github.com/akkk09/artcraft-store/issues/new?template=plugin-review.yml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTab {
    Catalog,
    Docs,
    CreatePlugin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetPlatform {
    All,
    Linux,
    MacOs,
    Windows,
}

struct AppDoc {
    id: &'static str,
    name: &'static str,
    glyph: &'static str,
    architecture: &'static str,
    description: &'static str,
    linux_path: &'static str,
    macos_path: &'static str,
    windows_path: &'static str,
    steps: &'static [&'static str],
}

const APP_DOCS: &[AppDoc] = &[
    AppDoc {
        id: "photocraft",
        name: "PhotoCraft",
        glyph: "P",
        architecture: "WASM Filters (ABI v1) & 3D LUTs",
        description: "Executes sandboxed WebAssembly pixel-manipulation modules (compiled to wasm32-unknown-unknown) with zero OS syscalls, alongside 3D color cubes (.cube).",
        linux_path: "~/.config/photocraft/plugins/",
        macos_path: "~/Library/Application Support/PhotoCraft/Plugins/",
        windows_path: "%APPDATA%\\PhotoCraft\\Plugins\\",
        steps: &[
            "Download the .wasm filter binary (or .cube 3D LUT file).",
            "Copy the file into your platform's plugins directory listed above.",
            "Launch PhotoCraft or select Filter ▸ Rescan Plug-in Directory.",
            "Access the filter under the Filter menu in its registered category.",
            "For 3D LUTs: Apply via Image ▸ Adjustments ▸ Apply 3D LUT... or use LUT Studio.",
        ],
    },
    AppDoc {
        id: "effectcraft",
        name: "EffectCraft",
        glyph: "E",
        architecture: "WASM Render Filters (API v1) & ScriptUI Panels (.jsx)",
        description: "High-performance video render filters in WASM/WAT, paired with dockable ExtendScript panels for keyframing, captions, and audio silence removal.",
        linux_path: "WASM: ~/.config/effectcraft/plugins/  |  ScriptUI: ~/.config/effectcraft/Scripts/ScriptUI Panels/",
        macos_path: "WASM: ~/Library/Application Support/EffectCraft/Plug-ins/  |  ScriptUI: Scripts/ScriptUI Panels/",
        windows_path: "WASM: %APPDATA%\\EffectCraft\\Plug-ins\\  |  ScriptUI: Scripts\\ScriptUI Panels\\",
        steps: &[
            "Render Plugins (.wasm, .wat): Place in plugins folder or load via Effect ▸ Load Effect Plug-in...",
            "ScriptUI Panels (.jsx): Copy into Scripts/ScriptUI Panels/ and restart EffectCraft.",
            "Open your panel from the Window menu (e.g. Window ▸ PivotCraft.jsx, QuietCraft.jsx, EaseCraft.jsx).",
            "Dock the panel anywhere in your workspace alongside Effect Controls and Timeline.",
        ],
    },
    AppDoc {
        id: "vectorcraft",
        name: "VectorCraft",
        glyph: "V",
        architecture: "Object Filters (ABI v1) & SVG Templates (.zip)",
        description: "Procedural path transforms running in a gas-limited wasmi sandbox, plus editable SVG template packages.",
        linux_path: "Plugins: ~/.config/vectorcraft/plugins/  |  Templates: ~/.config/vectorcraft/templates/",
        macos_path: "Plugins: ~/Library/Application Support/VectorCraft/Plugins/  |  Templates: Templates/",
        windows_path: "Plugins: %APPDATA%\\VectorCraft\\Plugins\\  |  Templates: Templates\\",
        steps: &[
            "WASM Filters: Copy to plugins folder; open Window ▸ Plug-in Manager or Object ▸ Live Filters.",
            "SVG Templates: Extract archive into your templates folder and select File ▸ New From Template...",
        ],
    },
    AppDoc {
        id: "filmcraft",
        name: "FilmCraft",
        glyph: "F",
        architecture: "Timeline Effect Presets (JSON v1) & 3D LUTs (.cube)",
        description: "Declarative parameter and effect chains (filmcraft.effect-presets v1) and 17/33/65-point 3D Look-Up Tables.",
        linux_path: "LUTs: ~/.config/filmcraft/LUTs/Creative/",
        macos_path: "LUTs: ~/Library/Application Support/FilmCraft/LUTs/Creative/",
        windows_path: "LUTs: %APPDATA%\\FilmCraft\\LUTs\\Creative\\",
        steps: &[
            "GUI Import: Right-click Effects panel Presets bin ▸ Import Presets... and select the JSON file.",
            "CLI Import: Run `filmcraft-cli --project project.fcproj exec presets.import '{\"path\":\"presets.json\"}'`",
            "Creative LUTs: Save .cube files to your LUTs folder or select Lumetri Color ▸ Creative ▸ Look LUT.",
        ],
    },
    AppDoc {
        id: "soundcraft",
        name: "SoundCraft",
        glyph: "S",
        architecture: "CLAP (.clap), VST3 (.vst3), Audio Units & DSP Presets",
        description: "Host-compliant digital audio workstation loading open CLAP, VST3, and macOS Audio Units with sample-accurate automation.",
        linux_path: "CLAP: ~/.clap/ or /usr/lib/clap/  |  VST3: ~/.vst3/ or /usr/lib/vst3/",
        macos_path: "CLAP: ~/Library/Audio/Plug-Ins/CLAP/  |  VST3: Plug-Ins/VST3/  |  AU: Components/",
        windows_path: "CLAP: %COMMONPROGRAMFILES%\\CLAP\\  |  VST3: %COMMONPROGRAMFILES%\\VST3\\",
        steps: &[
            "Place third-party audio plugins in your platform's standard audio plugin directory.",
            "In SoundCraft, open Preferences ▸ Audio Plug-ins ▸ Rescan All.",
            "Insert the plugin on any channel strip track.",
            "Channel Strip Presets: Load via the track insert header menu (Load Channel Strip Preset...).",
        ],
    },
    AppDoc {
        id: "pdfcraft",
        name: "PdfCraft",
        glyph: "A",
        architecture: "Acrobat JavaScript (ISO 32000 in Boa Sandbox)",
        description: "Interactive forms, field calculations, and automation scripts running in a sandboxed pure-Rust JavaScript engine.",
        linux_path: "Tools ▸ JavaScript ▸ Document JavaScripts",
        macos_path: "Tools ▸ JavaScript ▸ Document JavaScripts",
        windows_path: "Tools ▸ JavaScript ▸ Document JavaScripts",
        steps: &[
            "Form Calculations: Open PDF in PdfCraft, switch to Form Edit Mode.",
            "Right-click field ▸ Properties ▸ Calculate ▸ Custom Script and paste calculation logic.",
            "Document JavaScripts: Register global scripts via Tools ▸ JavaScript ▸ Document JavaScripts.",
        ],
    },
    AppDoc {
        id: "designcraft",
        name: "DesignCraft",
        glyph: "D",
        architecture: "IDML Publication Templates & Typographic Styles",
        description: "Multi-page layout templates, master spreads, and typography style packages.",
        linux_path: "Documents/DesignCraft/Templates/",
        macos_path: "Documents/DesignCraft/Templates/",
        windows_path: "Documents\\DesignCraft\\Templates\\",
        steps: &[
            "Save downloaded template archives to Documents/DesignCraft/Templates/.",
            "Open directly with File ▸ New From Template... or File ▸ Open...",
            "Load paragraph & character styles from the Paragraph Styles panel flyout menu.",
        ],
    },
    AppDoc {
        id: "lightcraft",
        name: "LightCraft",
        glyph: "L",
        architecture: "RAW Develop Presets (.xmp) & 3D LUT Profiles (.cube)",
        description: "Non-destructive RAW photo development parameter curves and color grading profiles.",
        linux_path: "~/.config/lightcraft/presets/",
        macos_path: "~/Library/Application Support/LightCraft/Presets/",
        windows_path: "%APPDATA%\\LightCraft\\Presets\\",
        steps: &[
            "Open LightCraft's Develop module.",
            "Click Presets panel (+) ▸ Import Presets... and select .xmp or preset bundle.",
            "Load 3D LUT profiles in Color Grading ▸ Profiles dropdown.",
        ],
    },
    AppDoc {
        id: "cadcraft",
        name: "CADCraft",
        glyph: "C",
        architecture: "Command Scripts (.cadscr) & Geometric Macros",
        description: "Parametric drafting automation, batch coordinate drawing, and procedural geometry macros.",
        linux_path: "~/.config/cadcraft/scripts/",
        macos_path: "~/Library/Application Support/CADCraft/Scripts/",
        windows_path: "%APPDATA%\\CADCraft\\Scripts\\",
        steps: &[
            "In CADCraft command line prompt, run: SCRIPT \"path/to/script.cadscr\"",
            "Run interactive macros via Tools ▸ Run Macro / Script...",
            "Bind macros to workspace shortcut keys or custom toolbar buttons.",
        ],
    },
];

#[derive(Debug, Clone, Deserialize, Default)]
struct Fork {
    #[serde(default)]
    name: String,
    #[serde(default)]
    app: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    url: String,
    #[serde(default, rename = "sourceUrl")]
    source_url: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Catalog {
    #[serde(default)]
    plugins: Vec<Plugin>,
    #[serde(default)]
    forks: Vec<Fork>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Plugin {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    app: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    compatibility: String,
    #[serde(default)]
    abi: Option<u32>,
    #[serde(default)]
    source: String,
    #[serde(default, rename = "sourceUrl")]
    source_url: Option<String>,
    #[serde(default, rename = "downloadUrl")]
    download_url: Option<String>,
    #[serde(default)]
    artifact: Option<String>,
    #[serde(default, rename = "releaseAsset")]
    release_asset: Option<String>,
}

const PHOTOCRAFT_FILTER_SNIPPET: &str = r##"// PhotoCraft WASM Filter (ABI v1)
#![no_std]

static MANIFEST: &[u8] = br#"{
  "id": "org.photocraft.sample.invert",
  "name": "Sample Invert",
  "version": "1.0.0",
  "kind": "filter",
  "params": {
    "intensity": { "type": "int", "min": 0, "max": 100, "default": 100 }
  }
}"#;

#[no_mangle]
pub extern "C" fn pc_abi_version() -> i32 { 1 }

#[no_mangle]
pub extern "C" fn pc_manifest() -> i64 {
    ((MANIFEST.len() as i64) << 32) | (MANIFEST.as_ptr() as u32 as i64)
}

static mut HEAP_NEXT: usize = 65536;

#[no_mangle]
pub extern "C" fn pc_alloc(size: i32) -> i32 {
    let start = unsafe { (HEAP_NEXT + 7) & !7usize };
    unsafe { HEAP_NEXT = start + size as usize; }
    start as i32
}

#[no_mangle]
pub extern "C" fn pc_process(
    in_ptr: i32,
    out_ptr: i32,
    width: i32,
    height: i32,
    _params_ptr: i32,
    _params_len: i32,
) -> i32 {
    let total_bytes = (width as usize) * (height as usize) * 4;
    let src = unsafe { core::slice::from_raw_parts(in_ptr as *const u8, total_bytes) };
    let dst = unsafe { core::slice::from_raw_parts_mut(out_ptr as *mut u8, total_bytes) };
    for i in (0..total_bytes).step_by(4) {
        dst[i] = 255 - src[i];         // R
        dst[i + 1] = 255 - src[i + 1]; // G
        dst[i + 2] = 255 - src[i + 2]; // B
        dst[i + 3] = src[i + 3];       // A
    }
    0
}
"##;

const EFFECTCRAFT_RENDER_SNIPPET: &str = r##"// EffectCraft Render Effect (API v1)
pub const MANIFEST: &str = r#"{
  "api": 1,
  "id": "org.effectcraft.plugins.sample",
  "name": "Sample Tint",
  "category": "Color Correction",
  "version": "1.0.0",
  "params": [
    {"id": "gain", "name": "Gain", "type": "slider", "default": 1.0, "min": 0.0, "max": 4.0}
  ]
}"#;

#[no_mangle]
pub extern "C" fn ec_api_version() -> i32 { 1 }

#[no_mangle]
pub extern "C" fn ec_manifest_ptr() -> i32 { MANIFEST.as_ptr() as usize as i32 }

#[no_mangle]
pub extern "C" fn ec_manifest_len() -> i32 { MANIFEST.len() as i32 }

static mut RENDER_BUF: [u8; 1024 * 1024 * 16] = [0; 1024 * 1024 * 16];

#[no_mangle]
pub extern "C" fn ec_alloc(bytes: i32) -> i32 {
    unsafe { RENDER_BUF.as_mut_ptr() as usize as i32 }
}
"##;

const SCRIPTUI_PANEL_SNIPPET: &str = r#"// EffectCraft ScriptUI Dockable Panel (.jsx ExtendScript)
(function (thisObj) {
    function buildUI(thisObj) {
        var win = (thisObj instanceof Panel)
            ? thisObj
            : new Window("palette", "MyTool", undefined, { resizeable: true });

        win.orientation = "column";
        win.alignChildren = ["fill", "top"];
        win.margins = 16;
        win.spacing = 10;

        var title = win.add("statictext", undefined, "My Tool Automation");
        var btnAction = win.add("button", undefined, "Execute Script Action");

        btnAction.onClick = function () {
            app.beginUndoGroup("My Tool Execution");
            var comp = app.project.activeItem;
            if (comp && comp instanceof CompItem) {
                for (var i = 1; i <= comp.selectedLayers.length; i++) {
                    var layer = comp.selectedLayers[i - 1];
                    // Custom keyframe, transform, or effect logic here
                }
            }
            app.endUndoGroup();
        };

        win.layout.layout(true);
        return win;
    }
    buildUI(thisObj);
})(this);
"#;

const VECTORCRAFT_FILTER_SNIPPET: &str = r##"// VectorCraft Path Transform Filter (ABI v1, wasmi sandbox)
pub const MANIFEST: &str = r#"{
  "abi": 1,
  "id": "org.vectorcraft.plugins.jitter",
  "name": "Path Jitter",
  "version": "1.0.0",
  "capabilities": ["path_transform", "bezier_subdivide"]
}"#;

#[no_mangle]
pub extern "C" fn vc_abi_version() -> i32 { 1 }

#[no_mangle]
pub extern "C" fn vc_manifest_ptr() -> i32 { MANIFEST.as_ptr() as usize as i32 }

#[no_mangle]
pub extern "C" fn vc_manifest_len() -> i32 { MANIFEST.len() as i32 }
"##;

const PUBLISHING_CHECKLIST_SNIPPET: &str = r#"# ArtCraft Storefront Submission Checklist
# 1. Place plugin source under: plugins/<app>/<plugin-name>/
# 2. Add catalog entry in catalog.json:
{
  "id": "my-plugin",
  "name": "My Plugin",
  "app": "photocraft",
  "kind": "WASM Filter (ABI v1)",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Short summary of what your plugin does.",
  "tags": ["filter", "effects", "creative"],
  "artifact": "photocraft-my-plugin-v1.0.0.wasm"
}

# 3. Test compilation and validate:
bash build-all.sh
python3 scripts/validate_catalog.py --check-artifacts
python3 -m unittest discover tests

# 4. Submit Pull Request to https://github.com/akkk09/artcraft-store
"#;

const SOUNDCRAFT_DSP_SNIPPET: &str = r#"{
  "format": "soundcraft.channel-strip",
  "version": "1.0.0",
  "name": "Vocal Dynamics & Air Preset",
  "author": "ArtCraft Audio Community",
  "plugins": [
    {
      "format": "clap",
      "id": "org.clap.high-pass-filter",
      "params": { "cutoff_hz": 80.0, "slope_db_oct": 18 }
    },
    {
      "format": "vst3",
      "id": "org.vst3.opto-compressor",
      "params": { "threshold_db": -18.5, "ratio": 4.0, "attack_ms": 15.0, "release_ms": 120.0 }
    },
    {
      "format": "clap",
      "id": "org.clap.air-shelf-eq",
      "params": { "frequency_hz": 12000.0, "gain_db": 3.5, "q": 0.7 }
    }
  ]
}"#;

const FILMCRAFT_PRESET_SNIPPET: &str = r#"{
  "format": "filmcraft.effect-presets",
  "version": "1.0.0",
  "id": "org.filmcraft.presets.cinematic-glow",
  "name": "Cinematic Anamorphic Bloom",
  "effects": [
    {
      "name": "Luma Key",
      "params": { "threshold": 0.82, "softness": 0.15 }
    },
    {
      "name": "Directional Blur",
      "params": { "direction": 90.0, "length": 45.0 }
    },
    {
      "name": "Chromatic Aberration",
      "params": { "red_shift": 1.02, "blue_shift": 0.98 }
    },
    {
      "name": "Composite Blend",
      "params": { "mode": "Screen", "opacity": 0.65 }
    }
  ]
}"#;

const PDFCRAFT_JS_SNIPPET: &str = r#"// PdfCraft ISO 32000 Form Calculator & Validator
(function () {
    var subtotalField = this.getField("Subtotal");
    var taxRateField = this.getField("TaxRate");
    var totalField = this.getField("GrandTotal");

    if (subtotalField && taxRateField && totalField) {
        var subtotal = Number(subtotalField.value) || 0.0;
        var taxRate = Number(taxRateField.value) || 0.0;
        var taxAmount = subtotal * (taxRate / 100.0);
        var grandTotal = subtotal + taxAmount;

        totalField.value = util.printf("$%.2f", grandTotal);
    }
})();
"#;

const CADCRAFT_SCRIPT_SNIPPET: &str = r#"; CADCraft Parametric Flange Drawing Script (.cadscr)
; Initialize Drawing Units and Layers
-UNITS 2 4 1 2 0 N
-LAYER M Geometry C 7 Geometry 
-LAYER M Centerlines C 1 Centerlines 

; Draw Outer Circular Flange
-LAYER S Geometry 
CIRCLE 100,100 90
CIRCLE 100,100 45

; Draw Centerlines
-LAYER S Centerlines 
LINE 5,100 195,100 
LINE 100,5 100,195 

; Draw 4x Mounting Bolt Holes at 45-degree Quadrants
-LAYER S Geometry 
CIRCLE 148.5,148.5 7.5
CIRCLE 51.5,148.5 7.5
CIRCLE 51.5,51.5 7.5
CIRCLE 148.5,51.5 7.5

ZOOM EXTENTS
"#;

struct StoreApp {
    plugins: Vec<Plugin>,
    forks: Vec<Fork>,
    search: String,
    active_app: String,
    active_tab: ActiveTab,
    docs_app: String,
    docs_platform: TargetPlatform,
    create_track: usize,
    notification: Option<String>,
}

impl StoreApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(19, 19, 19);
        visuals.window_fill = Color32::from_rgb(24, 24, 24);
        visuals.extreme_bg_color = Color32::from_rgb(14, 14, 14);
        visuals.faint_bg_color = Color32::from_rgb(32, 32, 32);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(26, 26, 26);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(214, 214, 214));
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 35, 35);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(227, 227, 227));
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(54, 54, 54);
        visuals.widgets.active.bg_fill = Color32::from_rgb(222, 222, 222);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(17, 17, 17));
        visuals.selection.bg_fill = Color32::from_rgb(222, 222, 222);
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(17, 17, 17));
        cc.egui_ctx.set_visuals(visuals);

        // Embed the validated catalog so the first render does not wait on a network request.
        let catalog = serde_json::from_str::<Catalog>(include_str!("../../catalog.json"))
            .unwrap_or_default();

        Self {
            plugins: catalog.plugins,
            forks: catalog.forks,
            search: String::new(),
            active_app: "all".to_owned(),
            active_tab: ActiveTab::Catalog,
            docs_app: "all".to_owned(),
            docs_platform: TargetPlatform::All,
            create_track: 0,
            notification: None,
        }
    }

    fn tab_button(ui: &mut egui::Ui, selected: bool, text: &str) -> egui::Response {
        let text_color = if selected {
            Color32::from_rgb(17, 17, 17)
        } else {
            Color32::from_rgb(195, 195, 195)
        };
        ui.selectable_label(selected, RichText::new(text).size(12.5).strong().color(text_color))
    }

    fn pill_button(ui: &mut egui::Ui, selected: bool, text: &str) -> egui::Response {
        let text_color = if selected {
            Color32::from_rgb(17, 17, 17)
        } else {
            Color32::from_rgb(205, 205, 205)
        };
        ui.selectable_label(selected, RichText::new(text).size(12.0).color(text_color))
    }

    fn app_name(app: &str) -> &'static str {
        match app {
            "photocraft" => "PhotoCraft",
            "effectcraft" => "EffectCraft",
            "vectorcraft" => "VectorCraft",
            "filmcraft" => "FilmCraft",
            "soundcraft" => "SoundCraft",
            "pdfcraft" => "PdfCraft",
            "designcraft" => "DesignCraft",
            "lightcraft" => "LightCraft",
            "cadcraft" => "CADCraft",
            _ => "ArtCraft",
        }
    }

    fn source_url(plugin: &Plugin) -> String {
        plugin.source_url.clone().unwrap_or_else(|| {
            if plugin.source.is_empty() {
                REPO_URL.to_owned()
            } else {
                format!("{REPO_URL}/tree/main/{}", plugin.source)
            }
        })
    }

    fn download_url(plugin: &Plugin) -> Option<String> {
        if let Some(url) = &plugin.download_url {
            return Some(url.clone());
        }
        plugin
            .release_asset
            .as_ref()
            .or(plugin.artifact.as_ref())
            .map(|name| format!("{BASE_URL}downloads/{name}"))
    }

    fn card(ui: &mut egui::Ui, plugin: &Plugin, switch_to_docs: &mut Option<String>) {
        Frame::new()
            .fill(Color32::from_rgb(26, 26, 26))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    let initial = plugin.name.chars().next().unwrap_or('A').to_uppercase().to_string();
                    Frame::new()
                        .fill(Color32::from_rgb(219, 219, 219))
                        .inner_margin(egui::Margin::same(9))
                        .corner_radius(egui::CornerRadius::same(8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(initial).color(Color32::from_rgb(23, 23, 23)).strong().size(20.0));
                        });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(Self::app_name(&plugin.app)).color(Color32::from_rgb(219, 219, 219)).size(12.0).strong());
                        ui.label(RichText::new(if plugin.kind.is_empty() { "Extension" } else { &plugin.kind }).color(Color32::from_rgb(162, 162, 162)).size(11.0));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(if plugin.version.is_empty() { "Community" } else { &plugin.version })
                            .color(Color32::from_rgb(162, 162, 162)).size(11.0));
                    });
                });

                ui.add_space(13.0);
                ui.label(RichText::new(&plugin.name).font(FontId::proportional(21.0)).strong().color(Color32::from_rgb(243, 243, 243)));
                ui.add_space(5.0);
                ui.label(RichText::new(&plugin.description).size(13.0).color(Color32::from_rgb(180, 180, 180)));
                ui.add_space(10.0);

                ui.horizontal_wrapped(|ui| {
                    if !plugin.compatibility.is_empty() {
                        ui.label(RichText::new(&plugin.compatibility).size(10.5).color(Color32::from_rgb(219, 219, 219)));
                    } else if let Some(abi) = plugin.abi {
                        ui.label(RichText::new(format!("ABI v{abi}")).size(10.5).color(Color32::from_rgb(219, 219, 219)));
                    }
                    for tag in plugin.tags.iter().take(4) {
                        ui.label(RichText::new(format!("· {tag}")).size(10.5).color(Color32::from_rgb(146, 146, 146)));
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(7.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(if plugin.author.is_empty() { "Community" } else { &plugin.author })
                        .size(11.0).color(Color32::from_rgb(146, 146, 146)));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(url) = Self::download_url(plugin) {
                            ui.hyperlink_to(RichText::new("Download ↓").strong().color(Color32::from_rgb(219, 219, 219)), url);
                        } else {
                            ui.label(RichText::new("No download").size(11.0).color(Color32::from_rgb(146, 146, 146)));
                        }
                        ui.add_space(8.0);
                        ui.hyperlink_to(RichText::new("Review ↗").size(11.0), REVIEW_URL);
                        ui.add_space(8.0);
                        ui.hyperlink_to(RichText::new("Source ↗").size(11.0), Self::source_url(plugin));
                        ui.add_space(8.0);
                        if ui.button(RichText::new("Install Guide").size(11.0)).clicked() {
                            *switch_to_docs = Some(plugin.app.clone());
                        }
                    });
                });
            });
    }

    fn fork_card(ui: &mut egui::Ui, fork: &Fork) {
        Frame::new()
            .fill(Color32::from_rgb(26, 26, 26))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    let initial = fork.name.chars().next().unwrap_or('F').to_uppercase().to_string();
                    Frame::new()
                        .fill(Color32::from_rgb(219, 219, 219))
                        .inner_margin(egui::Margin::same(9))
                        .corner_radius(egui::CornerRadius::same(8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(initial).color(Color32::from_rgb(23, 23, 23)).strong().size(20.0));
                        });
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(Self::app_name(&fork.app)).color(Color32::from_rgb(219, 219, 219)).size(12.0).strong());
                        ui.label(RichText::new("Community Fork / Extended Runtime").color(Color32::from_rgb(162, 162, 162)).size(11.0));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(if fork.version.is_empty() { "Latest" } else { &fork.version })
                            .color(Color32::from_rgb(162, 162, 162)).size(11.0));
                    });
                });

                ui.add_space(13.0);
                ui.label(RichText::new(&fork.name).font(FontId::proportional(21.0)).strong().color(Color32::from_rgb(243, 243, 243)));
                ui.add_space(5.0);
                ui.label(RichText::new(&fork.description).size(13.0).color(Color32::from_rgb(180, 180, 180)));
                ui.add_space(10.0);

                ui.horizontal_wrapped(|ui| {
                    for tag in fork.tags.iter().take(5) {
                        ui.label(RichText::new(format!("· {tag}")).size(10.5).color(Color32::from_rgb(146, 146, 146)));
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(7.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("by {}", if fork.author.is_empty() { "Community" } else { &fork.author }))
                        .size(11.0).color(Color32::from_rgb(146, 146, 146)));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if !fork.url.is_empty() {
                            ui.hyperlink_to(RichText::new("Latest Release ↗").strong().color(Color32::from_rgb(219, 219, 219)), &fork.url);
                        }
                        if let Some(src) = &fork.source_url {
                            ui.add_space(8.0);
                            ui.hyperlink_to(RichText::new("Source Fork ↗").size(11.0), src);
                        }
                    });
                });
            });
    }

    fn render_docs(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("● USER GUIDES   ·   SDK ARCHITECTURE   ·   ALL 09 APPS")
            .size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
        ui.add_space(14.0);
        ui.label(RichText::new("Extension Architecture\n& Installation Guides.").size(42.0).strong().color(Color32::from_rgb(243, 243, 243)));
        ui.add_space(8.0);
        ui.label(RichText::new("Step-by-step setup guides, directory paths, and developer architecture references for all ArtCraft creative applications.")
            .size(15.0).color(Color32::from_rgb(180, 180, 180)));
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            if ui.button(RichText::new("🛠️  Build your own extension: Open Plug-in Creation SDK Guide ↗").strong().size(12.5)).clicked() {
                self.active_tab = ActiveTab::CreatePlugin;
            }
        });
        ui.add_space(14.0);

        if let Some(msg) = &self.notification {
            Frame::new()
                .fill(Color32::from_rgb(30, 60, 30))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(60, 120, 60)))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("✓ {msg}")).color(Color32::from_rgb(180, 240, 180)).size(12.0).strong());
                });
            ui.add_space(12.0);
        }

        // App Filters
        ui.label(RichText::new("SELECT APPLICATION:").size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            let all_selected = self.docs_app == "all";
            if Self::pill_button(ui, all_selected, "All apps").clicked() {
                self.docs_app = "all".to_owned();
            }
            for doc in APP_DOCS {
                let selected = self.docs_app == doc.id;
                if Self::pill_button(ui, selected, doc.name).clicked() {
                    self.docs_app = doc.id.to_owned();
                }
            }
        });

        ui.add_space(12.0);

        // Platform Filter
        ui.label(RichText::new("OPERATING SYSTEM:").size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            for (platform, label) in [
                (TargetPlatform::All, "All Systems"),
                (TargetPlatform::Linux, "Linux"),
                (TargetPlatform::MacOs, "macOS"),
                (TargetPlatform::Windows, "Windows"),
            ] {
                let selected = self.docs_platform == platform;
                if Self::pill_button(ui, selected, label).clicked() {
                    self.docs_platform = platform;
                }
            }
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(16.0);

        // Render matching docs
        let matching_docs: Vec<&AppDoc> = APP_DOCS
            .iter()
            .filter(|doc| self.docs_app == "all" || doc.id == self.docs_app)
            .collect();

        let mut switch_to_track: Option<usize> = None;

        for doc in matching_docs {
            let doc_name = doc.name;
            let doc_glyph = doc.glyph;
            let doc_arch = doc.architecture;
            let doc_desc = doc.description;
            let doc_linux = doc.linux_path;
            let doc_macos = doc.macos_path;
            let doc_win = doc.windows_path;
            let doc_steps = doc.steps;
            let platform = self.docs_platform;

            let mut copied_path: Option<(&'static str, &'static str)> = None;

            Frame::new()
                .fill(Color32::from_rgb(26, 26, 26))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    // App Card Header
                    ui.horizontal(|ui| {
                        Frame::new()
                            .fill(Color32::from_rgb(219, 219, 219))
                            .inner_margin(egui::Margin::same(9))
                            .corner_radius(egui::CornerRadius::same(8))
                            .show(ui, |ui| {
                                ui.label(RichText::new(doc_glyph).color(Color32::from_rgb(23, 23, 23)).strong().size(22.0));
                            });
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(doc_name).font(FontId::proportional(22.0)).strong().color(Color32::from_rgb(243, 243, 243)));
                            ui.label(RichText::new(doc_arch).size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        });
                    });

                    ui.add_space(10.0);
                    ui.label(RichText::new(doc_desc).size(13.5).color(Color32::from_rgb(180, 180, 180)));
                    ui.add_space(14.0);

                    // Path boxes
                    if platform == TargetPlatform::All || platform == TargetPlatform::Linux {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Linux:").strong().size(12.0).color(Color32::from_rgb(219, 219, 219)));
                            ui.label(RichText::new(doc_linux).monospace().size(12.0).color(Color32::from_rgb(235, 235, 235)));
                            if ui.button(RichText::new("Copy").size(11.0)).clicked() {
                                copied_path = Some(("Linux", doc_linux));
                            }
                        });
                        ui.add_space(4.0);
                    }
                    if platform == TargetPlatform::All || platform == TargetPlatform::MacOs {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("macOS:").strong().size(12.0).color(Color32::from_rgb(219, 219, 219)));
                            ui.label(RichText::new(doc_macos).monospace().size(12.0).color(Color32::from_rgb(235, 235, 235)));
                            if ui.button(RichText::new("Copy").size(11.0)).clicked() {
                                copied_path = Some(("macOS", doc_macos));
                            }
                        });
                        ui.add_space(4.0);
                    }
                    if platform == TargetPlatform::All || platform == TargetPlatform::Windows {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Windows:").strong().size(12.0).color(Color32::from_rgb(219, 219, 219)));
                            ui.label(RichText::new(doc_win).monospace().size(12.0).color(Color32::from_rgb(235, 235, 235)));
                            if ui.button(RichText::new("Copy").size(11.0)).clicked() {
                                copied_path = Some(("Windows", doc_win));
                            }
                        });
                        ui.add_space(4.0);
                    }

                    ui.add_space(10.0);
                    ui.label(RichText::new("ACTIVATION PROCEDURE:").size(11.0).strong().color(Color32::from_rgb(219, 219, 219)));
                    ui.add_space(4.0);
                    for (idx, step) in doc_steps.iter().enumerate() {
                        ui.label(RichText::new(format!("{}. {}", idx + 1, step)).size(13.0).color(Color32::from_rgb(200, 200, 200)));
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("DEVELOPER SPECIFICATIONS:").size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let track_target = match doc.id {
                                "photocraft" => 0,
                                "effectcraft" => 1,
                                "vectorcraft" => 3,
                                "filmcraft" => 5,
                                "soundcraft" => 4,
                                "pdfcraft" => 6,
                                "designcraft" => 7,
                                "lightcraft" => 0,
                                "cadcraft" => 7,
                                _ => 0,
                            };
                            if ui.button(RichText::new("🛠️  View Developer SDK Guide & Code Template ↗").size(11.5).strong()).clicked() {
                                switch_to_track = Some(track_target);
                            }
                        });
                    });
                });

            if let Some((plat, path)) = copied_path {
                ui.ctx().copy_text(path.to_owned());
                self.notification = Some(format!("Copied {plat} path for {doc_name} to clipboard"));
            }

            if let Some(track) = switch_to_track {
                self.active_tab = ActiveTab::CreatePlugin;
                self.create_track = track;
            }

            ui.add_space(16.0);
        }

        // Community Forks Section in Docs
        let relevant_docs_forks: Vec<&Fork> = self.forks
            .iter()
            .filter(|f| self.docs_app == "all" || f.app == self.docs_app)
            .collect();
        if !relevant_docs_forks.is_empty() {
            ui.add_space(10.0);
            Frame::new()
                .fill(Color32::from_rgb(26, 26, 26))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                .inner_margin(20.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("COMMUNITY FORKS & EXTENDED RUNTIMES").size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
                    ui.add_space(6.0);
                    ui.label(RichText::new("Modified engine runtimes and external plugin hosts.").size(20.0).strong().color(Color32::from_rgb(243, 243, 243)));
                    ui.add_space(6.0);
                    ui.label(RichText::new("Some community projects fork the core ArtCraft repositories to extend the engine itself with external plugin standards (e.g., OpenFX) or proprietary project file parsers. These run as standalone modified builds rather than standard user plugins.")
                        .size(13.5).color(Color32::from_rgb(180, 180, 180)));
                    ui.add_space(14.0);
                    for fork in relevant_docs_forks {
                        Self::fork_card(ui, fork);
                        ui.add_space(8.0);
                    }
                });
            ui.add_space(14.0);
        }

        // Developer Section
        ui.add_space(14.0);
        Frame::new()
            .fill(Color32::from_rgb(24, 24, 24))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
            .inner_margin(20.0)
            .show(ui, |ui| {
                ui.label(RichText::new("DEVELOPER & SDK ARCHITECTURE").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                ui.add_space(8.0);
                ui.label(RichText::new("Build native WebAssembly filters in Rust using wasm32-unknown-unknown:").size(13.0).color(Color32::from_rgb(180, 180, 180)));
                ui.add_space(4.0);
                ui.label(RichText::new("cargo build --target wasm32-unknown-unknown --release").monospace().size(12.5).color(Color32::from_rgb(220, 220, 220)));
                ui.add_space(10.0);
                ui.label(RichText::new("Run the local store catalog audit and unit test suite:").size(13.0).color(Color32::from_rgb(180, 180, 180)));
                ui.add_space(4.0);
                ui.label(RichText::new("python3 -m unittest tests/test_catalog.py\npython3 scripts/validate_catalog.py --check-artifacts").monospace().size(12.0).color(Color32::from_rgb(220, 220, 220)));
            });

        ui.add_space(20.0);
    }

    fn render_create_plugin(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("● DEVELOPER SDK   ·   WASM ABI   ·   EXTENDSCRIPT   ·   ALL 09 APPS")
            .size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
        ui.add_space(14.0);
        ui.label(RichText::new("Plug-in Creation Guide\n& Developer SDK.").size(42.0).strong().color(Color32::from_rgb(243, 243, 243)));
        ui.add_space(8.0);
        ui.label(RichText::new("Step-by-step developer tutorials, ABI specifications, code templates, and store publishing guidelines.")
            .size(15.0).color(Color32::from_rgb(180, 180, 180)));
        ui.add_space(18.0);

        if let Some(msg) = &self.notification {
            Frame::new()
                .fill(Color32::from_rgb(30, 60, 30))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(60, 120, 60)))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("✓ {msg}")).color(Color32::from_rgb(180, 240, 180)).size(12.0).strong());
                });
            ui.add_space(12.0);
        }

        // Sub-topic Track Pills
        ui.label(RichText::new("DEVELOPER GUIDE TRACK:").size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            let tracks = [
                (0, "PhotoCraft & LightCraft (WASM ABI v1 & LUTs)"),
                (1, "EffectCraft Video Effects (Render API v1)"),
                (2, "EffectCraft ScriptUI Panels (.jsx)"),
                (3, "VectorCraft Path Filters (wasmi) & Templates"),
                (4, "SoundCraft Audio Plugins (CLAP / VST3 / AU)"),
                (5, "FilmCraft Timeline Presets & 3D LUTs"),
                (6, "PdfCraft ISO 32000 JavaScript Automation"),
                (7, "DesignCraft & CADCraft Scripts"),
                (8, "Sandboxing, Fuel Limits & Security Rules"),
                (9, "Storefront Publishing Checklist"),
            ];
            for (idx, label) in tracks {
                let selected = self.create_track == idx;
                if Self::pill_button(ui, selected, label).clicked() {
                    self.create_track = idx;
                }
            }
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(16.0);

        let mut copied_snippet: Option<(&'static str, &'static str)> = None;

        match self.create_track {
            0 => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("PHOTOCRAFT & LIGHTCRAFT: WASM FILTER ABI v1 & 3D COLOR LUTS").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("PhotoCraft and LightCraft execute sandboxed WebAssembly binaries compiled to wasm32-unknown-unknown. Plugins operate directly on linear memory RGBA pixel buffers with zero operating system syscalls.")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Memory Layout & Buffer Format:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• 32-bit interleaved RGBA [R, G, B, A] with 8-bits per channel in range 0..=255\n• Unmultiplied alpha with row-major memory order\n• Buffer byte length: width * height * 4 bytes\n• Linear memory page growth managed via core::arch::wasm32::memory_grow").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Mandatory Exported C ABI Functions:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• pc_abi_version() -> i32: Must return integer 1\n• pc_manifest() -> i64: Returns packed pointer & length ((len << 32) | ptr) to JSON manifest\n• pc_alloc(size: i32) -> i32: Heap allocator for linear memory buffers\n• pc_process(in_ptr, out_ptr, width, height, params_ptr, params_len) -> i32: Applies filter; returns 0 on success").monospace().size(11.5).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Supported Parameter Manifest Types:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• \"int\": Integer slider with min, max, default\n• \"float\": Floating-point slider with min, max, default, decimals\n• \"color\": 4-component RGBA vector [r, g, b, a] in range 0.0..1.0\n• \"checkbox\": Boolean toggle\n• \"popup\": Dropdown list with options array").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("3D Look-Up Table (.cube) Specification:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• Supported grid sizes: 17x17x17, 33x33x33, and 65x65x65 points\n• Standard Adobe/DaVinci .cube format with LUT_3D_SIZE, DOMAIN_MIN, DOMAIN_MAX\n• Real-time trilinear interpolation evaluated on GPU/CPU pixel shaders").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Sample Rust Filter Implementation (no_std):").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button(RichText::new("Copy Code").size(11.0)).clicked() {
                                    copied_snippet = Some(("PhotoCraft Filter", PHOTOCRAFT_FILTER_SNIPPET));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        Frame::new().fill(Color32::from_rgb(13, 13, 13)).inner_margin(12.0).corner_radius(egui::CornerRadius::same(6)).show(ui, |ui| {
                            ui.label(RichText::new(PHOTOCRAFT_FILTER_SNIPPET).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                        });
                        ui.add_space(12.0);
                        ui.label(RichText::new("Compilation & Audit Command:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("cargo build --target wasm32-unknown-unknown --release").monospace().size(12.0).color(Color32::from_rgb(235, 235, 235)));
                    });
            }
            1 => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("EFFECTCRAFT: REAL-TIME VIDEO RENDER EFFECTS (WASM API v1)").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("EffectCraft render effects execute once per composition frame in the video playback and rendering pipeline. Built in Rust or WAT targeting WebAssembly, plugins operate on high-dynamic-range floating-point buffers.")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("HDR Pixel Format & Buffer Layout:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• 32-bit float RGBA [f32; 4] per pixel in range 0.0..1.0+ without clipping\n• High-precision color preservation for blurs, glows, light sweeps, and caustics\n• Reusable 8-byte aligned scratch allocation to avoid frame allocations").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Exported API v1 Functions:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• ec_api_version() -> i32: Returns 1\n• ec_manifest_ptr() -> i32 & ec_manifest_len() -> i32: Manifest access\n• ec_alloc(bytes: i32) -> i32: Memory arena allocator\n• ec_render(width: i32, height: i32, time: f64, params_ptr: i32, in_pixels: i32, out_pixels: i32) -> i32").monospace().size(11.5).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Temporal Keyframing & Procedural Synthesis:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• The time parameter provides fractional playback seconds (e.g. 2.50s at frame 60 in 24fps)\n• Procedural effects compute instantaneous phase: let phase = (time * speed) % cycle_period\n• Shader primitives: Signed Distance Fields (SDF), Sobel normal vectors, chromatic fringe dispersion").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Sample WASM Render Effect Implementation:").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button(RichText::new("Copy Code").size(11.0)).clicked() {
                                    copied_snippet = Some(("EffectCraft Render Effect", EFFECTCRAFT_RENDER_SNIPPET));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        Frame::new().fill(Color32::from_rgb(13, 13, 13)).inner_margin(12.0).corner_radius(egui::CornerRadius::same(6)).show(ui, |ui| {
                            ui.label(RichText::new(EFFECTCRAFT_RENDER_SNIPPET).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                        });
                    });
            }
            2 => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("EFFECTCRAFT: SCRIPTUI DOCKABLE PANELS (.jsx EXTENDSCRIPT)").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("ExtendScript panels automate keyframing, timeline manipulation, layer batching, and custom UI controls. Panels dock seamlessly inside EffectCraft's workspace alongside Timeline and Effect Controls.")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Host Object Model Hierarchy:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• app.project: Active project entity\n• app.project.activeItem: Current active CompItem\n• comp.selectedLayers: Array of selected AVLayer / ShapeLayer objects\n• layer.property(\"Transform\").property(\"Position\").setValueAtTime(time, value)").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Mandatory Undo Safety Invariant:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("All script actions that modify project state must be wrapped with app.beginUndoGroup(\"Action Name\") and app.endUndoGroup(). This ensures single-step Ctrl+Z / Cmd+Z undo for creators.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Sample ScriptUI Dockable Panel (.jsx):").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button(RichText::new("Copy Code").size(11.0)).clicked() {
                                    copied_snippet = Some(("ScriptUI Panel", SCRIPTUI_PANEL_SNIPPET));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        Frame::new().fill(Color32::from_rgb(13, 13, 13)).inner_margin(12.0).corner_radius(egui::CornerRadius::same(6)).show(ui, |ui| {
                            ui.label(RichText::new(SCRIPTUI_PANEL_SNIPPET).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                        });
                    });
            }
            3 => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("VECTORCRAFT: OBJECT FILTERS (wasmi ENGINE) & SVG TEMPLATES").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("VectorCraft executes procedural geometry filters in an embedded wasmi WebAssembly interpreter with instruction gas/fuel metering. It also supports editable SVG template archives.")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Geometry Pipeline & Manifest Capabilities:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• Input: Bezier paths composed of anchor points, cubic in/out tangent control handles, and closed contour markers\n• Capabilities array: [\"path_transform\", \"bezier_subdivide\", \"color_recolor\"]\n• Gas budget: Plugins receive a deterministic fuel allocation to protect against infinite loops").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("SVG Template Distribution Packages:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• Standard .zip archives placed in templates/ directory\n• Contains template.svg with structured layer IDs, 512x512 thumbnail.png, and metadata.json\n• Loaded via File ▸ New From Template... in VectorCraft").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("VectorCraft Path Filter Implementation:").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button(RichText::new("Copy Code").size(11.0)).clicked() {
                                    copied_snippet = Some(("VectorCraft Filter", VECTORCRAFT_FILTER_SNIPPET));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        Frame::new().fill(Color32::from_rgb(13, 13, 13)).inner_margin(12.0).corner_radius(egui::CornerRadius::same(6)).show(ui, |ui| {
                            ui.label(RichText::new(VECTORCRAFT_FILTER_SNIPPET).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                        });
                    });
            }
            4 => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("SOUNDCRAFT: AUDIO PLUGINS (CLAP / VST3 / AU) & DSP PRESETS").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("SoundCraft is a digital audio workstation loading open-standard audio plugins with sample-accurate automation. Creators can build audio effects, instruments, and channel strip DSP chains.")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Supported Plugin Standards:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• CLAP (.clap): Modern open C-ABI audio plugin standard with polyphonic modulation and thread pool support\n• VST3 (.vst3): Cross-platform industry standard plugin architecture\n• Audio Units (.component): Native macOS low-latency audio plugins").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Real-Time DSP Safety Invariants:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• Zero heap allocation (malloc / Box::new) on the audio rendering thread\n• Lock-free synchronization between UI and DSP audio callback\n• Sample-accurate block processing with smoothed parameter ramps").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Sample Channel Strip DSP Preset (.json):").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button(RichText::new("Copy Code").size(11.0)).clicked() {
                                    copied_snippet = Some(("SoundCraft Preset", SOUNDCRAFT_DSP_SNIPPET));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        Frame::new().fill(Color32::from_rgb(13, 13, 13)).inner_margin(12.0).corner_radius(egui::CornerRadius::same(6)).show(ui, |ui| {
                            ui.label(RichText::new(SOUNDCRAFT_DSP_SNIPPET).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                        });
                    });
            }
            5 => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("FILMCRAFT: TIMELINE EFFECT PRESETS & 3D COLOR LUTS").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("FilmCraft enables modular timeline workflows through declarative JSON effect preset chains and creative 3D Look-Up Tables (.cube).")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Declarative Effect Chain Schema:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• filmcraft.effect-presets v1 specification\n• Multi-stage chaining: Luma Keying, Directional Blur, Chromatic Aberration, Composite Blends\n• Temporal easing: Linear, Bezier, Hold, and Exponential interpolation curves").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("CLI Automation & Import:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("filmcraft-cli --project project.fcproj exec presets.import '{\"path\":\"presets.json\"}'").monospace().size(12.0).color(Color32::from_rgb(235, 235, 235)));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Sample FilmCraft Preset Chain (.json):").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button(RichText::new("Copy Code").size(11.0)).clicked() {
                                    copied_snippet = Some(("FilmCraft Preset", FILMCRAFT_PRESET_SNIPPET));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        Frame::new().fill(Color32::from_rgb(13, 13, 13)).inner_margin(12.0).corner_radius(egui::CornerRadius::same(6)).show(ui, |ui| {
                            ui.label(RichText::new(FILMCRAFT_PRESET_SNIPPET).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                        });
                    });
            }
            6 => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("PDFCRAFT: FORM CALCULATIONS & DOCUMENT AUTOMATION (ISO 32000)").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("PdfCraft runs Acrobat JavaScript (ISO 32000 standard) inside a sandboxed Boa engine. Scripts automate document calculation, validate input keystrokes, and manage interactive fields.")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("Event Model & Form DOM:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• event.value: Target field value during calculation or formatting\n• event.change: Key character during keystroke validation events\n• this.getField(\"FieldName\"): Access document field objects and values\n• util.printf(\"$%.2f\", total): Formats numbers into currency or decimals").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Sample ISO 32000 Form Calculator Script:").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button(RichText::new("Copy Code").size(11.0)).clicked() {
                                    copied_snippet = Some(("PdfCraft Script", PDFCRAFT_JS_SNIPPET));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        Frame::new().fill(Color32::from_rgb(13, 13, 13)).inner_margin(12.0).corner_radius(egui::CornerRadius::same(6)).show(ui, |ui| {
                            ui.label(RichText::new(PDFCRAFT_JS_SNIPPET).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                        });
                    });
            }
            7 => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("DESIGNCRAFT LAYOUTS & CADCRAFT PARAMETRIC MACROS").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("DesignCraft supports publication templates (IDML packages and typographic styles), while CADCraft executes batch command scripts (.cadscr) for precision 2D/3D drafting.")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("CADCraft Scripting Syntax (.cadscr):").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("• Headless execution: One command and argument pair per line\n• Drawing primitives: LINE, CIRCLE, ARC, PLINE, EXTRUDE\n• Layer assignment: -LAYER M <name> C <color> <name> and -LAYER S <name>\n• Viewport controls: ZOOM EXTENTS, REGEN").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Sample CADCraft Flange Script (.cadscr):").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button(RichText::new("Copy Code").size(11.0)).clicked() {
                                    copied_snippet = Some(("CADCraft Script", CADCRAFT_SCRIPT_SNIPPET));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        Frame::new().fill(Color32::from_rgb(13, 13, 13)).inner_margin(12.0).corner_radius(egui::CornerRadius::same(6)).show(ui, |ui| {
                            ui.label(RichText::new(CADCRAFT_SCRIPT_SNIPPET).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                        });
                    });
            }
            8 => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("SANDBOXING, FUEL LIMITS & SECURITY INVARIANTS").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("To maintain crash resilience, host security, and cross-platform portability across Linux, macOS, and Windows, all plugins must satisfy four core invariants:")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("1. Zero OS Syscalls:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("WASM modules must not import POSIX syscalls, unconstrained filesystem I/O, or raw TCP/UDP sockets. All state transfer is mediated via linear memory buffers.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("2. Linear Memory Bounds:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("Modules operate within isolated 64MB linear memory arenas by default. Memory expansions via memory_grow must check bounds explicitly to avoid OOM panics.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("3. Cross-Platform Determinism:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("Image and video filters must produce bit-identical results on x86_64 and ARM64. Avoid host-dependent floating-point behavior or unseeded randomness.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("4. Instruction Fuel Metering:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("VectorCraft's wasmi engine and EffectCraft interpreters decrement a CPU fuel meter per instruction. Infinite loops terminate gracefully without freezing the host UI.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                    });
            }
            _ => {
                Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("STOREFRONT PUBLISHING & VERIFICATION CHECKLIST").size(14.0).strong().color(Color32::from_rgb(243, 243, 243)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("Follow this 5-step checklist to submit your extension to the official ArtCraft Store catalog:")
                            .size(13.0).color(Color32::from_rgb(180, 180, 180)));
                        ui.add_space(12.0);
                        ui.label(RichText::new("1. Repository Structure:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("Place plugin source code under plugins/<app>/<plugin-name>/ or link an open GitHub repository.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("2. Catalog Registration:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("Add your item to catalog.json with id, name, app, kind, version, author, description, tags, and artifact filename.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("3. Compile Release Artifacts:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("Run bash build-all.sh to compile WASM binaries and copy release packages to dist/.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("4. Run Automated Audits:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("Execute python3 scripts/validate_catalog.py --check-artifacts and python3 -m unittest discover tests.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(8.0);
                        ui.label(RichText::new("5. Submit Pull Request:").strong().color(Color32::from_rgb(219, 219, 219)));
                        ui.label(RichText::new("Open a Pull Request on GitHub or file an issue using the Plugin Review template.").size(12.0).color(Color32::from_rgb(200, 200, 200)));
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Submission Checklist & Commands:").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button(RichText::new("Copy Checklist").size(11.0)).clicked() {
                                    copied_snippet = Some(("Publishing Checklist", PUBLISHING_CHECKLIST_SNIPPET));
                                }
                            });
                        });
                        ui.add_space(6.0);
                        Frame::new().fill(Color32::from_rgb(13, 13, 13)).inner_margin(12.0).corner_radius(egui::CornerRadius::same(6)).show(ui, |ui| {
                            ui.label(RichText::new(PUBLISHING_CHECKLIST_SNIPPET).monospace().size(11.0).color(Color32::from_rgb(220, 220, 220)));
                        });
                    });
            }
        }

        if let Some((label, snippet)) = copied_snippet {
            ui.ctx().copy_text(snippet.to_owned());
            self.notification = Some(format!("Copied {label} snippet to clipboard"));
        }

        ui.add_space(20.0);
    }
}

impl eframe::App for StoreApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("topbar")
            .frame(Frame::new().fill(Color32::from_rgb(16, 16, 16)).inner_margin(egui::Margin::symmetric(24, 15)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("A").size(18.0).strong().color(Color32::from_rgb(219, 219, 219)));
                    ui.label(RichText::new("ARTCRAFT").size(14.0).strong().color(Color32::from_rgb(242, 242, 242)));
                    ui.label(RichText::new("/ STORE").size(12.0).color(Color32::from_rgb(142, 142, 142)));
                    ui.add_space(18.0);

                    // Modular Tab Switcher
                    let is_catalog = self.active_tab == ActiveTab::Catalog;
                    if Self::tab_button(ui, is_catalog, "Catalog").clicked() {
                        self.active_tab = ActiveTab::Catalog;
                    }
                    ui.add_space(4.0);
                    let is_docs = self.active_tab == ActiveTab::Docs;
                    if Self::tab_button(ui, is_docs, "Documentation").clicked() {
                        self.active_tab = ActiveTab::Docs;
                    }
                    ui.add_space(4.0);
                    let is_create = self.active_tab == ActiveTab::CreatePlugin;
                    if Self::tab_button(ui, is_create, "Create a Plug-in").clicked() {
                        self.active_tab = ActiveTab::CreatePlugin;
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.hyperlink_to(RichText::new("GitHub ↗").size(12.0).color(Color32::from_rgb(219, 219, 219)), REPO_URL);
                        ui.add_space(12.0);
                        ui.hyperlink_to(RichText::new("LUT Studio ↗").size(12.0), format!("{BASE_URL}lut-studio/"));
                        ui.add_space(12.0);
                        ui.hyperlink_to(RichText::new("Catalog JSON").size(12.0), format!("{BASE_URL}catalog.json"));
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(Frame::new().fill(Color32::from_rgb(19, 19, 19)).inner_margin(egui::Margin::symmetric(24, 22)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.set_max_width(1120.0);

                    match self.active_tab {
                        ActiveTab::Docs => {
                            self.render_docs(ui);
                        }
                        ActiveTab::CreatePlugin => {
                            self.render_create_plugin(ui);
                        }
                        ActiveTab::Catalog => {
                            ui.label(RichText::new("● COMMUNITY BUILT   ·   OPEN SOURCE   ·   MADE FOR CREATORS")
                                .size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.add_space(15.0);
                            ui.label(RichText::new("Tools for your\ncreative flow.").size(48.0).strong().color(Color32::from_rgb(243, 243, 243)));
                            ui.add_space(10.0);
                            ui.label(RichText::new("Small, focused extensions for ArtCraft creative applications, built by the community.")
                                .size(15.0).color(Color32::from_rgb(180, 180, 180)));
                            ui.add_space(14.0);
                            ui.horizontal(|ui| {
                                if ui.button(RichText::new("📖  Installation & Architecture Docs").size(12.5).strong()).clicked() {
                                    self.active_tab = ActiveTab::Docs;
                                }
                                ui.add_space(8.0);
                                if ui.button(RichText::new("🛠️  Create a Plug-in").size(12.5).strong()).clicked() {
                                    self.active_tab = ActiveTab::CreatePlugin;
                                }
                            });
                            ui.add_space(18.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(format!("{:02}", self.plugins.len())).size(20.0).strong().color(Color32::from_rgb(219, 219, 219)));
                                ui.label(RichText::new("catalog items").size(12.0).color(Color32::from_rgb(146, 146, 146)));
                                ui.add_space(18.0);
                                ui.label(RichText::new("09").size(20.0).strong().color(Color32::from_rgb(219, 219, 219)));
                                ui.label(RichText::new("creative apps").size(12.0).color(Color32::from_rgb(146, 146, 146)));
                                ui.add_space(18.0);
                                ui.label(RichText::new("MIT").size(20.0).strong().color(Color32::from_rgb(219, 219, 219)));
                                ui.label(RichText::new("open project").size(12.0).color(Color32::from_rgb(146, 146, 146)));
                            });

                            ui.add_space(28.0);
                            ui.separator();
                            ui.add_space(20.0);
                            ui.label(RichText::new("BROWSE THE CATALOG").size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.add_space(6.0);
                            ui.label(RichText::new("Find your next tool.").size(27.0).strong().color(Color32::from_rgb(243, 243, 243)));
                            ui.add_space(12.0);
                            ui.add_sized(
                                Vec2::new(ui.available_width().min(540.0), 36.0),
                                egui::TextEdit::singleline(&mut self.search).hint_text("Search name, feature, tag…"),
                            );
                            ui.add_space(10.0);
                            ui.horizontal_wrapped(|ui| {
                                for (key, label) in [
                                    ("all", "All apps"),
                                    ("photocraft", "PhotoCraft"),
                                    ("effectcraft", "EffectCraft"),
                                    ("vectorcraft", "VectorCraft"),
                                    ("filmcraft", "FilmCraft"),
                                    ("soundcraft", "SoundCraft"),
                                    ("pdfcraft", "PdfCraft"),
                                    ("designcraft", "DesignCraft"),
                                    ("lightcraft", "LightCraft"),
                                    ("cadcraft", "CADCraft"),
                                ] {
                                    let selected = self.active_app == key;
                                    if Self::pill_button(ui, selected, label).clicked() {
                                        self.active_app = key.to_owned();
                                    }
                                }
                            });
                            ui.add_space(14.0);

                            let query = self.search.trim().to_lowercase();
                            let shown: Vec<Plugin> = self.plugins.iter().filter(|plugin| {
                                let app_matches = self.active_app == "all" || plugin.app == self.active_app;
                                let searchable = format!(
                                    "{} {} {} {} {} {}",
                                    plugin.name,
                                    plugin.description,
                                    plugin.author,
                                    plugin.kind,
                                    plugin.app,
                                    plugin.tags.join(" ")
                                ).to_lowercase();
                                app_matches && searchable.contains(&query)
                            }).cloned().collect();

                            let mut switch_to_docs: Option<String> = None;

                            if shown.is_empty() {
                                Frame::new().fill(Color32::from_rgb(26, 26, 26))
                                    .inner_margin(20.0).show(ui, |ui| {
                                        ui.label(RichText::new("No matching items").strong().size(17.0));
                                        ui.label(RichText::new("Try another search or select a different app.").color(Color32::from_rgb(162, 162, 162)));
                                    });
                            } else {
                                let columns = if ui.available_width() >= 760.0 { 2 } else { 1 };
                                ui.columns(columns, |cols| {
                                    for (index, plugin) in shown.iter().enumerate() {
                                        let column = index % columns;
                                        cols[column].add_space(6.0);
                                        Self::card(&mut cols[column], plugin, &mut switch_to_docs);
                                        cols[column].add_space(10.0);
                                    }
                                });
                            }

                            if let Some(target_app) = switch_to_docs {
                                self.active_tab = ActiveTab::Docs;
                                self.docs_app = target_app;
                            }

                            let relevant_forks: Vec<&Fork> = self.forks
                                .iter()
                                .filter(|f| self.active_app == "all" || f.app == self.active_app)
                                .collect();
                            if !relevant_forks.is_empty() {
                                ui.add_space(26.0);
                                ui.separator();
                                ui.add_space(20.0);
                                ui.label(RichText::new("COMMUNITY FORKS & EXTENDED RUNTIMES").size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
                                ui.add_space(6.0);
                                ui.label(RichText::new("Modified builds & custom runtimes.").size(27.0).strong().color(Color32::from_rgb(243, 243, 243)));
                                ui.add_space(6.0);
                                ui.label(RichText::new("Community projects that fork the core application repository to add experimental engine features, external plugin standards (such as OpenFX), or project file interchange.")
                                    .size(14.0).color(Color32::from_rgb(180, 180, 180)));
                                ui.add_space(14.0);
                                for fork in relevant_forks {
                                    Self::fork_card(ui, fork);
                                    ui.add_space(10.0);
                                }
                            }

                            ui.add_space(20.0);
                            Frame::new().fill(Color32::from_rgb(26, 26, 26))
                                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                                .inner_margin(14.0).show(ui, |ui| {
                                    ui.label(RichText::new("Before you install").strong().color(Color32::from_rgb(219, 219, 219)));
                                    ui.label(RichText::new("Check each listing’s source, file format, and compatibility. Downloads are served from the store or the project that maintains the item.")
                                        .size(12.0).color(Color32::from_rgb(180, 180, 180)));
                                });
                        }
                    }

                    ui.add_space(26.0);
                    ui.separator();
                    ui.add_space(12.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("ARTCRAFT STORE  ·  COMMUNITY PROJECT").size(10.5).color(Color32::from_rgb(146, 146, 146)));
                        ui.add_space(10.0);
                        ui.hyperlink_to(RichText::new("Source code ↗").size(11.0), REPO_URL);
                        ui.add_space(10.0);
                        ui.hyperlink_to(RichText::new("Catalog API ↗").size(11.0), format!("{BASE_URL}api/v1/manifest.json"));
                    });
                });
            });
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(async {
        let canvas = web_sys::window()
            .expect("window unavailable")
            .document()
            .expect("document unavailable")
            .get_element_by_id("the_canvas_id")
            .expect("canvas element missing")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("element is not a canvas");
        let options = eframe::WebOptions::default();
        eframe::WebRunner::new()
            .start(
                canvas,
                options,
                Box::new(|cc| Ok(Box::new(StoreApp::new(cc)))),
            )
            .await
            .expect("failed to start egui ArtCraft Store");
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("Build this storefront for wasm32-unknown-unknown with Trunk.");
}
