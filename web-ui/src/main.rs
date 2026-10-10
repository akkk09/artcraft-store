use eframe::egui::{self, Align, Color32, FontId, Frame, Layout, RichText, Stroke, Vec2};
use serde::Deserialize;

const BASE_URL: &str = "https://akkk09.github.io/artcraft-store/";
const REPO_URL: &str = "https://github.com/akkk09/artcraft-store";
const REVIEW_URL: &str = "https://github.com/akkk09/artcraft-store/issues/new?template=plugin-review.yml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTab {
    Catalog,
    Docs,
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

struct StoreApp {
    plugins: Vec<Plugin>,
    forks: Vec<Fork>,
    search: String,
    active_app: String,
    active_tab: ActiveTab,
    docs_app: String,
    docs_platform: TargetPlatform,
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
                });

            if let Some((plat, path)) = copied_path {
                ui.ctx().copy_text(path.to_owned());
                self.notification = Some(format!("Copied {plat} path for {doc_name} to clipboard"));
            }

            ui.add_space(16.0);
        }

        // Community Forks Section in Docs
        if !self.forks.is_empty() && (self.docs_app == "all" || self.docs_app == "effectcraft") {
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
                    for fork in &self.forks {
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

                            if !self.forks.is_empty() {
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
                                for fork in &self.forks {
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
