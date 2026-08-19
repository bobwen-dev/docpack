use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::constants;
use crate::ignore::ExcludeRules;
use crate::lang::I18n;
use crate::pack;
use crate::settings::Settings;
use crate::style;

enum AppState {
    Idle,
    Done(String),
    Error(String),
}

pub struct DocPackApp {
    i18n: I18n,
    source_paths: Vec<PathBuf>,
    output_path: Option<PathBuf>,
    state: AppState,
    drag_hovered: bool,
    selected_indices: Vec<usize>,
    last_arrow_pressed: Option<egui::Key>,
    show_settings: bool,
    show_first_run: bool,
    settings_tab: usize,
    exclude_text: String,
    exclude_text_initial: String,
    file_count: usize,
    binary_skipped: usize,
    settings: Settings,
    pack_progress: Option<Arc<Mutex<pack::Progress>>>,
    pack_result_rx: Option<std::sync::mpsc::Receiver<Result<pack::PackResult, String>>>,
}

impl DocPackApp {
    pub fn new(mut settings: Settings) -> Self {
        let mut i18n = I18n::new();
        i18n.load_lang(&settings.language);
        let exclude_text = if settings.exclude_file.is_empty() {
            crate::ignore::ExcludeRules::new().patterns().join("\n")
        } else {
            std::mem::take(&mut settings.exclude_file)
        };
        let exclude_text_initial = exclude_text.clone();
        Self {
            i18n,
            source_paths: Vec::new(),
            output_path: None,
            state: AppState::Idle,
            drag_hovered: false,
            selected_indices: Vec::new(),
            last_arrow_pressed: None,
            show_settings: false,
            show_first_run: !settings.context_menu_prompted,
            settings_tab: 0,
            exclude_text,
            exclude_text_initial,
            file_count: 0,
            binary_skipped: 0,
            settings,
            pack_progress: None,
            pack_result_rx: None,
        }
    }

    fn sync_exclude_to_settings(&mut self) {
        let trimmed = self.exclude_text.trim();
        if trimmed.is_empty() {
            self.settings.exclude_file = String::new();
            self.exclude_text = ExcludeRules::new().patterns().join("\n");
        } else {
            self.settings.exclude_file = self.exclude_text.clone();
        }
    }

    fn save_settings(&mut self) {
        self.sync_exclude_to_settings();
        if let Err(e) = crate::settings::save_settings(&self.settings) {
            self.state = AppState::Error(e);
        }
    }
}

impl eframe::App for DocPackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(style::PIXELS_PER_POINT);
        ctx.set_fonts(crate::style::get_font_definitions().clone());

        self.process_drag_and_drop(ctx);

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(self.i18n.get("app_name"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new("⚙").min_size(egui::vec2(28.0, 24.0)))
                        .clicked()
                    {
                        self.show_settings = !self.show_settings;
                    }
                });
            });
        });

        if self.show_settings {
            egui::Window::new(self.i18n.get("settings"))
                .id(egui::Id::new("settings"))
                .resizable(false)
                .collapsible(false)
                .default_width(500.0)
                .anchor(egui::Align2::CENTER_TOP, [0.0, 60.0])
                .show(ctx, |ui| {
                    self.show_settings_panel(ui);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_main_panel(ui);
        });

        // Progress / result popup
        self.show_pack_progress(ctx);

        // Check for completed pack result
        if let Some(ref rx) = self.pack_result_rx {
            if let Ok(result) = rx.try_recv() {
                self.pack_result_rx = None;
                self.pack_progress = None;
                match result {
                    Ok(res) => {
                        self.file_count = res.file_count;
                        self.binary_skipped = res.binary_skipped;
                    }
                    Err(e) => {
                        self.state = AppState::Error(e);
                    }
                }
            }
        }

        if self.show_first_run {
            egui::Window::new(self.i18n.get("first_run_title"))
                .id(egui::Id::new("first_run"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(self.i18n.get("first_run_message"));
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(self.i18n.get("first_run_install")).clicked() {
                            let exe = std::env::current_exe().unwrap_or_default();
                            match crate::platform::install::ContextMenu::install(&exe) {
                                Ok(()) => {
                                    self.settings.context_menu_prompted = true;
                                    self.save_settings();
                                    self.show_first_run = false;
                                }
                                Err(e) => {
                                    self.state = AppState::Error(e);
                                    self.show_first_run = false;
                                }
                            }
                        }
                        if ui.button(self.i18n.get("first_run_skip")).clicked() {
                            self.settings.context_menu_prompted = true;
                            self.save_settings();
                            self.show_first_run = false;
                        }
                    });
                });
        }
    }
}

