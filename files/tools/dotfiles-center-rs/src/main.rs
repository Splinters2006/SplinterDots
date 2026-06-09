use eframe::egui::{self, Color32, RichText, ScrollArea, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const SETTINGS_KEYS: &[(&str, &str)] = &[
    ("DOTFILES_EDITOR", "nano"),
    ("DOTFILES_BROWSER", "zen-browser"),
    ("DOTFILES_THEME", "dark"),
    ("DOTFILES_ACCENT", "#89b4fa"),
    ("DOTFILES_WALLPAPER", ""),
    ("DOTFILES_TERMINAL", "kitty"),
    ("DOTFILES_FILE_MANAGER", "thunar"),
    ("DOTFILES_APP_LAUNCHER", "wofi --show drun"),
    ("DOTFILES_BAR_POSITION", "top"),
    ("DOTFILES_BAR_HEIGHT", "34"),
    ("DOTFILES_BAR_RADIUS", "8"),
    ("DOTFILES_BAR_OPACITY", "0.92"),
    ("DOTFILES_BAR_WORKSPACES", "left"),
    ("DOTFILES_BAR_CLOCK", "center"),
    ("DOTFILES_BAR_STATUS", "right"),
    ("DOTFILES_BAR_SHOW_WORKSPACES", "true"),
    ("DOTFILES_BAR_SHOW_CLOCK", "true"),
    ("DOTFILES_BAR_SHOW_VOLUME", "true"),
    ("DOTFILES_BAR_SHOW_NETWORK", "true"),
    ("DOTFILES_BAR_SHOW_BATTERY", "true"),
    ("DOTFILES_BAR_SHOW_CPU", "false"),
    ("DOTFILES_BAR_SHOW_MEMORY", "false"),
    ("DOTFILES_BAR_SHOW_TRAY", "false"),
    ("DOTFILES_BAR_CLOCK_FORMAT", "%a %d %b  %H:%M"),
    ("DOTFILES_BAR_CLOCK_SECONDS", "false"),
    ("DOTFILES_BAR_WORKSPACE_COUNT", "9"),
    ("DOTFILES_BAR_FONT_SIZE", "12"),
    ("DOTFILES_BAR_SPACING", "14"),
    ("DOTFILES_BAR_BORDER_WIDTH", "1"),
    ("DOTFILES_BAR_ACCENT_WORKSPACE", "true"),
];

const EXCLUDED_HYPR_OPTIONS: &[&str] = &[
    "misc:disable_hyprland_qtutils_check",
    "debug:watchdog_timeout",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Shortcut {
    name: String,
    key: String,
    kind: String,
    value: String,
}

#[derive(Clone, Debug, Deserialize)]
struct HyprSchemaFile {
    options: Vec<HyprOption>,
}

#[derive(Clone, Debug, Deserialize)]
struct HyprOption {
    section: Option<String>,
    path: String,
    label: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    default: Option<String>,
    choices: Option<Vec<String>>,
    min: Option<f64>,
    max: Option<f64>,
}

#[derive(Default)]
struct HyprNode {
    value: Option<String>,
    children: BTreeMap<String, HyprNode>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Overview,
    Hyprland,
    Shortcuts,
    Appearance,
    Bar,
    Setup,
}

struct Paths {
    root: PathBuf,
    config_home: PathBuf,
    home: PathBuf,
    dotfiles_dir: PathBuf,
    local_conf: PathBuf,
    keybinds_json: PathBuf,
    hypr_values_json: PathBuf,
    disabled_file: PathBuf,
    hypr_dir: PathBuf,
    hypr_colors: PathBuf,
    hypr_keybinds: PathBuf,
    hypr_generated: PathBuf,
    quickshell_dir: PathBuf,
    quickshell_file: PathBuf,
    mako_config: PathBuf,
    hypr_schema_file: PathBuf,
}

struct DotfilesCenter {
    paths: Paths,
    tab: Tab,
    values: HashMap<String, String>,
    shortcuts: Vec<Shortcut>,
    selected_shortcut: Option<usize>,
    shortcut_edit: Shortcut,
    hypr_schema: Vec<HyprOption>,
    hypr_values: HashMap<String, String>,
    status: String,
    no_show_on_startup: bool,
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 760.0])
            .with_min_inner_size([900.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Dotfiles Center",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(DotfilesCenter::new()))
        }),
    )
}

impl Paths {
    fn new() -> Self {
        let root = env::var_os("SPLINTERDOTS_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));

        let config_home = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".config"));

        let dotfiles_dir = config_home.join("dotfiles");
        let hypr_dir = config_home.join("hypr");
        let quickshell_dir = config_home.join("quickshell").join("splinterbar");

        Self {
            root: root.clone(),
            config_home: config_home.clone(),
            home: home.clone(),
            local_conf: dotfiles_dir.join("local.conf"),
            keybinds_json: dotfiles_dir.join("keybinds.json"),
            hypr_values_json: dotfiles_dir.join("hyprland-values.json"),
            disabled_file: dotfiles_dir.join("welcome-disabled"),
            dotfiles_dir,
            hypr_colors: hypr_dir.join("colors.conf"),
            hypr_keybinds: hypr_dir.join("keybindings.conf"),
            hypr_generated: hypr_dir.join("dotfiles-generated.conf"),
            hypr_dir,
            quickshell_file: quickshell_dir.join("shell.qml"),
            quickshell_dir,
            mako_config: config_home.join("mako").join("config"),
            hypr_schema_file: root.join("config").join("dotfiles").join("hyprland-options.json"),
        }
    }

    fn script(&self, name: &str) -> PathBuf {
        self.root.join("scripts").join(name)
    }

    fn repo_settings(&self) -> PathBuf {
        self.root.join("config").join("dotfiles").join("settings.conf")
    }
}

impl DotfilesCenter {
    fn new() -> Self {
        let paths = Paths::new();
        let values = read_settings(&paths);
        let shortcuts = load_shortcuts(&paths);
        let hypr_schema = load_hypr_schema(&paths);
        let hypr_values = load_hypr_values(&paths, &hypr_schema);
        let no_show_on_startup = paths.disabled_file.exists();

        Self {
            paths,
            tab: Tab::Overview,
            values,
            shortcuts,
            selected_shortcut: None,
            shortcut_edit: default_shortcut(),
            hypr_schema,
            hypr_values,
            status: String::new(),
            no_show_on_startup,
        }
    }

