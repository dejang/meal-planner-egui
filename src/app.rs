use core::str;
use std::{
    future::Future,
    sync::{Arc, Mutex},
};

use base64::prelude::*;
use rfd::FileHandle;
use uuid::Uuid;

use crate::{
    icon,
    meal_planner::MealPlanner,
    planner::Planner,
    recipe_editor::Editor,
    recipe_gallery::RecipeGallery,
    shopping_list::ShoppingList,
    theme::{on_forest, palette, paper_window_frame, paper_window_header, surface_frame},
    typography::icons::ICON_LEAF,
    util::DEFAULT_PADDING,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::recipe_import::{supported_sites, RecipeImporter};

#[cfg(not(target_arch = "wasm32"))]
const DRAFT_IMPORT_CONFLICT: &str = "Close the current new recipe before importing another one";

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: std::future::Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn execute<F: std::future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MealPlannerApp {
    #[serde(skip)]
    planner: Planner,
    #[serde(skip)]
    pub editor_visible: bool,
    #[serde(skip)]
    editor_recipe_id: Option<Uuid>,
    #[serde(skip)]
    pub shopping_list_visible: bool,
    #[serde(skip)]
    pub settings_window_visible: bool,
    #[serde(skip)]
    welcome_dismissed: bool,
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    import_recipe_visible: bool,
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    import_recipe_url: String,
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    import_recipe_error: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    recipe_importer: RecipeImporter,
    #[serde(skip)]
    import_data: Arc<Mutex<(String, Vec<u8>)>>,
    #[serde(skip)]
    recipe_gallery: RecipeGallery,
    #[serde(skip)]
    shopping_list: ShoppingList,
    meal_planner: MealPlanner,
}

impl Default for MealPlannerApp {
    fn default() -> Self {
        Self {
            planner: Planner::default(),
            editor_visible: false,
            editor_recipe_id: None,
            shopping_list_visible: false,
            settings_window_visible: false,
            welcome_dismissed: false,
            #[cfg(not(target_arch = "wasm32"))]
            import_recipe_visible: false,
            #[cfg(not(target_arch = "wasm32"))]
            import_recipe_url: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            import_recipe_error: None,
            #[cfg(not(target_arch = "wasm32"))]
            recipe_importer: RecipeImporter::default(),
            shopping_list: ShoppingList::default(),
            import_data: Arc::new(Mutex::new((String::new(), vec![]))),
            meal_planner: MealPlanner::default(),
            recipe_gallery: RecipeGallery::default(),
        }
    }
}

impl MealPlannerApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn show_recipe_import(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let mut close = false;
        egui::Window::new("Import Recipe from URL")
            .open(&mut self.import_recipe_visible)
            .title_bar(false)
            .collapsible(false)
            .frame(paper_window_frame(ui.style()))
            .resizable(true)
            .default_width(560.0)
            .min_width(340.0)
            .vscroll(true)
            .show(&ctx, |ui| {
                close = paper_window_header(ui, "Import recipe");
                ui.label(
                    egui::RichText::new("Bring a recipe into your collection, ready to edit.")
                        .color(palette::MUTED),
                );
                ui.add_space(16.0);
                egui::Frame::new()
                    .fill(palette::SURFACE)
                    .inner_margin(20)
                    .corner_radius(10)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            egui::RichText::new("RECIPE URL")
                                .size(11.0)
                                .strong()
                                .color(palette::MUTED),
                        );
                        ui.add_space(6.0);
                        let loading = self.recipe_importer.is_loading();
                        let response = ui
                            .add_enabled_ui(!loading, |ui| {
                                ui.add_sized(
                                    [ui.available_width(), 40.0],
                                    egui::TextEdit::singleline(&mut self.import_recipe_url)
                                        .font(egui::FontId::proportional(16.0))
                                        .margin(egui::vec2(12.0, 10.0))
                                        .hint_text("https://…")
                                        .desired_width(f32::INFINITY),
                                )
                            })
                            .inner;
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(
                                "Paste a recipe link from a supported website below.",
                            )
                            .size(14.0)
                            .color(palette::MUTED),
                        );
                        ui.add_space(16.0);
                        let can_submit = !loading && !self.import_recipe_url.trim().is_empty();
                        let submit = ui
                            .allocate_ui_with_layout(
                                egui::vec2(ui.available_width(), 38.0),
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let response = ui.add_enabled(
                                        can_submit,
                                        egui::Button::new(
                                            egui::RichText::new(if loading {
                                                "Importing…"
                                            } else {
                                                "Import recipe"
                                            })
                                            .color(palette::SURFACE),
                                        )
                                        .fill(palette::FOREST)
                                        .min_size(egui::vec2(140.0, 38.0)),
                                    );
                                    if loading {
                                        ui.spinner();
                                    }
                                    response
                                },
                            )
                            .inner;

                        if can_submit
                            && (submit.clicked()
                                || (response.lost_focus()
                                    && ui.input(|input| input.key_pressed(egui::Key::Enter))))
                        {
                            self.import_recipe_error =
                                if self.meal_planner.can_create_draft_recipe() {
                                    self.recipe_importer
                                        .start(&ctx, self.import_recipe_url.trim())
                                        .err()
                                } else {
                                    Some(DRAFT_IMPORT_CONFLICT.to_string())
                                };
                        }
                        if let Some(error) = &self.import_recipe_error {
                            ui.add_space(10.0);
                            ui.colored_label(ui.visuals().error_fg_color, error);
                        }
                    });
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("SUPPORTED WEBSITES")
                        .size(11.0)
                        .strong()
                        .color(palette::MUTED),
                );
                for (name, domains) in supported_sites() {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(name).strong().color(palette::FOREST));
                    ui.horizontal_wrapped(|ui| {
                        for domain in domains {
                            ui.hyperlink_to(
                                egui::RichText::new(*domain).size(14.0),
                                format!("https://{domain}"),
                            );
                        }
                    });
                }
                ui.add_space(8.0);
            });
        if close {
            self.import_recipe_visible = false;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    #[cfg(target_arch = "wasm32")]
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let json = BASE64_STANDARD
            .decode(include_bytes!("../state.json"))
            .unwrap();
        let default_state = serde_json::from_slice(json.as_slice()).unwrap();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let previous_state: MealPlannerApp =
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            if previous_state.meal_planner.is_daily_plan_empty()
                && previous_state.meal_planner.get_recipes().len() == 0
            {
                return default_state;
            }
            return previous_state;
        }

        Default::default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn export_data(&mut self) {
        use std::io::Write;
        let mut file = std::fs::File::create("state.json").unwrap();
        let content = serde_json::to_string(&self).unwrap();
        file.write_all(BASE64_STANDARD.encode(content).as_bytes())
            .expect("Exporting data failed");
    }

    #[cfg(target_arch = "wasm32")]
    fn export_data(&mut self) {
        use web_sys::wasm_bindgen::JsCast;

        let content = serde_json::to_string(&self).unwrap();
        let win = web_sys::window().unwrap();
        let doc = win.document().unwrap();

        let link = doc.create_element("a").unwrap();
        let _ = link.set_attribute(
            "href",
            &format!("data:text/plain,{}", BASE64_STANDARD.encode(content)),
        );
        let _ = link.set_attribute("download", "backup.json");
        let link: web_sys::HtmlAnchorElement =
            web_sys::HtmlAnchorElement::unchecked_from_js(link.into());
        link.click();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn import_data(&mut self, task: impl Future<Output = Option<FileHandle>> + Send + 'static) {
        let file_path = self.import_data.clone();

        execute(async move {
            let file = task.await;
            if let Some(file) = file {
                let mut file_path = file_path.lock().unwrap();
                file_path.0 = file.path().to_str().unwrap().to_string();
            }
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn import_data(&mut self, task: impl Future<Output = Option<FileHandle>> + 'static) {
        let file_path = self.import_data.clone();

        execute(async move {
            let file = task.await;
            if let Some(file) = file {
                let file_bytes = file.read().await;
                file_path.lock().unwrap().1 = file_bytes;
            }
        });
    }
}

impl eframe::App for MealPlannerApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.meal_planner.poll_analysis();

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(result) = self.recipe_importer.take_result() {
            match result {
                Ok(imported) => {
                    if let Some(draft) = self.meal_planner.create_draft_recipe() {
                        imported.apply_to(draft, self.import_recipe_url.trim());
                        let recipe_id = draft.id;
                        self.editor_recipe_id = Some(recipe_id);
                        self.editor_visible = true;
                        self.import_recipe_visible = false;
                        self.import_recipe_error = None;
                        self.meal_planner
                            .lookup_nutrients_for_recipe_id(&ctx, recipe_id);
                    } else {
                        self.import_recipe_error = Some(DRAFT_IMPORT_CONFLICT.to_string());
                    }
                }
                Err(error) => self.import_recipe_error = Some(error),
            }
        }

        {
            if let Ok(mut lock) = self.import_data.clone().try_lock() {
                if !lock.0.is_empty() {
                    let content = std::fs::read_to_string(&lock.0).expect("Unable to read");
                    let decoded = BASE64_STANDARD.decode(content).unwrap();
                    if self
                        .meal_planner
                        .load_json(std::str::from_utf8(decoded.as_slice()).unwrap())
                    {
                        println!("Successful");
                    }

                    lock.0 = String::new();
                }

                if !lock.1.is_empty() {
                    let decoded = BASE64_STANDARD.decode(&lock.1).unwrap();
                    self.meal_planner
                        .load_json(std::str::from_utf8(decoded.as_slice()).unwrap());
                    lock.1 = vec![];
                }
            }
        }

        // Fixed top menu bar
        egui::Panel::top("main_menu_bar")
            .frame(
                egui::Frame::new()
                    .fill(palette::FOREST)
                    .inner_margin(egui::Margin::symmetric(18, 10)),
            )
            .show(ui, |ui| {
                on_forest(ui);
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.label(icon(ICON_LEAF).size(23.0).color(palette::CITRUS));
                    ui.label(
                        egui::RichText::new("Meal planner")
                            .size(19.0)
                            .strong()
                            .color(palette::SURFACE),
                    );
                    ui.add_space(20.0);
                    // NOTE: no File->Quit on web pages!
                    let is_web = cfg!(target_arch = "wasm32");
                    if !is_web {
                        ui.menu_button("File", |ui| {
                            if ui.button("Quit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                        ui.add_space(16.0);
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    let create_recipe = {
                        let mut create_recipe = false;
                        let chevron_size = egui::vec2(12.0, 12.0);
                        let chevron =
                            egui::Atom::custom(ui.id().with("new_recipe_chevron"), chevron_size);
                        let (response, _) = egui::containers::menu::MenuButton::new((
                            "New Recipe",
                            chevron,
                        ))
                        .ui(ui, |ui| {
                            if ui.button("Create").clicked() {
                                create_recipe = true;
                                ui.close();
                            }

                            if ui.button("Import from URL").clicked() {
                                self.import_recipe_visible = true;
                                self.import_recipe_error = None;
                                ui.close();
                            }
                        });
                        // Paint the chevron around the button's center, independent of font baselines.
                        let center = egui::pos2(
                            response.rect.right()
                                - ui.spacing().button_padding.x
                                - chevron_size.x * 0.5,
                            response.rect.center().y,
                        );
                        ui.painter().add(egui::Shape::line(
                            vec![
                                center + egui::vec2(-4.0, -2.0),
                                center + egui::vec2(0.0, 2.0),
                                center + egui::vec2(4.0, -2.0),
                            ],
                            egui::Stroke::new(1.5, ui.style().interact(&response).fg_stroke.color),
                        ));
                        create_recipe
                    };
                    #[cfg(target_arch = "wasm32")]
                    let create_recipe = ui.button("Create Recipe").clicked();
                    if create_recipe {
                        if let Some(draft) = self.meal_planner.create_draft_recipe() {
                            self.editor_recipe_id = Some(draft.id);
                            self.editor_visible = true;
                        }
                    }

                    if ui.button("Import Data").clicked() {
                        let task = rfd::AsyncFileDialog::new().pick_file();
                        self.import_data(task);
                    }

                    if ui.button("Export Data").clicked() {
                        self.export_data();
                    }

                    if ui.button("Shopping List").clicked() {
                        self.shopping_list_visible = true;
                    }

                    if ui.button("Settings").clicked() {
                        self.settings_window_visible = true;
                    }
                });
            });

        // Resizable bottom panel for the planner
        let screen_height = ctx.content_rect().height();
        let planner_edit = egui::Panel::bottom("planner_panel")
            .resizable(true)
            .show_separator_line(false)
            .min_size(screen_height * 0.15)
            .default_size(screen_height * 0.5)
            .frame(
                egui::Frame::new()
                    .fill(palette::PAPER)
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show(ui, |ui| {
                surface_frame()
                    .inner_margin(10.0)
                    .show(ui, |ui| self.planner.ui(ui, &mut self.meal_planner))
                    .inner
            })
            .inner;

        // Central panel for recipe browser
        let edit_recipe = egui::CentralPanel::default()
            .show(ui, |ui| self.recipe_gallery.ui(ui, &mut self.meal_planner))
            .inner;

        if let Some(recipe_id) = edit_recipe.or(planner_edit) {
            self.editor_recipe_id = Some(recipe_id);
            self.editor_visible = true;
        }

        // Recipe URL import window
        #[cfg(not(target_arch = "wasm32"))]
        self.show_recipe_import(ui);

        // Recipe Editor window
        let editor_size = egui::vec2(
            (ctx.content_rect().width() - 40.0).clamp(280.0, 980.0),
            (ctx.content_rect().height() - 40.0).clamp(280.0, 860.0),
        );
        let mut editor_result = None;
        let response = egui::Window::new("Recipe Editor")
            .id(egui::Id::new("recipe_editor_cookbook"))
            .open(&mut self.editor_visible)
            .resizable(true)
            .collapsible(false)
            .title_bar(false)
            .frame(paper_window_frame(ui.style()))
            .default_size(editor_size)
            .max_size(
                (ctx.content_rect().size() - egui::vec2(16.0, 16.0)).max(egui::vec2(280.0, 280.0)),
            )
            .min_size(egui::vec2(280.0, 280.0))
            .default_pos(ctx.content_rect().center() - editor_size * 0.5)
            .show(&ctx, |ui| {
                if let Some(id) = self.editor_recipe_id {
                    editor_result = Some(
                        Editor::new().ui(ui, self.meal_planner.get_recipe_by_id_mut(&id).unwrap()),
                    );
                }
            });

        if let Some(result) = editor_result {
            if let Some(response) = result.ingredients {
                if response.lost_focus() {
                    if let Some(id) = self.editor_recipe_id {
                        self.meal_planner.lookup_nutrients_for_recipe_id(&ctx, id);
                    }
                }
            }
            if result.close {
                self.editor_visible = false;
            }
        }

        if response.is_none() || !self.editor_visible {
            self.editor_recipe_id = None;
            self.meal_planner.delete_draft_recipe();
        }

        // Shopping List window
        let shopping_size = egui::vec2(
            (ctx.content_rect().width() - 40.0).clamp(320.0, 1080.0),
            (ctx.content_rect().height() - 40.0).clamp(300.0, 900.0),
        );
        let mut close_shopping_list = false;
        egui::Window::new("Shopping List")
            .id(egui::Id::new("shopping_list_sheet"))
            .open(&mut self.shopping_list_visible)
            .title_bar(false)
            .collapsible(false)
            .resizable(true)
            .frame(paper_window_frame(ui.style()))
            .default_size(shopping_size)
            .min_size(egui::vec2(320.0, 300.0))
            .max_size(
                (ctx.content_rect().size() - egui::vec2(16.0, 16.0)).max(egui::vec2(320.0, 300.0)),
            )
            .default_pos(ctx.content_rect().center() - shopping_size * 0.5)
            .show(&ctx, |ui| {
                close_shopping_list = self.shopping_list.show(ui, &self.meal_planner);
            });
        if close_shopping_list {
            self.shopping_list_visible = false;
        }

        // Settings window
        let mut close_settings = false;
        egui::Window::new("Settings")
            .open(&mut self.settings_window_visible)
            .title_bar(false)
            .collapsible(false)
            .frame(paper_window_frame(ui.style()))
            .default_width(520.0)
            .min_height(240.0)
            .resizable(true)
            .show(&ctx, |ui| {
                close_settings = paper_window_header(ui, "Settings");
                egui::Frame::new()
                    .fill(palette::SURFACE)
                    .inner_margin(16)
                    .corner_radius(6)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label("Edamam API Key");
                            ui.text_edit_singleline(&mut self.meal_planner.api_key);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Edamam APP ID");
                            ui.text_edit_singleline(&mut self.meal_planner.app_id);
                        });
                    });
            });

        if close_settings {
            self.settings_window_visible = false;
        }

        // Welcome screen
        let mut welcome_visible = !self.meal_planner.is_api_configured() && !self.welcome_dismissed;
        let mut close_welcome = false;
        egui::Window::new("Welcome Screen")
            .open(&mut welcome_visible)
            .title_bar(false)
            .collapsible(false)
            .frame(paper_window_frame(ui.style()))
            .default_width(560.0)
            .min_height(400.)
            .max_height(650.)
            .resizable(true)
            .show(&ctx, |ui| {
                close_welcome = paper_window_header(ui, "Welcome");
                egui::Frame::new().fill(palette::SURFACE).inner_margin(16).corner_radius(6)
                    .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.heading("You haven't configured APP_ID and API_KEY for Edamam service");
                ui.add_space(DEFAULT_PADDING);
                ui.horizontal(|ui| {
                    ui.label("Go to ");
                    if ui.link("https://www.edamam.com ").clicked() {
                        // OpenUrl::new_tab("https://www.edamam.com");
                    };
                    ui.label("and sign up for a free account.");
                });
                ui.add_space(DEFAULT_PADDING);
                ui.label("Create a new app for the Nutrition Analysis API. Use the API_KEY and APP_ID in Settings Window.");
                ui.add_space(DEFAULT_PADDING);
                ui.label("You can use the Planner and Browse Recipe features without an Edamam account.");
                    });
            });
        if close_welcome {
            self.welcome_dismissed = true;
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use egui::{pos2, vec2, Event, Id, Modifiers, PointerButton, RawInput, Rect};

    fn render_import(
        ctx: &egui::Context,
        app: &mut MealPlannerApp,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1400.0, 1200.0))),
                events,
                ..Default::default()
            },
            |ui| app.show_recipe_import(ui),
        );
        output.textures_delta.clear();
        output
    }

    #[test]
    fn resizing_import_window_keeps_supported_websites_below_a_compact_form() {
        let ctx = egui::Context::default();
        crate::set_theme(&ctx, crate::Theme::default());
        let mut app = MealPlannerApp {
            import_recipe_visible: true,
            ..Default::default()
        };
        for _ in 0..3 {
            render_import(&ctx, &mut app, vec![]);
        }
        let window_id = Id::new(Some("Import Recipe from URL"));
        let initial = ctx.memory(|memory| memory.area_rect(window_id).unwrap());
        let before = render_import(&ctx, &mut app, vec![]);
        let site_bounds = |output: &egui::FullOutput, text: &str| {
            output
                .shapes
                .iter()
                .find_map(|shape| {
                    if let egui::Shape::Text(label) = &shape.shape {
                        if label.galley.text() == text {
                            let rect = Rect::from_min_size(label.pos, label.galley.size());
                            assert!(shape.clip_rect.contains_rect(rect), "{text} is clipped");
                            return Some(rect);
                        }
                    }
                    None
                })
                .unwrap_or_else(|| panic!("{text} is not visible"))
        };
        let sites_y = site_bounds(&before, "SUPPORTED WEBSITES").top() - initial.top();
        let corner = initial.right_bottom() - vec2(2.0, 2.0);
        let destination = corner + vec2(200.0, 350.0);
        for events in [
            vec![Event::PointerMoved(corner)],
            vec![Event::PointerButton {
                pos: corner,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            }],
            vec![Event::PointerMoved(destination)],
            vec![Event::PointerButton {
                pos: destination,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        ] {
            render_import(&ctx, &mut app, events);
        }
        for _ in 0..3 {
            render_import(&ctx, &mut app, vec![]);
        }
        let resized = ctx.memory(|memory| memory.area_rect(window_id).unwrap());
        assert!(resized.height() > initial.height() + 200.0);
        let after = render_import(&ctx, &mut app, vec![]);
        assert!(
            (site_bounds(&after, "SUPPORTED WEBSITES").top() - resized.top() - sites_y).abs() < 1.0
        );
        for (name, domains) in supported_sites() {
            assert!(resized.contains_rect(site_bounds(&after, name)));
            for domain in domains {
                assert!(resized.contains_rect(site_bounds(&after, domain)));
            }
        }
    }
}