impl DocPackApp {
    fn process_drag_and_drop(&mut self, ctx: &egui::Context) {
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            self.drag_hovered = true;
            ctx.request_repaint();
        }

        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.drag_hovered = false;
                for f in &i.raw.dropped_files {
                    if let Some(path) = &f.path {
                        self.source_paths.push(path.clone());
                    }
                }
            }
        });
    }

    fn show_main_panel(&mut self, ui: &mut egui::Ui) {
        if self.drag_hovered {
            ui.painter()
                .rect_filled(ui.max_rect(), 10.0, style::DRAG_HOVER_COLOR);
        }

        ui.label(self.i18n.get("select_path"));

        let bottom_h = 95.0;
        let max_h = (ui.available_height() - bottom_h - 30.0).max(0.0);
        egui::ScrollArea::vertical()
            .max_height(max_h)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                if !self.source_paths.is_empty() {
                    for (i, path) in self.source_paths.iter().enumerate() {
                        let selected = self.selected_indices.contains(&i);
                        let response =
                            ui.selectable_label(selected, path.to_string_lossy().to_string());
                        if selected {
                            response.scroll_to_me(Some(egui::Align::Center));
                        }
                        if response.clicked() {
                            let shift = ui.input(|i| i.modifiers.shift);
                            let ctrl = ui.input(|i| i.modifiers.ctrl);
                            if ctrl {
                                if let Some(pos) =
                                    self.selected_indices.iter().position(|x| *x == i)
                                {
                                    self.selected_indices.remove(pos);
                                } else {
                                    self.selected_indices.push(i);
                                }
                            } else if shift {
                                let last = self.selected_indices.last().copied().unwrap_or(0);
                                let (start, end) = if last <= i { (last, i) } else { (i, last) };
                                self.selected_indices = (start..=end).collect();
                            } else {
                                self.selected_indices = vec![i];
                            }
                        }
                    }
                } else {
                    ui.add_space(max_h.max(40.0));
                }
            });

        // Handle keyboard navigation: arrows and delete
        let arrow_down = ui.input(|i| {
            i.key_pressed(egui::Key::ArrowDown)
                && self.last_arrow_pressed != Some(egui::Key::ArrowDown)
        });
        let arrow_up = ui.input(|i| {
            i.key_pressed(egui::Key::ArrowUp) && self.last_arrow_pressed != Some(egui::Key::ArrowUp)
        });
        if arrow_down {
            self.last_arrow_pressed = Some(egui::Key::ArrowDown);
            let next = if self.selected_indices.is_empty() {
                0
            } else {
                let last = self.selected_indices.last().copied().unwrap_or(0);
                if last + 1 < self.source_paths.len() {
                    last + 1
                } else {
                    last
                }
            };
            self.selected_indices = vec![next];
        } else if arrow_up {
            self.last_arrow_pressed = Some(egui::Key::ArrowUp);
            let prev = if self.selected_indices.is_empty() {
                self.source_paths.len().saturating_sub(1)
            } else {
                let first = self.selected_indices.first().copied().unwrap_or(0);
                if first > 0 {
                    first - 1
                } else {
                    0
                }
            };
            self.selected_indices = vec![prev];
        } else if !ui.input(|i| i.key_down(egui::Key::ArrowDown) || i.key_down(egui::Key::ArrowUp))
        {
            self.last_arrow_pressed = None;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            if !self.selected_indices.is_empty() {
                let mut indices = self.selected_indices.clone();
                indices.sort_unstable();
                for idx in indices.iter().rev() {
                    if *idx < self.source_paths.len() {
                        self.source_paths.remove(*idx);
                    }
                }
                self.selected_indices.clear();
                self.file_count = 0;
                self.binary_skipped = 0;
            }
        }
        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::A)) {
            self.selected_indices = (0..self.source_paths.len()).collect();
        }

        ui.add_space(style::SPACING);
        ui.separator();
        ui.add_space(style::SPACING_GAP);

        ui.horizontal(|ui| {
            ui.label(self.i18n.get("output_path"));
            let out = self
                .output_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            let mut out_str = out.clone();
            ui.text_edit_singleline(&mut out_str);
            if out_str != out {
                self.output_path = if out_str.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&out_str))
                };
            }
        });

        ui.add_space(style::SPACING_GAP);

        if self.file_count > 0 {
            let msg = self
                .i18n
                .get("file_count")
                .replace("{count}", &self.file_count.to_string());
            ui.label(msg);
            if self.binary_skipped > 0 {
                let msg = self
                    .i18n
                    .get("binary_skipped")
                    .replace("{count}", &self.binary_skipped.to_string());
                ui.label(msg);
            }
        }

        ui.add_space(style::SPACING_GAP);
        ui.vertical_centered(|ui| {
            if ui.button(self.i18n.get("pack")).clicked() {
                self.do_pack();
            }
        });
        ui.add_space(style::SPACING_GAP);

        match &self.state {
            AppState::Done(msg) => {
                ui.colored_label(style::COLOR_SUCCESS, msg);
            }
            AppState::Error(msg) => {
                ui.colored_label(style::COLOR_ERROR, msg);
            }
            _ => {}
        }
    }

    fn show_settings_panel(&mut self, ui: &mut egui::Ui) {
        let tabs = vec![
            self.i18n.get("tab_exclude").to_string(),
            self.i18n.get("tab_encoding").to_string(),
            self.i18n.get("tab_language").to_string(),
            self.i18n.get("tab_context_menu").to_string(),
            self.i18n.get("tab_about").to_string(),
        ];
        egui::ScrollArea::vertical().show(ui, |ui| {
            let _response = ui.horizontal_wrapped(|ui| {
                for (i, tab) in tabs.iter().enumerate() {
                    let selected = self.settings_tab == i;
                    if ui.selectable_label(selected, tab.clone()).clicked() {
                        self.settings_tab = i;
                    }
                    if i < tabs.len() - 1 {
                        ui.separator();
                    }
                }
            });
            ui.separator();

            match self.settings_tab {
                0 => {
                    ui.label(self.i18n.get("exclude_file"));
                    ui.label(self.i18n.get("exclude_desc"));
                    ui.label(self.i18n.get("exclude_hint"));
                    let _ = egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.exclude_text)
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(8)
                                    .hint_text(self.i18n.get("exclude_hint_placeholder")),
                            );
                        });
                }
                1 => {
                    ui.label(self.i18n.get("encoding"));
                    ui.label(self.i18n.get("encoding_desc"));
                    ui.label(self.i18n.get("encoding_hint"));
                    let _ = egui::ScrollArea::vertical().show(ui, |ui| {
                        let encodings: Vec<String> = if self.settings.local_encodings.is_empty() {
                            crate::settings::DEFAULT_LOCAL_ENCODINGS
                                .iter()
                                .map(|s| s.to_string())
                                .collect()
                        } else {
                            self.settings.local_encodings.clone()
                        };
                        let mut text: String = encodings.join("\n");
                        let _ = egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut text)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(6)
                                        .hint_text(self.i18n.get("encoding_hint_placeholder")),
                                );
                            });
                        let new_encodings: Vec<String> =
                            text.lines().map(|l| l.trim().to_string()).collect();
                        let encodings_to_save: Vec<String> = if new_encodings.is_empty() {
                            crate::settings::DEFAULT_LOCAL_ENCODINGS
                                .iter()
                                .map(|s| s.to_string())
                                .collect()
                        } else {
                            new_encodings
                        };
                        if encodings_to_save != encodings {
                            self.settings.local_encodings = encodings_to_save;
                        }
                    });
                }
                2 => {
                    ui.label(self.i18n.get("language"));
                    ui.label(self.i18n.get("language_desc"));
                    for (code, name) in self.i18n.available_langs() {
                        let selected = self.i18n.current_code() == code;
                        if ui.radio(selected, &name).clicked() {
                            self.i18n.set_lang(&code);
                            self.settings.language = code.clone();
                        }
                    }
                }
                3 => {
                    ui.label(self.i18n.get("context_menu"));
                    ui.label(self.i18n.get("context_menu_desc"));
                    if ui.button(self.i18n.get("install_context_menu")).clicked() {
                        let exe = std::env::current_exe().unwrap_or_default();
                        match crate::platform::install::ContextMenu::install(&exe) {
                            Ok(()) => {
                                self.state =
                                    AppState::Done(self.i18n.get("context_menu_installed"));
                            }
                            Err(e) => {
                                self.state = AppState::Error(e);
                            }
                        }
                    }
                    if ui.button(self.i18n.get("uninstall_context_menu")).clicked() {
                        match crate::platform::install::ContextMenu::uninstall() {
                            Ok(()) => {
                                self.state =
                                    AppState::Done(self.i18n.get("context_menu_uninstalled"));
                            }
                            Err(e) => {
                                self.state = AppState::Error(e);
                            }
                        }
                    }
                }
                4 => {
                    ui.label(self.i18n.get("about"));
                    ui.label(format!(
                        "{} v{}",
                        self.i18n.get("app_name"),
                        env!("CARGO_PKG_VERSION")
                    ));
                    ui.label(self.i18n.get("app_desc"));
                }
                _ => {}
            }
        });
        ui.separator();
        ui.add_space(style::SPACING);
        ui.horizontal(|ui| {
            let btn_w = 120.0;
            let space = (ui.available_width() - btn_w * 2.0 - style::SPACING).max(0.0);
            ui.add_space(space * 0.5);
            if ui
                .add_sized([btn_w, 24.0], egui::Button::new(self.i18n.get("save")))
                .clicked()
            {
                self.sync_exclude_to_settings();
                if let Err(e) = crate::settings::save_settings(&self.settings) {
                    self.state = AppState::Error(e);
                }
                self.show_settings = false;
            }
            ui.add_space(style::SPACING);
            if ui
                .add_sized([btn_w, 24.0], egui::Button::new(self.i18n.get("cancel")))
                .clicked()
            {
                self.settings = crate::settings::load_settings();
                self.exclude_text = self.exclude_text_initial.clone();
                self.i18n.load_lang(&self.settings.language);
                self.show_settings = false;
            }
        });
        ui.add_space(style::SPACING);
    }

    fn do_pack(&mut self) {
        if self.source_paths.is_empty() {
            self.state = AppState::Error(self.i18n.get("no_source_paths"));
            return;
        }
        if self.pack_progress.is_some() {
            return; // already packing
        }

        let output = self.output_path.clone().unwrap_or_else(|| {
            pack::resolve_output_name(&self.source_paths, Path::new(constants::DEFAULT_OUTPUT))
        });

        let rules = if self.exclude_text.trim().is_empty() {
            ExcludeRules::new()
        } else {
            ExcludeRules::from_rules_text(&self.exclude_text)
        };

        let sources = self.source_paths.clone();
        let local_encodings = if self.settings.local_encodings.is_empty() {
            crate::settings::DEFAULT_LOCAL_ENCODINGS
                .iter()
                .map(|s| s.to_string())
                .collect()
        } else {
            self.settings.local_encodings.clone()
        };

        let progress = Arc::new(Mutex::new(pack::Progress {
            current: 0,
            total: 0,
            file: String::new(),
            phase: pack::ProgressPhase::Collecting,
        }));
        self.pack_progress = Some(progress.clone());

        let (tx, rx) = std::sync::mpsc::channel();
        self.pack_result_rx = Some(rx);

        std::thread::spawn(move || {
            let on_progress: pack::ProgressFn = Box::new(move |p| {
                if let Ok(mut guard) = progress.lock() {
                    *guard = p;
                }
            });

            let result = if sources.len() == 1 && sources[0].is_dir() {
                pack::pack_dir(
                    &sources[0],
                    &output,
                    &rules,
                    &local_encodings,
                    Some(&on_progress),
                )
            } else {
                pack::pack_files(
                    &sources,
                    &output,
                    &rules,
                    &local_encodings,
                    Some(&on_progress),
                )
            };

            let _ = tx.send(result);
        });
    }

    fn show_pack_progress(&self, ctx: &egui::Context) {
        let progress = match &self.pack_progress {
            Some(p) => p,
            None => return,
        };
        let p = match progress.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        let (title, msg) = match p.phase {
            pack::ProgressPhase::Collecting => (
                self.i18n.get("packing").to_string(),
                self.i18n.get("packing").to_string(),
            ),
            pack::ProgressPhase::Reading => (
                self.i18n.get("packing").to_string(),
                format!("{} ({} / {})", p.file, p.current, p.total),
            ),
            pack::ProgressPhase::Writing => (
                self.i18n.get("packing").to_string(),
                self.i18n.get("packing").to_string(),
            ),
            pack::ProgressPhase::Done => (
                self.i18n.get("packing").to_string(),
                self.i18n.get("done").to_string(),
            ),
        };

        let progress_val = if p.total > 0 {
            p.current as f32 / p.total as f32
        } else {
            0.0
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add(egui::ProgressBar::new(progress_val).desired_width(250.0));
                ui.add_space(6.0);
                ui.label(msg);
            });
    }
}

