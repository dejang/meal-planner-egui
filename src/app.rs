use core::str;
use std::{
    future::Future,
    sync::{Arc, Mutex},
};

use base64::prelude::*;
use ehttp::Request;
use log::error;
use rfd::FileHandle;

use crate::{
    meal_planner::MealPlanner,
    models::{AnalysisRequest, AnalysisResponse, Recipe},
    planner::Planner,
    recipe_editor::Editor,
    recipe_viewer::{EditState, RecipeBrowser},
    shopping_list::ShoppingList,
    util::{percentage, DEFAULT_PADDING},
};

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: std::future::Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn execute<F: std::future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Download {
    #[default]
    None,
    InProgress,
    Done(ehttp::Result<AnalysisResponse>),
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
    pub shopping_list_visible: bool,
    #[serde(skip)]
    pub settings_window_visible: bool,
    #[serde(skip)]
    download: Arc<Mutex<Download>>,
    #[serde(skip)]
    import_data: Arc<Mutex<(String, Vec<u8>)>>,
    #[serde(skip)]
    browser: RecipeBrowser,
    #[serde(skip)]
    shopping_list: ShoppingList,
    meal_planner: MealPlanner,
}

impl Default for MealPlannerApp {
    fn default() -> Self {
        Self {
            planner: Planner::default(),
            editor_visible: false,
            shopping_list_visible: false,
            settings_window_visible: false,
            browser: RecipeBrowser::default(),
            shopping_list: ShoppingList::default(),
            download: Arc::new(Mutex::new(Download::None)),
            import_data: Arc::new(Mutex::new((String::new(), vec![]))),
            meal_planner: MealPlanner::default(),
        }
    }
}

