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
    api_key: String,
    app_id: String,
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

    pub recipies: Vec<Recipe>,
    pub recipe: Recipe,
    pub daily_plan: Vec<Vec<usize>>,
}

impl Default for MealPlannerApp {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            app_id: String::new(),
            planner: Planner::default(),
            editor_visible: false,
            shopping_list_visible: false,
            settings_window_visible: false,
            browser: RecipeBrowser::default(),
            shopping_list: ShoppingList::default(),
            recipies: vec![],
            recipe: Recipe::default(),
            download: Arc::new(Mutex::new(Download::None)),
            import_data: Arc::new(Mutex::new((String::new(), vec![]))),
            daily_plan: vec![vec![], vec![], vec![], vec![], vec![], vec![]],
        }
    }
}

impl MealPlannerApp {
    pub fn load_from_bytes(&mut self, json: &[u8]) {
        let state: MealPlannerApp = serde_json::from_slice(json).unwrap();
        self.recipe = state.recipe;
        self.recipies = state.recipies;
        self.daily_plan = state.daily_plan;
    }

    #[cfg(target_arch = "wasm32")]
    pub fn is_daily_plan_empty(&self) -> bool {
        let mut is_empty = 0;
        for day in &self.daily_plan {
            is_empty = is_empty >> day.len();
        }

        is_empty == 0
    }

    #[cfg(target_arch = "Wasm32")]
    pub fn same_recipe_collection(&self, incoming: &Vec<Recipe>) -> bool {
        for r in self.recipies {
            let mut not_found = true;
            for ir in incoming {
                if r.title == ir.title {
                    not_found = false;
                    break;
                }
            }
            if not_found {
                return false;
            }
        }
        true
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
        let json = base64::decode(include_bytes!("../state.json")).unwrap();
        let default_state = serde_json::from_slice(json.as_slice()).unwrap();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let previous_state: MealPlannerApp =
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            if previous_state.is_daily_plan_empty() && previous_state.recipies.len() == 0 {
                return default_state;
            }
            return previous_state;
        }

        Default::default()
    }

    fn request(&mut self, ctx: &egui::Context) {
        let analysis_request = AnalysisRequest {
            ingr: self.recipe.ingredients_to_vec(),
        };

        let url = format!(
            "https://api.edamam.com/api/nutrition-details?app_id={}&app_key={}",
            self.app_id, self.api_key
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
        //TODO: currently broken because of bug in egui
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
                        self.recipe.macros = analysis;
                    }
                }
            };
        }

        {
            if let Ok(mut lock) = self.import_data.clone().try_lock() {
                if !lock.0.is_empty() {
                    let content = std::fs::read_to_string(&lock.0).expect("Unable to read");
                    self.load_from_bytes(BASE64_STANDARD.decode(content).unwrap().as_slice());
                    lock.0 = String::new();
                }

                if !lock.1.is_empty() {
                    self.load_from_bytes(BASE64_STANDARD.decode(&lock.1).unwrap().as_slice());
                    lock.1 = vec![];
                }
            }
        }

        {
            // save our recipe if we closed the editor and it's been given a title
            if !self.editor_visible && !self.recipe.title.is_empty() {
                if let EditState::EDITING(recipe_idx) = self.browser.edit_recipe_idx {
                    self.recipies[recipe_idx] = self.recipe.clone();
                } else {
                    self.recipies.push(self.recipe.clone());
                }
                self.recipe = Recipe::default();
                self.download = Arc::new(Mutex::new(Download::None));
                self.browser.edit_recipe_idx = EditState::EMPTY;
            }
        }

        {
            if let EditState::DeleteRecipeAtIndex(idx) = self.browser.edit_recipe_idx {
                self.recipies.remove(idx);
                self.browser.edit_recipe_idx = EditState::EMPTY;
            }
        }

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("main_top_panel").show(ctx, |ui| {
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

                // #[cfg(not(target_arch = "wasm32"))]
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

        egui::CentralPanel::default().show(ctx, |ui| {
            let max_height = ui.available_height();
            if let EditState::PENDING(recipe_idx) = self.browser.edit_recipe_idx {
                self.recipe = self.recipies.get(recipe_idx).unwrap().clone();
                self.browser.edit_recipe_idx = EditState::EDITING(recipe_idx);
                self.editor_visible = true;
            }

            egui::TopBottomPanel::top("recipe_browser").default_height(percentage(max_height, 60)).resizable(true).show_inside(ui, |ui| {
                self.browser.show(ui, &self.recipies);
            });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.planner.ui(ui, &mut self.daily_plan, &self.recipies);
            });

            let response = egui::Window::new("Recipe Editor")
                .open(&mut self.editor_visible)
                .resizable(true)
                .collapsible(false)
                .default_height(600.)
                .default_width(percentage(ui.max_rect().width(), 80))
                .show(&ctx.clone(), |ui| Editor::new().ui(ui, &mut self.recipe));
            if let Some(inner) = response {
                if let Some(response) = inner.inner {
                    if response.lost_focus() {
                        self.request(ctx);
                    }
                }
            }

            egui::Window::new("Shopping List")
                .open(&mut self.shopping_list_visible)
                .min_height(300.)
                .resizable(true)
                .show(&ctx.clone(), |ui| {
                    self.shopping_list.show(ui, &self.daily_plan, &self.recipies);
               });

            egui::Window::new("Settings")
                .open(&mut self.settings_window_visible)
                .min_height(300.)
                .resizable(true)
                .show(&ctx.clone(), |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Edamam API Key");
                            ui.text_edit_singleline(&mut self.api_key);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Edamam APP ID");
                            ui.text_edit_singleline(&mut self.app_id);
                        });
                    });
                });

            let mut show_welcome_screen = self.api_key.is_empty() || self.app_id.is_empty();
            egui::Window::new("Welcome Screen")
                .open(&mut show_welcome_screen)
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
       });
    }
}