    fn ui_theme(&self, ctx: &egui::Context) {
        if self.value("DOTFILES_THEME") == "light" {
            ctx.set_visuals(egui::Visuals::light());
        } else {
            ctx.set_visuals(egui::Visuals::dark());
        }
    }

    fn value(&self, key: &str) -> String {
        self.values
            .get(key)
            .cloned()
            .unwrap_or_else(|| default_value(key))
    }

    fn set_value(&mut self, key: &str, value: String) {
        self.values.insert(key.to_string(), value);
    }

    fn bool_value(&self, key: &str) -> bool {
        is_true(&self.value(key))
    }

    fn set_bool_value(&mut self, key: &str, value: bool) {
        self.set_value(key, if value { "true" } else { "false" }.to_string());
    }

    fn save_all(&mut self) {
        let result = (|| -> Result<(), String> {
            write_local_conf(&self.paths, &self.values)?;
            save_shortcuts(&self.paths, &self.shortcuts)?;
            save_hypr_values(&self.paths, &self.hypr_values)?;
            write_colors(&self.paths, &self.values)?;
            write_keybinds(&self.paths, &self.values, &self.shortcuts)?;
            write_hyprland_settings(&self.paths, &self.hypr_schema, &self.hypr_values)?;
            write_quickshell_bar(&self.paths, &self.values)?;
            self.save_startup_choice()?;
            Ok(())
        })();

        self.status = match result {
            Ok(()) => "Saved.".to_string(),
            Err(err) => format!("Save failed: {err}"),
        };
    }

    fn save_startup_choice(&self) -> Result<(), String> {
        ensure_dir(&self.paths.dotfiles_dir)?;
        if self.no_show_on_startup {
            fs::write(&self.paths.disabled_file, "").map_err(err_string)?;
        } else if self.paths.disabled_file.exists() {
            fs::remove_file(&self.paths.disabled_file).map_err(err_string)?;
        }
        Ok(())
    }

    fn apply_wallpaper(&mut self) {
        let wallpaper = self.value("DOTFILES_WALLPAPER");
        let _ = write_local_conf(&self.paths, &self.values);

        let result = Command::new(self.paths.script("dotfiles-wallpaper"))
            .arg("set")
            .arg(wallpaper)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        self.status = match result {
            Ok(status) if status.success() => "Wallpaper applied.".to_string(),
            Ok(_) => "Wallpaper helper returned an error.".to_string(),
            Err(err) => format!("Could not run wallpaper helper: {err}"),
        };
    }