impl MealPlannerApp {
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
                && previous_state.meal_planner.recipies.len() == 0
            {
                return default_state;
            }
            return previous_state;
        }

        Default::default()
    }

    fn request(&mut self, ctx: &egui::Context) {
        let analysis_request = AnalysisRequest {
            ingr: self.meal_planner.recipe.ingredients_to_vec(),
        };

        let url = format!(
            "https://api.edamam.com/api/nutrition-details?app_id={}&app_key={}",
            self.meal_planner.get_app_id(),
            self.meal_planner.get_api_key()
        );

        let ctx = ctx.clone();
        let download_store = self.download.clone();
        *download_store.lock().unwrap() = Download::InProgress;
        ehttp::fetch(
            Request::json(url, &analysis_request).unwrap(),
            move |response| {
                if let Ok(response) = response {
                    let raw_text = response.text().unwrap();
                    if response.status == 200 {
                        let analysis = serde_json::from_str(raw_text).unwrap();
                        *download_store.lock().unwrap() = Download::Done(Ok(analysis));
                    } else {
                        *download_store.lock().unwrap() = Download::Done(Err(raw_text.to_string()));
                    }
                    ctx.request_repaint(); // Wake up UI thread
                } else {
                    let network_error = response.err().unwrap();
                    error!("Network Error: {}", network_error);
                    *download_store.lock().unwrap() = Download::Done(Err(network_error));
                    ctx.request_repaint(); // Wake up UI thread
                }
            },
        );
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

        let content = serde_json::to_string_pretty(&self).unwrap();
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            // process any incoming download for the currently editing recipe
            let download = self.download.lock().unwrap();

            match download.clone() {
                Download::None => {}
                Download::InProgress => {}
                Download::Done(result) => {
                    if let Ok(analysis) = result {
                        self.meal_planner.recipe.macros = analysis;
                    }
                }
            };
        }

        {
            if let Ok(mut lock) = self.import_data.clone().try_lock() {
                if !lock.0.is_empty() {
                    let content = std::fs::read_to_string(&lock.0).expect("Unable to read");
                    self.meal_planner
                        .load_from_bytes(BASE64_STANDARD.decode(content).unwrap().as_slice());
                    lock.0 = String::new();
                }

                if !lock.1.is_empty() {
                    self.meal_planner
                        .load_from_bytes(BASE64_STANDARD.decode(&lock.1).unwrap().as_slice());
                    lock.1 = vec![];
                }
            }
        }

        {
            // save our recipe if we closed the editor and it's been given a title
            if !self.editor_visible && !self.meal_planner.recipe.title.is_empty() {
                if let EditState::Editing(recipe_idx) = self.browser.edit_recipe_idx {
                    self.meal_planner.recipies[recipe_idx] = self.meal_planner.recipe.clone();
                } else {
                    self.meal_planner
                        .recipies
                        .push(self.meal_planner.recipe.clone());
                }
                self.meal_planner.recipe = Recipe::default();
                self.download = Arc::new(Mutex::new(Download::None));
                self.browser.edit_recipe_idx = EditState::Empty;
            }
        }

        {
            if let EditState::DeleteRecipeAtIndex(idx) = self.browser.edit_recipe_idx {
                self.meal_planner.remove_recipe(idx);
                self.browser.edit_recipe_idx = EditState::Empty;
            }
        }

        // Fixed top menu bar
        egui::TopBottomPanel::top("main_menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
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

                if ui.button("New Recipe").clicked() {
                    self.editor_visible = true;
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

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        // Resizable bottom panel for the planner
        let screen_height = ctx.screen_rect().height();
        egui::TopBottomPanel::bottom("planner_panel")
            .resizable(true)
            .show_separator_line(false)
            .min_height(screen_height * 0.15)
            .default_height(screen_height * 0.3)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(10.0)
                    .rounding(5.0)
                    .stroke(egui::Stroke::new(1.0, ui.visuals().window_stroke.color))
                    .shadow(egui::epaint::Shadow {
                        color: ui.visuals().window_shadow.color,
                        offset: egui::vec2(2.0, 2.0),
                        blur: 8.0,
                        spread: 2.0,
                    })
                    .show(ui, |ui| {
                        self.planner.ui(ui, &mut self.meal_planner);
                    });
            });

        // Central panel for recipe browser
        egui::CentralPanel::default().show(ctx, |ui| {
            if let EditState::Pending(recipe_idx) = self.browser.edit_recipe_idx {
                self.meal_planner.recipe =
                    self.meal_planner.recipies.get(recipe_idx).unwrap().clone();
                self.browser.edit_recipe_idx = EditState::Editing(recipe_idx);
                self.editor_visible = true;
            }

            self.browser.show(ui, &self.meal_planner.recipies);
        });

        // Recipe Editor window
        let response = egui::Window::new("Recipe Editor")
            .open(&mut self.editor_visible)
            .resizable(true)
            .collapsible(false)
            .default_height(600.)
            .default_width(percentage(ctx.screen_rect().width(), 80))
            .show(&ctx.clone(), |ui| {
                Editor::new().ui(ui, &mut self.meal_planner.recipe)
            });
        if let Some(inner) = response {
            if let Some(response) = inner.inner {
                if response.lost_focus() {
                    self.request(ctx);
                }
            }
        }

        // Shopping List window
        egui::Window::new("Shopping List")
            .open(&mut self.shopping_list_visible)
            .min_height(300.)
            .resizable(true)
            .show(&ctx.clone(), |ui| {
                self.shopping_list.show(
                    ui,
                    &self.meal_planner.daily_plan,
                    &self.meal_planner.recipies,
                );
            });

        // Settings window
        egui::Window::new("Settings")
            .open(&mut self.settings_window_visible)
            .min_height(300.)
            .resizable(true)
            .show(&ctx.clone(), |ui| {
                ui.group(|ui| {
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

        // Welcome screen
        egui::Window::new("Welcome Screen")
            .open(&mut !self.meal_planner.is_api_configured())
            .min_height(400.)
            .max_height(650.)
            .resizable(true)
            .show(ctx, |ui| {
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
    }
}
