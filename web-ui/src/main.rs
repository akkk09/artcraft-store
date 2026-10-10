use eframe::egui::{self, Align, Color32, FontId, Frame, Layout, RichText, Stroke, Vec2};
use serde::Deserialize;

const BASE_URL: &str = "https://akkk09.github.io/artcraft-store/";
const REPO_URL: &str = "https://github.com/akkk09/artcraft-store";
const REVIEW_URL: &str = "https://github.com/akkk09/artcraft-store/issues/new?template=plugin-review.yml";

#[derive(Debug, Clone, Deserialize, Default)]
struct Catalog {
    #[serde(default)]
    plugins: Vec<Plugin>,
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
    search: String,
    active_app: String,
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
        visuals.widgets.active.bg_fill = Color32::from_rgb(219, 219, 219);
        visuals.selection.bg_fill = Color32::from_rgb(219, 219, 219);
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(21, 21, 21));
        cc.egui_ctx.set_visuals(visuals);

        // Embed the validated catalog so the first render does not wait on a network request.
        let catalog = serde_json::from_str::<Catalog>(include_str!("../../catalog.json"))
            .unwrap_or_default();

        Self {
            plugins: catalog.plugins,
            search: String::new(),
            active_app: "all".to_owned(),
        }
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

    fn card(ui: &mut egui::Ui, plugin: &Plugin) {
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
                    });
                });
            });
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
                    ui.label(RichText::new("● COMMUNITY BUILT   ·   OPEN SOURCE   ·   MADE FOR CREATORS")
                        .size(10.5).strong().color(Color32::from_rgb(219, 219, 219)));
                    ui.add_space(15.0);
                    ui.label(RichText::new("Tools for your\ncreative flow.").size(48.0).strong().color(Color32::from_rgb(243, 243, 243)));
                    ui.add_space(10.0);
                    ui.label(RichText::new("Small, focused extensions for ArtCraft creative applications, built by the community.")
                        .size(15.0).color(Color32::from_rgb(180, 180, 180)));
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
                            if ui.selectable_label(selected, RichText::new(label).size(12.0)).clicked() {
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
                                Self::card(&mut cols[column], plugin);
                                cols[column].add_space(10.0);
                            }
                        });
                    }

                    ui.add_space(20.0);
                    Frame::new().fill(Color32::from_rgb(26, 26, 26))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 55)))
                        .inner_margin(14.0).show(ui, |ui| {
                            ui.label(RichText::new("Before you install").strong().color(Color32::from_rgb(219, 219, 219)));
                            ui.label(RichText::new("Check each listing’s source, file format, and compatibility. Downloads are served from the store or the project that maintains the item.")
                                .size(12.0).color(Color32::from_rgb(180, 180, 180)));
                        });

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