    fn restart_bar(&mut self) {
        let _ = Command::new("pkill")
            .args(["-x", "quickshell"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        let result = Command::new("quickshell")
            .args(["-c", "splinterbar"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        self.status = match result {
            Ok(_) => "QML bar restarted.".to_string(),
            Err(err) => format!("Could not restart QML bar: {err}"),
        };
    }

    fn save_and_restart_bar(&mut self) {
        self.save_all();
        self.restart_bar();
    }

    fn shortcut_hint_rows(&self) -> Vec<(String, String)> {
        let wanted = [
            ("App launcher", "Open apps"),
            ("Terminal", "Terminal"),
            ("Dotfiles Center", "Dotfiles Center"),
            ("Reload desktop", "Reload desktop"),
        ];

        let mut rows = Vec::new();
        for (name, label) in wanted {
            if let Some(sc) = self
                .shortcuts
                .iter()
                .find(|sc| sc.name.eq_ignore_ascii_case(name))
            {
                rows.push((format_shortcut_key(&sc.key), label.to_string()));
            }
        }

        rows.push(("Super + Left Mouse".to_string(), "Drag windows".to_string()));
        rows.push(("Super + Right Mouse".to_string(), "Resize windows".to_string()));
        rows.truncate(6);
        rows
    }

    fn tab_button(ui: &mut egui::Ui, selected: bool, label: &str) -> bool {
        let height = if selected { 38.0 } else { 30.0 };
        ui.add_sized(
            Vec2::new(128.0, height),
            egui::SelectableLabel::new(selected, RichText::new(label).strong()),
        )
        .clicked()
    }

    fn top_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if Self::tab_button(ui, self.tab == Tab::Overview, "Overview") {
                self.tab = Tab::Overview;
            }
            if Self::tab_button(ui, self.tab == Tab::Hyprland, "Hyprland") {
                self.tab = Tab::Hyprland;
            }
            if Self::tab_button(ui, self.tab == Tab::Shortcuts, "Shortcuts") {
                self.tab = Tab::Shortcuts;
            }
            if Self::tab_button(ui, self.tab == Tab::Appearance, "Appearance") {
                self.tab = Tab::Appearance;
            }
            if Self::tab_button(ui, self.tab == Tab::Bar, "QML Bar") {
                self.tab = Tab::Bar;
            }
            if Self::tab_button(ui, self.tab == Tab::Setup, "Setup") {
                self.tab = Tab::Setup;
            }
        });
    }

    fn overview_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Your Arch desktop, ready to shape");
        ui.label("Use switches and dropdowns instead of editing config files by hand.");
        ui.add_space(12.0);

        egui::Grid::new("overview-shortcuts")
            .num_columns(3)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                for (index, (key, label)) in self.shortcut_hint_rows().iter().enumerate() {
                    ui.group(|ui| {
                        ui.set_min_size(Vec2::new(210.0, 58.0));
                        ui.label(RichText::new(key).strong());
                        ui.label(label);
                    });
                    if index % 3 == 2 {
                        ui.end_row();
                    }
                }
            });

        ui.add_space(18.0);
        ui.heading("Installed components");
        for item in [
            "Hyprland · Quickshell QML bar · Wofi launcher · Mako notifications",
            "Kitty terminal · Thunar file manager · Zen Browser",
            "PipeWire audio · NetworkManager · Bluetooth",
            "Screenshot · Clipboard · Wallpaper · XDG portals · greetd login",
        ] {
            ui.label(format!("• {item}"));
        }

        ui.add_space(18.0);
        ui.heading("Helpful terminal actions");
        for (cmd, desc) in [
            ("dotctl all", "Update repo, install packages, apply dotfiles"),
            ("dotctl status", "Show linked files"),
            ("dotctl doctor", "Debug Hyprland and graphics issues"),
            ("dotctl center", "Open this window"),
        ] {
            ui.horizontal(|ui| {
                ui.monospace(cmd);
                ui.label(desc);
            });
        }
    }

    fn hyprland_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hyprland settings");
        ui.label("Switches for on/off, dropdowns for choices, and simple number controls for safe settings.");
        ui.add_space(8.0);

        let mut grouped: BTreeMap<String, Vec<HyprOption>> = BTreeMap::new();
        for option in &self.hypr_schema {
            grouped
                .entry(option.section.clone().unwrap_or_else(|| "Other".to_string()))
                .or_default()
                .push(option.clone());
        }

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (section, options) in grouped {
                egui::CollapsingHeader::new(RichText::new(section).strong())
                    .default_open(true)
                    .show(ui, |ui| {
                        for option in options {
                            self.hypr_option_row(ui, &option);
                        }
                    });
            }
        });
    }

    fn hypr_option_row(&mut self, ui: &mut egui::Ui, option: &HyprOption) {
        let path = option.path.clone();
        let label = option
            .label
            .clone()
            .unwrap_or_else(|| path.replace(':', " / ").replace('_', " "));

        let kind = option.kind.clone().unwrap_or_else(|| "text".to_string());
        let default = option.default.clone().unwrap_or_default();
        let current = self.hypr_values.entry(path.clone()).or_insert(default);

        ui.horizontal(|ui| {
            ui.set_min_height(28.0);
            ui.label(label);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| match kind.as_str() {
                "bool" => {
                    let mut checked = is_true(current);
                    if ui.checkbox(&mut checked, "").changed() {
                        *current = if checked { "true" } else { "false" }.to_string();
                    }
                }
                "choice" => {
                    let choices = option.choices.clone().unwrap_or_default();
                    egui::ComboBox::from_id_source(format!("hypr-{path}"))
                        .selected_text(current.clone())
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for choice in choices {
                                ui.selectable_value(current, choice.clone(), choice);
                            }
                        });
                }
                "int" => {
                    let min = option.min.unwrap_or(0.0) as i64;
                    let max = option.max.unwrap_or(9999.0) as i64;
                    let mut value = current.parse::<i64>().unwrap_or(min);
                    if ui.add(egui::DragValue::new(&mut value).clamp_range(min..=max)).changed() {
                        *current = value.to_string();
                    }
                }
                "float" => {
                    let min = option.min.unwrap_or(0.0);
                    let max = option.max.unwrap_or(9999.0);
                    let mut value = current.parse::<f64>().unwrap_or(min);
                    if ui.add(egui::DragValue::new(&mut value).speed(0.05).clamp_range(min..=max)).changed() {
                        *current = format!("{value:g}");
                    }
                }
                _ => {}
            });
        });
        ui.separator();
    }

    fn shortcuts_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Shortcuts");
        ui.label("All shortcuts use Super. Save changes, then reload Hyprland.");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if ui.button("Add shortcut").clicked() {
                self.shortcuts.push(Shortcut {
                    name: "New shortcut".to_string(),
                    key: "SHIFT, N".to_string(),
                    kind: "app".to_string(),
                    value: self.value("DOTFILES_TERMINAL"),
                });
                self.selected_shortcut = Some(self.shortcuts.len() - 1);
                self.shortcut_edit = self.shortcuts[self.shortcuts.len() - 1].clone();
            }

            if ui.button("Remove selected").clicked() {
                if let Some(index) = self.selected_shortcut {
                    if index < self.shortcuts.len() {
                        self.shortcuts.remove(index);
                        self.selected_shortcut = None;
                    }
                }
            }

            if ui.button("Restore defaults").clicked() {
                self.shortcuts = default_shortcuts();
                self.selected_shortcut = None;
            }
        });

        ui.add_space(8.0);

        ui.columns(2, |columns| {
            ScrollArea::vertical().max_height(420.0).show(&mut columns[0], |ui| {
                for (index, sc) in self.shortcuts.iter().enumerate() {
                    let selected = self.selected_shortcut == Some(index);
                    let text = format!("{}    {}", format_shortcut_key(&sc.key), sc.name);
                    if ui.selectable_label(selected, text).clicked() {
                        self.selected_shortcut = Some(index);
                        self.shortcut_edit = sc.clone();
                    }
                }
            });

            columns[1].group(|ui| {
                ui.heading("Selected shortcut");
                if let Some(index) = self.selected_shortcut {
                    if index < self.shortcuts.len() {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut self.shortcut_edit.name);

                        ui.label("Key");
                        ui.text_edit_singleline(&mut self.shortcut_edit.key);
                        ui.small("Examples: Return · D · SHIFT, S · CTRL, ALT, T");

                        ui.label("Type");
                        egui::ComboBox::from_id_source("shortcut-kind")
                            .selected_text(friendly_kind(&self.shortcut_edit.kind))
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(self.shortcut_edit.kind == "app", "App / script")
                                    .clicked()
                                {
                                    self.shortcut_edit.kind = "app".to_string();
                                }
                                if ui
                                    .selectable_label(self.shortcut_edit.kind == "desktop", "Desktop action")
                                    .clicked()
                                {
                                    self.shortcut_edit.kind = "desktop".to_string();
                                }
                            });

                        ui.label("What should happen");
                        ui.text_edit_singleline(&mut self.shortcut_edit.value);

                        if ui.button("Update selected shortcut").clicked() {
                            self.shortcuts[index] = self.shortcut_edit.clone();
                        }
                    }
                } else {
                    ui.label("Select a shortcut to edit it.");
                }
            });
        });

        ui.add_space(8.0);
        ui.label("Mouse controls: Super + left mouse drag moves windows. Super + right mouse drag resizes windows.");
    }

    fn appearance_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Appearance");

        ui.horizontal(|ui| {
            ui.label("Theme");
            let mut theme = self.value("DOTFILES_THEME");
            egui::ComboBox::from_id_source("theme")
                .selected_text(&theme)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut theme, "dark".to_string(), "dark");
                    ui.selectable_value(&mut theme, "light".to_string(), "light");
                });
            self.set_value("DOTFILES_THEME", theme);
        });

        ui.horizontal(|ui| {
            ui.label("Accent color");
            let mut accent = self.value("DOTFILES_ACCENT");
            ui.text_edit_singleline(&mut accent);
            self.set_value("DOTFILES_ACCENT", accent);
        });

        ui.horizontal(|ui| {
            ui.label("Wallpaper");
            let mut wallpaper = self.value("DOTFILES_WALLPAPER");
            ui.text_edit_singleline(&mut wallpaper);
            self.set_value("DOTFILES_WALLPAPER", wallpaper);
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Apply wallpaper").clicked() {
                self.apply_wallpaper();
            }
            if ui.button("Apply theme").clicked() {
                self.save_all();
            }
        });
    }

    fn bar_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("QML Bar");
        ScrollArea::vertical().show(ui, |ui| {
            ui.group(|ui| {
                ui.heading("Layout");
                combo_value(ui, &mut self.values, "DOTFILES_BAR_POSITION", &["top", "bottom"]);
            });

            ui.group(|ui| {
                ui.heading("Modules");
                for (label, key) in [
                    ("Workspaces", "DOTFILES_BAR_SHOW_WORKSPACES"),
                    ("Clock", "DOTFILES_BAR_SHOW_CLOCK"),
                    ("Volume", "DOTFILES_BAR_SHOW_VOLUME"),
                    ("Network", "DOTFILES_BAR_SHOW_NETWORK"),
                    ("Battery", "DOTFILES_BAR_SHOW_BATTERY"),
                    ("CPU usage", "DOTFILES_BAR_SHOW_CPU"),
                    ("Memory usage", "DOTFILES_BAR_SHOW_MEMORY"),
                ] {
                    let mut checked = self.bool_value(key);
                    if ui.checkbox(&mut checked, label).changed() {
                        self.set_bool_value(key, checked);
                    }
                }
            });

            ui.group(|ui| {
                ui.heading("Clock");
                ui.label("Format");
                let mut fmt = self.value("DOTFILES_BAR_CLOCK_FORMAT");
                ui.text_edit_singleline(&mut fmt);
                self.set_value("DOTFILES_BAR_CLOCK_FORMAT", fmt);

                let mut seconds = self.bool_value("DOTFILES_BAR_CLOCK_SECONDS");
                if ui.checkbox(&mut seconds, "Show seconds").changed() {
                    self.set_bool_value("DOTFILES_BAR_CLOCK_SECONDS", seconds);
                }
            });

            ui.group(|ui| {
                ui.heading("Workspaces");
                combo_value(
                    ui,
                    &mut self.values,
                    "DOTFILES_BAR_WORKSPACE_COUNT",
                    &["5", "6", "7", "8", "9", "10", "12", "15", "20"],
                );

                let mut accent = self.bool_value("DOTFILES_BAR_ACCENT_WORKSPACE");
                if ui.checkbox(&mut accent, "Accent active workspace").changed() {
                    self.set_bool_value("DOTFILES_BAR_ACCENT_WORKSPACE", accent);
                }
            });

            ui.group(|ui| {
                ui.heading("Sizing and style");
                slider_value(ui, &mut self.values, "DOTFILES_BAR_HEIGHT", 24.0, 72.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_RADIUS", 0.0, 28.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_OPACITY", 0.2, 1.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_FONT_SIZE", 8.0, 24.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_SPACING", 4.0, 40.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_BORDER_WIDTH", 0.0, 4.0);
            });

            ui.horizontal(|ui| {
                if ui.button("Save and restart bar").clicked() {
                    self.save_and_restart_bar();
                }
                if ui.button("Restart bar only").clicked() {
                    self.restart_bar();
                }
            });
        });
    }

    fn setup_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Setup");
        for (label, key) in [
            ("Terminal", "DOTFILES_TERMINAL"),
            ("File manager", "DOTFILES_FILE_MANAGER"),
            ("App launcher", "DOTFILES_APP_LAUNCHER"),
            ("Editor", "DOTFILES_EDITOR"),
            ("Browser", "DOTFILES_BROWSER"),
        ] {
            ui.horizontal(|ui| {
                ui.label(label);
                let mut value = self.value(key);
                ui.text_edit_singleline(&mut value);
                self.set_value(key, value);
            });
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("Open dotfiles folder").clicked() {
                run_quiet("xdg-open", &[self.paths.root.to_string_lossy().as_ref()]);
            }
            if ui.button("Reload Hyprland").clicked() {
                run_quiet("hyprctl", &["reload"]);
            }
        });
    }
}