pub fn get_icon_data() -> egui::IconData {
    let icon_data = crate::icon_bytes::ICON_PNG;
    match image::load_from_memory(icon_data) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            egui::IconData {
                rgba: rgba.into_raw(),
                width: w,
                height: h,
            }
        }
        Err(_) => egui::IconData::default(),
    }
}

pub fn run_gui(settings: Settings, initial_paths: Vec<PathBuf>) -> Result<(), String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((style::WINDOW_WIDTH, style::WINDOW_HEIGHT))
            .with_drag_and_drop(true)
            .with_icon(get_icon_data()),
        ..Default::default()
    };

    eframe::run_native(
        constants::APP_NAME,
        options,
        Box::new(move |_cc| {
            let mut app = DocPackApp::new(settings);
            app.source_paths = initial_paths;
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| format!("GUI error: {}", e))
}

struct PackOnlyApp {
    progress: Arc<Mutex<pack::Progress>>,
    result_rx: Option<std::sync::mpsc::Receiver<Result<pack::PackResult, String>>>,
    done: bool,
    result_msg: String,
    is_ok: bool,
    i18n: I18n,
}

impl eframe::App for PackOnlyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(style::PIXELS_PER_POINT);
        ctx.set_fonts(crate::style::get_font_definitions().clone());

        if !self.done {
            if let Some(ref rx) = self.result_rx {
                if let Ok(result) = rx.try_recv() {
                    self.done = true;
                    self.result_rx = None;
                    match result {
                        Ok(res) => {
                            self.is_ok = true;
                            self.result_msg = self
                                .i18n
                                .get("pack_done")
                                .replace("{count}", &res.file_count.to_string())
                                .replace("{path}", &res.output_path.to_string_lossy());
                        }
                        Err(e) => {
                            self.is_ok = false;
                            self.result_msg = e;
                        }
                    }
                }
            }
        }

        let p = match self.progress.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                if self.done {
                    if self.is_ok {
                        ui.colored_label(style::COLOR_SUCCESS, &self.result_msg);
                    } else {
                        ui.colored_label(style::COLOR_ERROR, &self.result_msg);
                    }
                } else {
                    let progress_val = if p.total > 0 {
                        p.current as f32 / p.total as f32
                    } else {
                        0.0
                    };
                    ui.add(egui::ProgressBar::new(progress_val).desired_width(280.0));
                    ui.add_space(8.0);
                    match p.phase {
                        pack::ProgressPhase::Collecting => {
                            ui.label(format!(
                                "{} {}/{}",
                                self.i18n.get("scanning"),
                                p.current,
                                p.total
                            ));
                        }
                        pack::ProgressPhase::Reading => {
                            ui.label(format!("{} ({}/{})", p.file, p.current, p.total));
                        }
                        _ => {
                            ui.label(self.i18n.get("unpacking"));
                        }
                    }
                }
                ui.add_space(16.0);
                let ok_btn = ui.add_enabled(
                    self.done,
                    egui::Button::new(self.i18n.get("ok")).min_size(egui::vec2(80.0, 32.0)),
                );
                if ok_btn.clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ui.add_space(12.0);
            });
        });

        ctx.request_repaint();
    }
}