impl eframe::App for DotfilesCenter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui_theme(ctx);

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.heading("Dotfiles Center");
            ui.label("Hyprland · Shortcuts · Appearance · QML Bar · Setup");
            ui.add_space(8.0);
            self.top_tabs(ui);
            ui.add_space(6.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Overview => self.overview_tab(ui),
            Tab::Hyprland => self.hyprland_tab(ui),
            Tab::Shortcuts => self.shortcuts_tab(ui),
            Tab::Appearance => self.appearance_tab(ui),
            Tab::Bar => self.bar_tab(ui),
            Tab::Setup => self.setup_tab(ui),
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.no_show_on_startup, "Don't show on startup");
                if ui.button(RichText::new("Save changes").strong()).clicked() {
                    self.save_all();
                }
                if !self.status.is_empty() {
                    ui.label(RichText::new(&self.status).color(Color32::LIGHT_GREEN));
                }
            });
        });
    }
}

fn read_settings(paths: &Paths) -> HashMap<String, String> {
    let mut values = defaults_map();
    load_shell_values(&paths.repo_settings(), &mut values);
    load_shell_values(&paths.local_conf, &mut values);
    values
}

fn defaults_map() -> HashMap<String, String> {
    SETTINGS_KEYS
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

fn default_value(key: &str) -> String {
    SETTINGS_KEYS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| (*v).to_string())
        .unwrap_or_default()
}

fn load_shell_values(path: &Path, values: &mut HashMap<String, String>) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || !trimmed.contains('=') {
            continue;
        }

        let Some((key, raw)) = trimmed.split_once('=') else {
            continue;
        };

        let key = key.trim();
        if key.is_empty() {
            continue;
        }

        let mut value = raw.trim().to_string();
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value = value[1..value.len() - 1]
                .replace("\\\"", "\"")
                .replace("\\\\", "\\");
        }

        values.insert(key.to_string(), value);
    }
}

fn write_local_conf(paths: &Paths, values: &HashMap<String, String>) -> Result<(), String> {
    ensure_dir(&paths.dotfiles_dir)?;

    let mut lines = vec![
        "# Machine-local dotfiles settings written by Dotfiles Center.".to_string(),
        "# It is safe to edit this file manually too.".to_string(),
    ];

    for (key, default) in SETTINGS_KEYS {
        let value = values.get(*key).cloned().unwrap_or_else(|| (*default).to_string());
        lines.push(format!("{key}={}", shell_quote(&value)));
    }

    fs::write(&paths.local_conf, lines.join("\n") + "\n").map_err(err_string)
}

fn shell_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn default_shortcut() -> Shortcut {
    Shortcut {
        name: "New shortcut".to_string(),
        key: "SHIFT, N".to_string(),
        kind: "app".to_string(),
        value: "kitty".to_string(),
    }
}

fn default_shortcuts() -> Vec<Shortcut> {
    vec![
        sc("Terminal", "Return", "app", "$terminal"),
        sc("File manager", "E", "app", "$fileManager"),
        sc("App launcher", "D", "app", "$menu"),
        sc("Browser", "B", "app", "$browser"),
        sc("Close window", "C", "desktop", "killactive"),
        sc("Exit desktop", "M", "desktop", "exit"),
        sc("Fullscreen", "F", "desktop", "fullscreen"),
        sc("Floating mode", "V", "desktop", "togglefloating"),
        sc("Split direction", "J", "desktop", "layoutmsg, togglesplit"),
        sc("Screenshot region", "S", "app", "dotfiles-screenshot region"),
        sc("Screenshot full screen", "SHIFT, S", "app", "dotfiles-screenshot full"),
        sc("Dotfiles Center", "W", "app", "dotctl center"),
        sc("Reload desktop", "SHIFT, R", "app", "hyprctl reload"),
    ]
}

fn sc(name: &str, key: &str, kind: &str, value: &str) -> Shortcut {
    Shortcut {
        name: name.to_string(),
        key: key.to_string(),
        kind: kind.to_string(),
        value: value.to_string(),
    }
}

fn load_shortcuts(paths: &Paths) -> Vec<Shortcut> {
    let Ok(content) = fs::read_to_string(&paths.keybinds_json) else {
        return default_shortcuts();
    };

    serde_json::from_str::<Vec<Shortcut>>(&content).unwrap_or_else(|_| default_shortcuts())
}

fn save_shortcuts(paths: &Paths, shortcuts: &[Shortcut]) -> Result<(), String> {
    ensure_dir(&paths.dotfiles_dir)?;
    let text = serde_json::to_string_pretty(shortcuts).map_err(err_string)?;
    fs::write(&paths.keybinds_json, text + "\n").map_err(err_string)
}

fn load_hypr_schema(paths: &Paths) -> Vec<HyprOption> {
    let Ok(content) = fs::read_to_string(&paths.hypr_schema_file) else {
        return Vec::new();
    };

    let Ok(schema) = serde_json::from_str::<HyprSchemaFile>(&content) else {
        return Vec::new();
    };

    schema
        .options
        .into_iter()
        .filter(|option| {
            let kind = option.kind.as_deref().unwrap_or("text");
            matches!(kind, "bool" | "choice" | "int" | "float")
                && !EXCLUDED_HYPR_OPTIONS.contains(&option.path.as_str())
        })
        .collect()
}

fn load_hypr_values(paths: &Paths, schema: &[HyprOption]) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for option in schema {
        values.insert(
            option.path.clone(),
            option.default.clone().unwrap_or_default(),
        );
    }

    if let Ok(content) = fs::read_to_string(&paths.hypr_values_json) {
        if let Ok(saved) = serde_json::from_str::<HashMap<String, String>>(&content) {
            for (key, value) in saved {
                values.insert(key, value);
            }
        }
    }

    values
}

fn save_hypr_values(paths: &Paths, values: &HashMap<String, String>) -> Result<(), String> {
    ensure_dir(&paths.dotfiles_dir)?;
    let text = serde_json::to_string_pretty(values).map_err(err_string)?;
    fs::write(&paths.hypr_values_json, text + "\n").map_err(err_string)
}

fn write_keybinds(
    paths: &Paths,
    values: &HashMap<String, String>,
    shortcuts: &[Shortcut],
) -> Result<(), String> {
    ensure_dir(&paths.hypr_dir)?;

    let mut lines = vec![
        "$mainMod = SUPER".to_string(),
        format!("$terminal = {}", value_or(values, "DOTFILES_TERMINAL")),
        format!("$fileManager = {}", value_or(values, "DOTFILES_FILE_MANAGER")),
        format!("$menu = {}", value_or(values, "DOTFILES_APP_LAUNCHER")),
        format!("$browser = {}", value_or(values, "DOTFILES_BROWSER")),
        String::new(),
        "# Hold Super + left mouse button to drag windows.".to_string(),
        "bindm = $mainMod, mouse:272, movewindow".to_string(),
        "# Hold Super + right mouse button to resize windows.".to_string(),
        "bindm = $mainMod, mouse:273, resizewindow".to_string(),
        String::new(),
    ];

    for shortcut in shortcuts {
        let (modifier, key) = hypr_key_parts(&shortcut.key);
        if shortcut.kind == "app" {
            lines.push(format!("bind = {modifier}, {key}, exec, {}", shortcut.value));
        } else {
            lines.push(format!("bind = {modifier}, {key}, {}", shortcut.value));
        }
    }

    lines.extend([
        "".to_string(),
        "bind = $mainMod, left, movefocus, l".to_string(),
        "bind = $mainMod, right, movefocus, r".to_string(),
        "bind = $mainMod, up, movefocus, u".to_string(),
        "bind = $mainMod, down, movefocus, d".to_string(),
        "".to_string(),
        "bind = $mainMod SHIFT, left, movewindow, l".to_string(),
        "bind = $mainMod SHIFT, right, movewindow, r".to_string(),
        "bind = $mainMod SHIFT, up, movewindow, u".to_string(),
        "bind = $mainMod SHIFT, down, movewindow, d".to_string(),
        "".to_string(),
    ]);

    for i in 1..=9 {
        lines.push(format!("bind = $mainMod, {i}, workspace, {i}"));
    }
    lines.push(String::new());
    for i in 1..=9 {
        lines.push(format!("bind = $mainMod SHIFT, {i}, movetoworkspace, {i}"));
    }

    fs::write(&paths.hypr_keybinds, lines.join("\n") + "\n").map_err(err_string)
}