pub fn run_gui_pack(settings: Settings, paths: Vec<PathBuf>) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No paths".into());
    }

    let mut i18n = I18n::new();
    i18n.load_lang(&settings.language);

    let output = pack::resolve_output_name(&paths, Path::new(constants::DEFAULT_OUTPUT));

    let app_settings = settings;
    let rules = if app_settings.exclude_file.trim().is_empty() {
        ExcludeRules::new()
    } else {
        ExcludeRules::from_rules_text(&app_settings.exclude_file)
    };
    let local_encodings = if app_settings.local_encodings.is_empty() {
        crate::settings::DEFAULT_LOCAL_ENCODINGS
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        app_settings.local_encodings.clone()
    };

    let progress = Arc::new(Mutex::new(pack::Progress {
        current: 0,
        total: 0,
        file: String::new(),
        phase: pack::ProgressPhase::Collecting,
    }));

    let (tx, rx) = std::sync::mpsc::channel();
    let progress_clone = progress.clone();

    std::thread::spawn(move || {
        let on_progress: pack::ProgressFn = Box::new(move |p| {
            if let Ok(mut guard) = progress_clone.lock() {
                *guard = p;
            }
        });

        let result = pack::pack_files(
            &paths,
            &output,
            &rules,
            &local_encodings,
            Some(&on_progress),
        );

        let _ = tx.send(result);
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((380.0, 150.0))
            .with_resizable(false)
            .with_icon(get_icon_data()),
        ..Default::default()
    };

    eframe::run_native(
        constants::APP_NAME,
        options,
        Box::new(move |_cc| {
            Ok(Box::new(PackOnlyApp {
                progress,
                result_rx: Some(rx),
                done: false,
                result_msg: String::new(),
                is_ok: true,
                i18n,
            }))
        }),
    )
    .map_err(|e| format!("GUI error: {}", e))
}