fn hypr_key_parts(key: &str) -> (String, String) {
    let key = key.trim();
    if key.to_uppercase().starts_with("SHIFT,") {
        (
            "$mainMod SHIFT".to_string(),
            key.split_once(',').map(|(_, k)| k.trim()).unwrap_or("").to_string(),
        )
    } else {
        ("$mainMod".to_string(), key.to_string())
    }
}

fn write_hyprland_settings(
    paths: &Paths,
    schema: &[HyprOption],
    values: &HashMap<String, String>,
) -> Result<(), String> {
    ensure_dir(&paths.hypr_dir)?;

    let mut root = HyprNode::default();

    for option in schema {
        if !hypr_option_supported(&option.path) {
            continue;
        }

        let raw = values
            .get(&option.path)
            .cloned()
            .or_else(|| option.default.clone())
            .unwrap_or_default();

        let value = clean_hypr_value(option, &raw);
        if value.is_empty() {
            continue;
        }

        insert_hypr_value(&mut root, &option.path, value);
    }

    let mut lines = vec![
        "# Generated by Dotfiles Center.".to_string(),
        "# This file is safe to regenerate.".to_string(),
        String::new(),
    ];
    render_hypr_node(&root, 0, &mut lines);

    fs::write(&paths.hypr_generated, lines.join("\n") + "\n").map_err(err_string)
}