pub fn run_gui_unpack(settings: Settings, paths: Vec<PathBuf>) -> Result<(), String> {
    if paths.is_empty() {
        return Err("No paths".into());
    }

    let mut i18n = I18n::new();
    i18n.load_lang(&settings.language);

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let mut total_files = 0usize;
        let mut last_output = PathBuf::new();
        for input in &paths {
            let default_output = crate::unpack::default_output_path(input);
            let output = crate::unpack::resolve_unpack_output(&default_output);
            match crate::unpack::unpack_docx(input, &output) {
                Ok(res) => {
                    total_files += res.file_count;
                    last_output = output;
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            }
        }
        let _ = tx.send(Ok(crate::unpack::UnpackResult {
            file_count: total_files,
            output_dir: last_output,
        }));
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((400.0, 130.0))
            .with_resizable(false)
            .with_icon(get_icon_data()),
        ..Default::default()
    };

    eframe::run_native(
        constants::APP_NAME,
        options,
        Box::new(move |_cc| {
            Ok(Box::new(UnpackResultApp {
                result_rx: Some(rx),
                done: false,
                result_msg: String::new(),
                is_ok: true,
                i18n,
            }))
        }),
    )
    .map_err(|e| format!("GUI error: {}", e))
}

struct UnpackResultApp {
    result_rx: Option<std::sync::mpsc::Receiver<Result<crate::unpack::UnpackResult, String>>>,
    done: bool,
    result_msg: String,
    is_ok: bool,
    i18n: I18n,
}

impl eframe::App for UnpackResultApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(style::PIXELS_PER_POINT);
        ctx.set_fonts(crate::style::get_font_definitions().clone());

        if !self.done {
            if let Some(ref rx) = self.result_rx {
                if let Ok(result) = rx.try_recv() {
                    self.done = true;
                    self.result_rx = None;
                    match result {
                        Ok(res) => {
                            self.is_ok = true;
                            self.result_msg = self
                                .i18n
                                .get("extracted_details")
                                .replace("{count}", &res.file_count.to_string())
                                .replace("{path}", &res.output_dir.display().to_string());
                        }
                        Err(e) => {
                            self.is_ok = false;
                            self.result_msg = e;
                        }
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                if self.done {
                    if self.is_ok {
                        ui.colored_label(style::COLOR_SUCCESS, &self.result_msg);
                    } else {
                        ui.colored_label(style::COLOR_ERROR, &self.result_msg);
                    }
                } else {
                    ui.label(self.i18n.get("packing"));
                }
                ui.add_space(16.0);
                let ok_btn = ui.add_enabled(
                    self.done,
                    egui::Button::new(self.i18n.get("ok")).min_size(egui::vec2(80.0, 32.0)),
                );
                if ok_btn.clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ui.add_space(12.0);
            });
        });

        if !self.done {
            ctx.request_repaint();
        }
    }
}