fn hypr_option_supported(path: &str) -> bool {
    if env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
        return true;
    }

    Command::new("hyprctl")
        .args(["getoption", path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(true)
}

fn insert_hypr_value(root: &mut HyprNode, path: &str, value: String) {
    let mut current = root;
    for part in path.split(':') {
        current = current.children.entry(part.to_string()).or_default();
    }
    current.value = Some(value);
}

fn render_hypr_node(node: &HyprNode, indent: usize, lines: &mut Vec<String>) {
    for (key, child) in &node.children {
        let prefix = " ".repeat(indent);
        if child.children.is_empty() {
            if let Some(value) = &child.value {
                lines.push(format!("{prefix}{key} = {value}"));
            }
        } else {
            lines.push(format!("{prefix}{key} {{"));
            render_hypr_node(child, indent + 4, lines);
            lines.push(format!("{prefix}}}"));
        }
    }
}

fn clean_hypr_value(option: &HyprOption, value: &str) -> String {
    let kind = option.kind.as_deref().unwrap_or("text");
    match kind {
        "bool" => {
            if is_true(value) {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        "choice" => {
            let choices = option.choices.clone().unwrap_or_default();
            if choices.contains(&value.to_string()) {
                value.to_string()
            } else {
                option.default.clone().unwrap_or_default()
            }
        }
        "int" => {
            let min = option.min.unwrap_or(0.0) as i64;
            let max = option.max.unwrap_or(9999.0) as i64;
            let number = value.parse::<i64>().unwrap_or(min).clamp(min, max);
            number.to_string()
        }
        "float" => {
            let min = option.min.unwrap_or(0.0);
            let max = option.max.unwrap_or(9999.0);
            let number = value.parse::<f64>().unwrap_or(min).clamp(min, max);
            format!("{number:g}")
        }
        _ => value.trim().to_string(),
    }
}

fn write_colors(paths: &Paths, values: &HashMap<String, String>) -> Result<(), String> {
    ensure_dir(&paths.hypr_dir)?;
    ensure_parent(&paths.mako_config)?;

    let palette = theme_palette(values);

    let colors = format!(
        "$accent = {}\n$inactive = {}\n$background = {}\n$text = {}\n",
        hex_to_hypr_rgba(&palette.accent),
        hex_to_hypr_rgba(&palette.inactive_border),
        hex_to_hypr_rgba(&palette.background),
        hex_to_hypr_rgba(&palette.text),
    );

    fs::write(&paths.hypr_colors, colors).map_err(err_string)?;

    let mako = format!(
        "background-color={}\ntext-color={}\nborder-color={}\nborder-size=2\nborder-radius=8\npadding=12\ndefault-timeout=5000\n",
        palette.background, palette.text, palette.accent,
    );
    fs::write(&paths.mako_config, mako).map_err(err_string)
}

struct Palette {
    accent: String,
    background: String,
    surface: String,
    surface_alt: String,
    text: String,
    muted: String,
    inactive_border: String,
    bar_rgb: String,
    active_text: String,
}

fn theme_palette(values: &HashMap<String, String>) -> Palette {
    let accent = value_or(values, "DOTFILES_ACCENT");
    if value_or(values, "DOTFILES_THEME") == "light" {
        Palette {
            accent,
            background: "#f8fafc".to_string(),
            surface: "#e2e8f0".to_string(),
            surface_alt: "#cbd5e1".to_string(),
            text: "#0f172a".to_string(),
            muted: "#475569".to_string(),
            inactive_border: "#94a3b8".to_string(),
            bar_rgb: "f8fafc".to_string(),
            active_text: "#ffffff".to_string(),
        }
    } else {
        Palette {
            accent,
            background: "#1e1e2e".to_string(),
            surface: "#313244".to_string(),
            surface_alt: "#45475a".to_string(),
            text: "#cdd6f4".to_string(),
            muted: "#bac2de".to_string(),
            inactive_border: "#45475a".to_string(),
            bar_rgb: "1e1e2e".to_string(),
            active_text: "#11111b".to_string(),
        }
    }
}

fn hex_to_hypr_rgba(hex: &str) -> String {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
        format!("rgba({}ff)", clean.to_lowercase())
    } else {
        "rgba(89b4faff)".to_string()
    }
}

fn write_quickshell_bar(paths: &Paths, values: &HashMap<String, String>) -> Result<(), String> {
    ensure_dir(&paths.quickshell_dir)?;

    let palette = theme_palette(values);
    let position = value_or(values, "DOTFILES_BAR_POSITION");
    let top = if position == "top" { "true" } else { "false" };
    let bottom = if position == "bottom" { "true" } else { "false" };

    let height = clamp_i(value_or(values, "DOTFILES_BAR_HEIGHT"), 34, 24, 72);
    let radius = clamp_i(value_or(values, "DOTFILES_BAR_RADIUS"), 8, 0, 28);
    let opacity = clamp_f(value_or(values, "DOTFILES_BAR_OPACITY"), 0.92, 0.2, 1.0);
    let font_size = clamp_i(value_or(values, "DOTFILES_BAR_FONT_SIZE"), 12, 8, 24);
    let spacing = clamp_i(value_or(values, "DOTFILES_BAR_SPACING"), 14, 4, 40);
    let border_width = clamp_i(value_or(values, "DOTFILES_BAR_BORDER_WIDTH"), 1, 0, 4);
    let workspace_count = clamp_i(value_or(values, "DOTFILES_BAR_WORKSPACE_COUNT"), 9, 1, 20);

    let mut clock_format = value_or(values, "DOTFILES_BAR_CLOCK_FORMAT");
    if is_true(&value_or(values, "DOTFILES_BAR_CLOCK_SECONDS")) && !clock_format.contains("%S") {
        clock_format.push_str(":%S");
    }

    let bg_alpha = (opacity * 255.0) as i32;
    let bg_color = format!("#{:02x}{}", bg_alpha, palette.bar_rgb);

    let show_workspaces = is_true(&value_or(values, "DOTFILES_BAR_SHOW_WORKSPACES"));
    let show_clock = is_true(&value_or(values, "DOTFILES_BAR_SHOW_CLOCK"));
    let show_volume = is_true(&value_or(values, "DOTFILES_BAR_SHOW_VOLUME"));
    let show_network = is_true(&value_or(values, "DOTFILES_BAR_SHOW_NETWORK"));
    let show_battery = is_true(&value_or(values, "DOTFILES_BAR_SHOW_BATTERY"));
    let show_cpu = is_true(&value_or(values, "DOTFILES_BAR_SHOW_CPU"));
    let show_memory = is_true(&value_or(values, "DOTFILES_BAR_SHOW_MEMORY"));
    let accent_workspace = is_true(&value_or(values, "DOTFILES_BAR_ACCENT_WORKSPACE"));

    let mut status_parts = Vec::new();
    if show_volume {
        status_parts.push("printf 'VOL '; wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null | awk '{v=int($2*100); if($3==\"[MUTED]\") print \"MUTE\"; else print v\"%\"}'");
    }
    if show_network {
        status_parts.push("printf '  NET '; nmcli -t -f GENERAL.STATE device show 2>/dev/null | grep -q ':100' && echo on || echo off");
    }
    if show_battery {
        status_parts.push("bat=$(cat /sys/class/power_supply/BAT0/capacity 2>/dev/null || cat /sys/class/power_supply/BAT1/capacity 2>/dev/null); [ -n \"$bat\" ] && printf '  BAT %s%%' \"$bat\"");
    }
    if show_cpu {
        status_parts.push("cpu=$(top -bn1 | grep 'Cpu(s)' | awk '{print int($2+$4)\"%\"}' 2>/dev/null); [ -n \"$cpu\" ] && printf '  CPU %s' \"$cpu\"");
    }
    if show_memory {
        status_parts.push("mem=$(free -m | awk '/^Mem/{printf \"%dMB\", $3}' 2>/dev/null); [ -n \"$mem\" ] && printf '  MEM %s' \"$mem\"");
    }

    let status_cmd = if status_parts.is_empty() {
        "echo ''".to_string()
    } else {
        status_parts.join("; ")
    };

    let left_section = if show_workspaces {
        let active_color = if accent_workspace {
            palette.accent.clone()
        } else {
            palette.surface.clone()
        };
        format!(
            r#"
          Row {{
            Layout.alignment: Qt.AlignVCenter
            spacing: 6

            Repeater {{
              model: {workspace_count}
              Rectangle {{
                width: (index + 1) === root.activeWorkspace ? {active_w} : {inactive_w}
                height: (index + 1) === root.activeWorkspace ? {active_h} : {inactive_h}
                anchors.verticalCenter: parent.verticalCenter
                radius: {button_radius}
                color: (index + 1) === root.activeWorkspace ? "{active_color}" : "{surface}"
                opacity: (index + 1) === root.activeWorkspace ? 1.0 : 0.72

                Text {{
                  anchors.centerIn: parent
                  text: index + 1
                  color: (index + 1) === root.activeWorkspace ? "{active_text}" : "{text}"
                  font.bold: true
                  font.pixelSize: {workspace_font}
                }}

                MouseArea {{
                  anchors.fill: parent
                  onClicked: switchWorkspace.running = true
                  hoverEnabled: true
                  onEntered: parent.opacity = 1.0
                  onExited: parent.opacity = (index + 1) === root.activeWorkspace ? 1.0 : 0.72
                }}

                Process {{
                  id: switchWorkspace
                  command: ["hyprctl", "dispatch", "workspace", String(index + 1)]
                }}
              }}
            }}
          }}"#,
            active_w = height - 2,
            inactive_w = height - 14,
            active_h = height - 2,
            inactive_h = height - 14,
            button_radius = (radius - 4).max(3),
            surface = palette.surface,
            text = palette.text,
            active_text = palette.active_text,
            workspace_font = font_size - 1,
        )
    } else {
        String::new()
    };

    let center_section = if show_clock {
        format!(
            r#"
          Text {{
            id: clock
            Layout.alignment: Qt.AlignCenter
            color: "{text}"
            font.bold: true
            font.pixelSize: {font_size}
            text: "Loading..."

            Process {{
              id: dateProc
              command: ["date", {clock_arg}]
              running: true
              stdout: StdioCollector {{
                onStreamFinished: clock.text = this.text.trim()
              }}
            }}

            Timer {{
              interval: 1000
              running: true
              repeat: true
              onTriggered: dateProc.running = true
            }}
          }}"#,
            text = palette.text,
            clock_arg = json_string(&format!("+{clock_format}")),
        )
    } else {
        String::new()
    };

    let right_section = if show_volume || show_network || show_battery || show_cpu || show_memory {
        format!(
            r#"
          Text {{
            id: status
            Layout.alignment: Qt.AlignRight | Qt.AlignVCenter
            color: "{muted}"
            font.pixelSize: {font_size}
            text: ""

            Process {{
              id: statusProc
              command: ["sh", "-c", {status_cmd}]
              running: true
              stdout: StdioCollector {{
                onStreamFinished: status.text = this.text.replace(/\n/g, "").trim()
              }}
            }}

            Timer {{
              interval: 3000
              running: true
              repeat: true
              onTriggered: statusProc.running = true
            }}
          }}"#,
            muted = palette.muted,
            status_cmd = json_string(&status_cmd),
        )
    } else {
        String::new()
    };

    let template = r#"// SplinterDots Quickshell bar.
// Generated by Dotfiles Center.

import Quickshell
import Quickshell.Io
import QtQuick
import QtQuick.Layouts

Variants {
  model: Quickshell.screens

  delegate: Component {
    PanelWindow {
      id: root
      property int activeWorkspace: 1
      required property var modelData
      screen: modelData

      anchors {
        top: __TOP__
        bottom: __BOTTOM__
        left: true
        right: true
      }

      margins {
        top: __TOP_MARGIN__
        bottom: __BOTTOM_MARGIN__
        left: 10
        right: 10
      }

      implicitHeight: __HEIGHT__
      color: "transparent"

      Process {
        id: activeWorkspaceProc
        command: ["hyprctl", "activeworkspace", "-j"]
        running: true
        stdout: StdioCollector {
          onStreamFinished: {
            try {
              root.activeWorkspace = JSON.parse(this.text).id
            } catch (e) {
            }
          }
        }
      }

      Timer {
        interval: 700
        running: true
        repeat: true
        onTriggered: activeWorkspaceProc.running = true
      }

      Rectangle {
        anchors.fill: parent
        radius: __RADIUS__
        color: "__BG_COLOR__"
        border.width: __BORDER_WIDTH__
        border.color: "__BORDER_COLOR__"

        RowLayout {
          anchors.fill: parent
          anchors.leftMargin: __SPACING__
          anchors.rightMargin: __SPACING__
          spacing: __SPACING__
__LEFT_SECTION__
          Item { Layout.fillWidth: true }
__CENTER_SECTION__
          Item { Layout.fillWidth: true }
__RIGHT_SECTION__
        }
      }
    }
  }
}
"#;

    let qml = template
        .replace("__TOP__", top)
        .replace("__BOTTOM__", bottom)
        .replace("__TOP_MARGIN__", if position == "top" { "8" } else { "0" })
        .replace("__BOTTOM_MARGIN__", if position == "bottom" { "8" } else { "0" })
        .replace("__HEIGHT__", &height.to_string())
        .replace("__RADIUS__", &radius.to_string())
        .replace("__BG_COLOR__", &bg_color)
        .replace("__BORDER_WIDTH__", &border_width.to_string())
        .replace("__BORDER_COLOR__", &palette.surface)
        .replace("__SPACING__", &spacing.to_string())
        .replace("__LEFT_SECTION__", &left_section)
        .replace("__CENTER_SECTION__", &center_section)
        .replace("__RIGHT_SECTION__", &right_section);

    fs::write(&paths.quickshell_file, qml).map_err(err_string)
}

fn value_or(values: &HashMap<String, String>, key: &str) -> String {
    values.get(key).cloned().unwrap_or_else(|| default_value(key))
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

fn is_true(value: &str) -> bool {
    matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

fn friendly_kind(kind: &str) -> &'static str {
    if kind == "app" {
        "App / script"
    } else {
        "Desktop action"
    }
}

fn format_shortcut_key(key: &str) -> String {
    let parts: Vec<String> = key
        .split(',')
        .map(|part| match part.trim().to_ascii_uppercase().as_str() {
            "SHIFT" => "Shift".to_string(),
            "CTRL" => "Ctrl".to_string(),
            "ALT" => "Alt".to_string(),
            _ => part.trim().to_string(),
        })
        .filter(|part| !part.is_empty())
        .collect();

    if parts.is_empty() {
        "Super".to_string()
    } else {
        format!("Super + {}", parts.join(" + "))
    }
}

fn combo_value(ui: &mut egui::Ui, values: &mut HashMap<String, String>, key: &str, choices: &[&str]) {
    let mut current = values.get(key).cloned().unwrap_or_else(|| default_value(key));
    ui.horizontal(|ui| {
        ui.label(label_from_key(key));
        egui::ComboBox::from_id_source(key)
            .selected_text(&current)
            .show_ui(ui, |ui| {
                for choice in choices {
                    ui.selectable_value(&mut current, (*choice).to_string(), *choice);
                }
            });
    });
    values.insert(key.to_string(), current);
}

fn slider_value(ui: &mut egui::Ui, values: &mut HashMap<String, String>, key: &str, min: f64, max: f64) {
    let mut current = values
        .get(key)
        .cloned()
        .unwrap_or_else(|| default_value(key))
        .parse::<f64>()
        .unwrap_or(min);

    if ui
        .add(egui::Slider::new(&mut current, min..=max).text(label_from_key(key)))
        .changed()
    {
        values.insert(key.to_string(), format!("{current:g}"));
    }
}

fn label_from_key(key: &str) -> String {
    key.trim_start_matches("DOTFILES_")
        .trim_start_matches("BAR_")
        .replace('_', " ")
        .to_ascii_lowercase()
}

fn clamp_i(value: String, default: i32, min: i32, max: i32) -> i32 {
    value.parse::<i32>().unwrap_or(default).clamp(min, max)
}

fn clamp_f(value: String, default: f64, min: f64, max: f64) -> f64 {
    value.parse::<f64>().unwrap_or(default).clamp(min, max)
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(err_string)
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)
    } else {
        Ok(())
    }
}

fn run_quiet(program: &str, args: &[&str]) {
    let _ = Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn err_string<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}
