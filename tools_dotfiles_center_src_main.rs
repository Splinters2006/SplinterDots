use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, ScrollArea, Stroke,
    Vec2,
};
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
    ("DOTFILES_BAR_RADIUS", "10"),
    ("DOTFILES_BAR_OPACITY", "0.92"),
    ("DOTFILES_BAR_FONT_SIZE", "12"),
    ("DOTFILES_BAR_SPACING", "14"),
    ("DOTFILES_BAR_BORDER_WIDTH", "1"),
    ("DOTFILES_BAR_WORKSPACE_COUNT", "9"),
    ("DOTFILES_BAR_REACTIVE_MS", "120"),
    ("DOTFILES_BAR_STATUS_MS", "1500"),
    ("DOTFILES_BAR_ICON_PACK", "nerd"),
    ("DOTFILES_BAR_ICON_FONT", "Symbols Nerd Font"),

    ("DOTFILES_BAR_SHOW_WORKSPACES", "true"),
    ("DOTFILES_BAR_SHOW_CLOCK", "true"),
    ("DOTFILES_BAR_SHOW_VOLUME", "true"),
    ("DOTFILES_BAR_SHOW_NETWORK", "true"),
    ("DOTFILES_BAR_SHOW_BATTERY", "true"),
    ("DOTFILES_BAR_SHOW_CPU", "false"),
    ("DOTFILES_BAR_SHOW_MEMORY", "false"),
    ("DOTFILES_BAR_SHOW_TEMP", "false"),
    ("DOTFILES_BAR_SHOW_DISK", "false"),
    ("DOTFILES_BAR_SHOW_BRIGHTNESS", "false"),
    ("DOTFILES_BAR_SHOW_BLUETOOTH", "false"),
    ("DOTFILES_BAR_SHOW_MEDIA", "false"),
    ("DOTFILES_BAR_SHOW_UPDATES", "false"),
    ("DOTFILES_BAR_SHOW_KEYBOARD", "false"),

    ("DOTFILES_WIDGET_CLOCK_FORMAT", "%a %d %b  %H:%M"),
    ("DOTFILES_WIDGET_CLOCK_SECONDS", "false"),
    ("DOTFILES_WIDGET_VOLUME_DEVICE", "@DEFAULT_AUDIO_SINK@"),
    ("DOTFILES_WIDGET_NETWORK_STYLE", "short"),
    ("DOTFILES_WIDGET_BATTERY_LOW", "20"),
    ("DOTFILES_WIDGET_CPU_LABEL", "CPU"),
    ("DOTFILES_WIDGET_MEMORY_LABEL", "RAM"),
    ("DOTFILES_WIDGET_TEMP_SENSOR", ""),
    ("DOTFILES_WIDGET_DISK_PATH", "/"),
    ("DOTFILES_WIDGET_BRIGHTNESS_DEVICE", ""),
    ("DOTFILES_WIDGET_MEDIA_LENGTH", "28"),
    ("DOTFILES_WIDGET_UPDATES_COMMAND", "checkupdates 2>/dev/null | wc -l"),
    ("DOTFILES_WIDGET_KEYBOARD_LABEL", "KB"),
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
    Widgets,
    Setup,
}

impl Tab {
    fn title(self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Hyprland => "Hyprland",
            Tab::Shortcuts => "Shortcuts",
            Tab::Appearance => "Appearance",
            Tab::Bar => "QML Bar",
            Tab::Widgets => "Widgets",
            Tab::Setup => "Setup",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Tab::Overview => "Your desktop at a glance",
            Tab::Hyprland => "Safe visual and behavior settings",
            Tab::Shortcuts => "Keyboard and mouse controls",
            Tab::Appearance => "Theme, accent, and wallpaper",
            Tab::Bar => "Bar layout, icons, and speed",
            Tab::Widgets => "Choose and configure every bar widget",
            Tab::Setup => "Default apps and helper actions",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Tab::Overview => "⌂",
            Tab::Hyprland => "◇",
            Tab::Shortcuts => "⌘",
            Tab::Appearance => "◐",
            Tab::Bar => "▣",
            Tab::Widgets => "✦",
            Tab::Setup => "⚙",
        }
    }
}

#[derive(Clone)]
struct AppTheme {
    accent: Color32,
    bg: Color32,
    sidebar: Color32,
    panel: Color32,
    panel_soft: Color32,
    card: Color32,
    card_hover: Color32,
    text: Color32,
    muted: Color32,
    border: Color32,
    success: Color32,
    danger: Color32,
}

struct Paths {
    root: PathBuf,
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

struct SplinterDots {
    paths: Paths,
    tab: Tab,
    values: HashMap<String, String>,
    shortcuts: Vec<Shortcut>,
    selected_shortcut: Option<usize>,
    hypr_schema: Vec<HyprOption>,
    hypr_values: HashMap<String, String>,
    status: String,
    no_show_on_startup: bool,
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1160.0, 780.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "SplinterDots",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(SplinterDots::new()))
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

impl SplinterDots {
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
            hypr_schema,
            hypr_values,
            status: String::new(),
            no_show_on_startup,
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

    fn app_theme(&self) -> AppTheme {
        let accent = parse_hex_color(&self.value("DOTFILES_ACCENT"))
            .unwrap_or(Color32::from_rgb(137, 180, 250));

        if self.value("DOTFILES_THEME") == "light" {
            AppTheme {
                accent,
                bg: Color32::from_rgb(238, 242, 248),
                sidebar: Color32::from_rgb(248, 250, 252),
                panel: Color32::from_rgb(255, 255, 255),
                panel_soft: Color32::from_rgb(241, 245, 249),
                card: Color32::from_rgb(255, 255, 255),
                card_hover: Color32::from_rgb(239, 246, 255),
                text: Color32::from_rgb(15, 23, 42),
                muted: Color32::from_rgb(100, 116, 139),
                border: Color32::from_rgb(203, 213, 225),
                success: Color32::from_rgb(22, 163, 74),
                danger: Color32::from_rgb(220, 38, 38),
            }
        } else {
            AppTheme {
                accent,
                bg: Color32::from_rgb(10, 12, 18),
                sidebar: Color32::from_rgb(16, 19, 29),
                panel: Color32::from_rgb(20, 24, 36),
                panel_soft: Color32::from_rgb(27, 32, 48),
                card: Color32::from_rgb(30, 36, 54),
                card_hover: Color32::from_rgb(38, 46, 68),
                text: Color32::from_rgb(226, 232, 240),
                muted: Color32::from_rgb(148, 163, 184),
                border: Color32::from_rgb(51, 65, 85),
                success: Color32::from_rgb(134, 239, 172),
                danger: Color32::from_rgb(251, 113, 133),
            }
        }
    }

    fn apply_style(&self, ctx: &egui::Context) {
        let theme = self.app_theme();

        let mut visuals = if self.value("DOTFILES_THEME") == "light" {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };

        visuals.override_text_color = Some(theme.text);
        visuals.panel_fill = theme.bg;
        visuals.window_fill = theme.panel;
        visuals.widgets.noninteractive.bg_fill = theme.panel_soft;
        visuals.widgets.inactive.bg_fill = theme.card;
        visuals.widgets.hovered.bg_fill = theme.card_hover;
        visuals.widgets.active.bg_fill = theme.accent;
        visuals.selection.bg_fill = theme.accent;
        visuals.hyperlink_color = theme.accent;

        let mut style = (*ctx.style()).clone();
        style.visuals = visuals;
        style.spacing.item_spacing = Vec2::new(10.0, 8.0);
        style.spacing.button_padding = Vec2::new(14.0, 9.0);
        style.text_styles.insert(egui::TextStyle::Heading, FontId::proportional(25.0));
        style.text_styles.insert(egui::TextStyle::Body, FontId::proportional(14.0));
        style.text_styles.insert(egui::TextStyle::Button, FontId::proportional(14.0));
        ctx.set_style(style);
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
            Ok(()) => "Saved".to_string(),
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
            Ok(status) if status.success() => "Wallpaper applied".to_string(),
            Ok(_) => "Wallpaper helper returned an error".to_string(),
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
            Ok(_) => "QML bar restarted".to_string(),
            Err(err) => format!("Could not restart QML bar: {err}"),
        };
    }

    fn save_and_restart_bar(&mut self) {
        self.save_all();
        self.restart_bar();
    }

    fn install_selected_icon_font(&mut self) {
        let pack = self.value("DOTFILES_BAR_ICON_PACK");
        let package = match pack.as_str() {
            "fontawesome" => "otf-font-awesome",
            "nerd" => "ttf-nerd-fonts-symbols",
            _ => {
                self.status = "Text icons do not need an extra font".to_string();
                return;
            }
        };

        let terminal = self.value("DOTFILES_TERMINAL");
        let command = format!(
            "if pacman -Qi {pkg} >/dev/null 2>&1; then echo '{pkg} already installed'; \
             elif command -v paru >/dev/null 2>&1; then paru -S --needed {pkg}; \
             elif command -v yay >/dev/null 2>&1; then yay -S --needed {pkg}; \
             else sudo pacman -S --needed {pkg}; fi; echo; read -rp 'Press enter to close...'",
            pkg = package
        );

        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("{} -e sh -c {}", terminal, shell_quote(&command)))
            .spawn();

        self.status = format!("Opening installer for {package}");
    }

    fn shortcut_hint_rows(&self) -> Vec<(String, String)> {
        let wanted = [
            ("App launcher", "Open apps"),
            ("Terminal", "Terminal"),
            ("SplinterDots", "SplinterDots"),
            ("Reload desktop", "Reload desktop"),
        ];

        let mut rows = Vec::new();
        for (name, label) in wanted {
            if let Some(sc) = self.shortcuts.iter().find(|sc| sc.name.eq_ignore_ascii_case(name)) {
                rows.push((format_shortcut_key(&sc.key), label.to_string()));
            }
        }

        rows.push(("Super + Left Mouse".to_string(), "Drag windows".to_string()));
        rows.push(("Super + Right Mouse".to_string(), "Resize windows".to_string()));
        rows.truncate(6);
        rows
    }

    fn sidebar(&mut self, ctx: &egui::Context) {
        let theme = self.app_theme();

        egui::SidePanel::left("sidebar")
            .exact_width(234.0)
            .frame(Frame {
                fill: theme.sidebar,
                inner_margin: Margin::same(18),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("SplinterDots").heading().color(theme.text).strong());
                    ui.label(RichText::new("Arch + Hyprland").color(theme.muted));
                });

                ui.add_space(24.0);

                for tab in [
                    Tab::Overview,
                    Tab::Hyprland,
                    Tab::Shortcuts,
                    Tab::Appearance,
                    Tab::Bar,
                    Tab::Widgets,
                    Tab::Setup,
                ] {
                    self.nav_button(ui, tab);
                    ui.add_space(4.0);
                }

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    ui.label(RichText::new("Local config").color(theme.muted));
                    ui.monospace("~/.config/dotfiles");
                });
            });
    }

    fn nav_button(&mut self, ui: &mut egui::Ui, tab: Tab) {
        let theme = self.app_theme();
        let selected = self.tab == tab;
        let fill = if selected { theme.accent } else { theme.panel_soft };
        let text = if selected { Color32::WHITE } else { theme.text };

        let response = Frame {
            fill,
            corner_radius: CornerRadius::same(13),
            inner_margin: Margin::symmetric(12, 10),
            stroke: if selected {
                Stroke::new(1.0, theme.accent)
            } else {
                Stroke::new(1.0, theme.border)
            },
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(tab.icon()).color(text).size(17.0).strong());
                ui.add_space(6.0);
                ui.label(RichText::new(tab.title()).color(text).strong());
            });
        })
        .response
        .interact(egui::Sense::click());

        if response.clicked() {
            self.tab = tab;
        }
    }

    fn page_header(&self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading(RichText::new(self.tab.title()).color(theme.text));
                ui.label(RichText::new(self.tab.subtitle()).color(theme.muted));
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                pill(ui, &theme, &self.value("DOTFILES_THEME"));
            });
        });

        ui.add_space(14.0);
    }

    fn card<R>(&self, ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
        let theme = self.app_theme();

        Frame {
            fill: theme.card,
            corner_radius: CornerRadius::same(18),
            inner_margin: Margin::same(18),
            outer_margin: Margin::symmetric(0, 6),
            stroke: Stroke::new(1.0, theme.border),
            ..Default::default()
        }
        .show(ui, add_contents)
        .inner
    }

    fn soft_card<R>(&self, ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
        let theme = self.app_theme();

        Frame {
            fill: theme.panel_soft,
            corner_radius: CornerRadius::same(16),
            inner_margin: Margin::same(14),
            stroke: Stroke::new(1.0, theme.border),
            ..Default::default()
        }
        .show(ui, add_contents)
        .inner
    }

    fn overview_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        self.card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Your desktop is ready to shape").size(22.0).strong());
                    ui.label(RichText::new("Friendly controls for your Arch + Hyprland setup.").color(theme.muted));
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    Frame {
                        fill: theme.accent,
                        corner_radius: CornerRadius::same(999),
                        inner_margin: Margin::symmetric(14, 8),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.label(RichText::new("Modern mode").color(Color32::WHITE).strong());
                    });
                });
            });
        });

        egui::Grid::new("overview_stats")
            .num_columns(3)
            .spacing([12.0, 12.0])
            .show(ui, |ui| {
                self.stat_card(ui, "Desktop", "Hyprland", "Window manager");
                self.stat_card(ui, "Bar", "Reactive QML", "Per-monitor workspaces");
                self.stat_card(ui, "Widgets", "Configurable", "Optional modules");
                ui.end_row();
            });

        self.card(ui, |ui| {
            ui.label(RichText::new("Favorite shortcuts").strong().size(17.0));
            ui.add_space(10.0);

            egui::Grid::new("overview-shortcuts")
                .num_columns(3)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    for (index, (key, label)) in self.shortcut_hint_rows().iter().enumerate() {
                        self.soft_card(ui, |ui| {
                            ui.set_min_size(Vec2::new(200.0, 54.0));
                            ui.label(RichText::new(key).strong().color(theme.text));
                            ui.label(RichText::new(label).color(theme.muted));
                        });
                        if index % 3 == 2 {
                            ui.end_row();
                        }
                    }
                });
        });

        self.card(ui, |ui| {
            ui.label(RichText::new("Installed pieces").strong().size(17.0));
            ui.add_space(8.0);
            for item in [
                "Hyprland · Quickshell QML bar · Wofi launcher · Mako notifications",
                "Kitty terminal · Thunar file manager · Zen Browser",
                "PipeWire audio · NetworkManager · Bluetooth",
                "Screenshot · Clipboard · Wallpaper · XDG portals · greetd login",
            ] {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(theme.accent));
                    ui.label(RichText::new(item).color(theme.muted));
                });
            }
        });
    }

    fn stat_card(&self, ui: &mut egui::Ui, label: &str, value: &str, note: &str) {
        let theme = self.app_theme();

        self.card(ui, |ui| {
            ui.set_min_size(Vec2::new(190.0, 88.0));
            ui.label(RichText::new(label).color(theme.muted));
            ui.label(RichText::new(value).size(20.0).strong().color(theme.text));
            ui.label(RichText::new(note).color(theme.muted));
        });
    }

    fn hyprland_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        self.card(ui, |ui| {
            ui.label(RichText::new("Safe Hyprland settings").size(18.0).strong());
            ui.label(RichText::new("Only beginner-friendly switches, dropdowns, and number controls are shown here.").color(theme.muted));
        });

        let mut grouped: BTreeMap<String, Vec<HyprOption>> = BTreeMap::new();
        for option in &self.hypr_schema {
            grouped
                .entry(option.section.clone().unwrap_or_else(|| "Other".to_string()))
                .or_default()
                .push(option.clone());
        }

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (section, options) in grouped {
                self.card(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("◇").color(theme.accent).size(18.0).strong());
                        ui.label(RichText::new(section).size(17.0).strong());
                    });
                    ui.add_space(8.0);

                    for option in options {
                        self.hypr_option_row(ui, &option);
                    }
                });
            }
        });
    }

    fn hypr_option_row(&mut self, ui: &mut egui::Ui, option: &HyprOption) {
        let theme = self.app_theme();
        let path = option.path.clone();
        let label = option
            .label
            .clone()
            .unwrap_or_else(|| path.replace(':', " / ").replace('_', " "));

        let kind = option.kind.clone().unwrap_or_else(|| "text".to_string());
        let default = option.default.clone().unwrap_or_default();
        let current = self.hypr_values.entry(path.clone()).or_insert(default);

        Frame {
            fill: theme.panel_soft,
            corner_radius: CornerRadius::same(12),
            inner_margin: Margin::symmetric(12, 9),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(label).color(theme.text).strong());
                    ui.label(RichText::new(path.clone()).color(theme.muted).monospace().size(11.0));
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| match kind.as_str() {
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
                            .width(170.0)
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
                        if ui.add(egui::DragValue::new(&mut value).range(min..=max)).changed() {
                            *current = value.to_string();
                        }
                    }
                    "float" => {
                        let min = option.min.unwrap_or(0.0);
                        let max = option.max.unwrap_or(9999.0);
                        let mut value = current.parse::<f64>().unwrap_or(min);
                        if ui.add(egui::DragValue::new(&mut value).speed(0.05).range(min..=max)).changed() {
                            *current = format_float(value);
                        }
                    }
                    _ => {}
                });
            });
        });
        ui.add_space(6.0);
    }

    fn shortcuts_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        self.card(ui, |ui| {
            ui.label(RichText::new("Shortcut editor").size(18.0).strong());
            ui.label(RichText::new("Every shortcut starts with Super. The list and overview update while you type.").color(theme.muted));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if primary_button(ui, &theme, "Add shortcut").clicked() {
                    self.shortcuts.push(Shortcut {
                        name: "New shortcut".to_string(),
                        key: "SHIFT, N".to_string(),
                        kind: "app".to_string(),
                        value: self.value("DOTFILES_TERMINAL"),
                    });
                    self.selected_shortcut = Some(self.shortcuts.len() - 1);
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
        });

        ui.columns(2, |columns| {
            self.card(&mut columns[0], |ui| {
                ui.label(RichText::new("Current shortcuts").strong().size(16.0));
                ui.add_space(8.0);

                ScrollArea::vertical().max_height(430.0).show(ui, |ui| {
                    for (index, sc) in self.shortcuts.iter().enumerate() {
                        let selected = self.selected_shortcut == Some(index);
                        let fill = if selected { theme.accent } else { theme.panel_soft };
                        let text = if selected { Color32::WHITE } else { theme.text };

                        let response = Frame {
                            fill,
                            corner_radius: CornerRadius::same(12),
                            inner_margin: Margin::symmetric(12, 10),
                            stroke: Stroke::new(1.0, if selected { theme.accent } else { theme.border }),
                            ..Default::default()
                        }
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format_shortcut_key(&sc.key)).color(text).strong());
                                ui.label(RichText::new(&sc.name).color(text));
                            });
                        })
                        .response
                        .interact(egui::Sense::click());

                        if response.clicked() {
                            self.selected_shortcut = Some(index);
                        }

                        ui.add_space(6.0);
                    }
                });
            });

            self.card(&mut columns[1], |ui| {
                ui.label(RichText::new("Edit selected").strong().size(16.0));
                ui.add_space(8.0);

                if let Some(index) = self.selected_shortcut {
                    if index < self.shortcuts.len() {
                        let mut changed = false;

                        ui.label(RichText::new("Name").color(theme.muted));
                        changed |= ui.text_edit_singleline(&mut self.shortcuts[index].name).changed();

                        ui.label(RichText::new("Key").color(theme.muted));
                        changed |= ui.text_edit_singleline(&mut self.shortcuts[index].key).changed();
                        ui.label(RichText::new("Examples: Return · D · SHIFT, S · CTRL, ALT, T").color(theme.muted).size(11.0));

                        ui.add_space(8.0);
                        ui.label(RichText::new("Type").color(theme.muted));
                        let mut kind = self.shortcuts[index].kind.clone();
                        egui::ComboBox::from_id_source("shortcut-kind")
                            .selected_text(friendly_kind(&kind))
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(kind == "app", "App / script").clicked() {
                                    kind = "app".to_string();
                                }
                                if ui.selectable_label(kind == "desktop", "Desktop action").clicked() {
                                    kind = "desktop".to_string();
                                }
                            });
                        if kind != self.shortcuts[index].kind {
                            self.shortcuts[index].kind = kind;
                            changed = true;
                        }

                        ui.label(RichText::new("What should happen").color(theme.muted));
                        changed |= ui.text_edit_singleline(&mut self.shortcuts[index].value).changed();

                        if changed {
                            self.status = "Shortcut updated live".to_string();
                        }
                    }
                } else {
                    ui.label(RichText::new("Select a shortcut to edit it.").color(theme.muted));
                }
            });
        });

        self.card(ui, |ui| {
            ui.label(RichText::new("Mouse controls").strong());
            ui.label(RichText::new("Super + left mouse drag moves windows. Super + right mouse drag resizes windows.").color(theme.muted));
        });
    }

    fn appearance_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        self.card(ui, |ui| {
            ui.label(RichText::new("Theme").strong().size(18.0));
            ui.label(RichText::new("Switch between a dark desktop and a bright desktop.").color(theme.muted));
            ui.add_space(10.0);

            let mut theme_value = self.value("DOTFILES_THEME");
            ui.horizontal(|ui| {
                selectable_pill(ui, &mut theme_value, "dark", "Dark");
                selectable_pill(ui, &mut theme_value, "light", "Light");
            });
            self.set_value("DOTFILES_THEME", theme_value);
        });

        self.card(ui, |ui| {
            ui.label(RichText::new("Accent and wallpaper").strong().size(18.0));
            ui.add_space(10.0);

            let mut accent = self.value("DOTFILES_ACCENT");
            labeled_text(ui, "Accent color", &mut accent);
            self.set_value("DOTFILES_ACCENT", accent);

            let mut wallpaper = self.value("DOTFILES_WALLPAPER");
            labeled_text(ui, "Wallpaper path", &mut wallpaper);
            self.set_value("DOTFILES_WALLPAPER", wallpaper);

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if primary_button(ui, &theme, "Apply wallpaper").clicked() {
                    self.apply_wallpaper();
                }
                if ui.button("Apply theme").clicked() {
                    self.save_all();
                }
            });
        });
    }

    fn bar_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        ScrollArea::vertical().show(ui, |ui| {
            self.card(ui, |ui| {
                ui.label(RichText::new("Layout and speed").strong().size(18.0));
                combo_value(ui, &mut self.values, "DOTFILES_BAR_POSITION", &["top", "bottom"]);
                combo_value(
                    ui,
                    &mut self.values,
                    "DOTFILES_BAR_WORKSPACE_COUNT",
                    &["5", "6", "7", "8", "9", "10", "12", "15", "20"],
                );
                slider_value(ui, &mut self.values, "DOTFILES_BAR_REACTIVE_MS", 40.0, 1000.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_STATUS_MS", 500.0, 6000.0);
                ui.label(RichText::new("Lower reactive value = faster monitor/workspace updates. 80-150 ms feels instant.").color(theme.muted).size(12.0));
            });

            self.card(ui, |ui| {
                ui.label(RichText::new("Icon pack").strong().size(18.0));
                combo_value(ui, &mut self.values, "DOTFILES_BAR_ICON_PACK", &["nerd", "fontawesome", "text"]);
                combo_value(
                    ui,
                    &mut self.values,
                    "DOTFILES_BAR_ICON_FONT",
                    &["Symbols Nerd Font", "Font Awesome 6 Free", "Font Awesome 6 Brands", "Sans"],
                );

                ui.add_space(8.0);
                if primary_button(ui, &theme, "Install selected icon font").clicked() {
                    self.install_selected_icon_font();
                }
                ui.label(RichText::new("Uses pacman first, then paru/yay when available.").color(theme.muted).size(12.0));
            });

            self.card(ui, |ui| {
                ui.label(RichText::new("Sizing and style").strong().size(18.0));
                slider_value(ui, &mut self.values, "DOTFILES_BAR_HEIGHT", 24.0, 72.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_RADIUS", 0.0, 28.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_OPACITY", 0.2, 1.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_FONT_SIZE", 8.0, 24.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_SPACING", 4.0, 40.0);
                slider_value(ui, &mut self.values, "DOTFILES_BAR_BORDER_WIDTH", 0.0, 4.0);

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if primary_button(ui, &theme, "Save and restart bar").clicked() {
                        self.save_and_restart_bar();
                    }
                    if ui.button("Restart bar only").clicked() {
                        self.restart_bar();
                    }
                });
            });
        });
    }

    fn widgets_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        ScrollArea::vertical().show(ui, |ui| {
            self.card(ui, |ui| {
                ui.label(RichText::new("Enable widgets").strong().size(18.0));
                ui.label(RichText::new("Every widget can be enabled or disabled. Configuration is below.").color(theme.muted));
                ui.add_space(10.0);

                egui::Grid::new("widget_enable_grid")
                    .num_columns(2)
                    .spacing([34.0, 8.0])
                    .show(ui, |ui| {
                        for (index, (label, key)) in WIDGET_TOGGLES.iter().enumerate() {
                            let mut checked = self.bool_value(key);
                            if ui.checkbox(&mut checked, *label).changed() {
                                self.set_bool_value(key, checked);
                            }
                            if index % 2 == 1 {
                                ui.end_row();
                            }
                        }
                    });
            });

            self.card(ui, |ui| {
                ui.label(RichText::new("Clock").strong().size(18.0));
                let mut fmt = self.value("DOTFILES_WIDGET_CLOCK_FORMAT");
                labeled_text(ui, "Format", &mut fmt);
                self.set_value("DOTFILES_WIDGET_CLOCK_FORMAT", fmt);
                let mut seconds = self.bool_value("DOTFILES_WIDGET_CLOCK_SECONDS");
                if ui.checkbox(&mut seconds, "Show seconds").changed() {
                    self.set_bool_value("DOTFILES_WIDGET_CLOCK_SECONDS", seconds);
                }
            });

            self.card(ui, |ui| {
                ui.label(RichText::new("Volume, network, and battery").strong().size(18.0));
                let mut device = self.value("DOTFILES_WIDGET_VOLUME_DEVICE");
                labeled_text(ui, "Volume device", &mut device);
                self.set_value("DOTFILES_WIDGET_VOLUME_DEVICE", device);

                combo_value(ui, &mut self.values, "DOTFILES_WIDGET_NETWORK_STYLE", &["short", "name"]);

                let mut low = self.value("DOTFILES_WIDGET_BATTERY_LOW");
                labeled_text(ui, "Battery low percentage", &mut low);
                self.set_value("DOTFILES_WIDGET_BATTERY_LOW", low);
            });

            self.card(ui, |ui| {
                ui.label(RichText::new("Performance widgets").strong().size(18.0));
                let mut cpu = self.value("DOTFILES_WIDGET_CPU_LABEL");
                labeled_text(ui, "CPU label", &mut cpu);
                self.set_value("DOTFILES_WIDGET_CPU_LABEL", cpu);

                let mut mem = self.value("DOTFILES_WIDGET_MEMORY_LABEL");
                labeled_text(ui, "Memory label", &mut mem);
                self.set_value("DOTFILES_WIDGET_MEMORY_LABEL", mem);

                let mut temp = self.value("DOTFILES_WIDGET_TEMP_SENSOR");
                labeled_text(ui, "Temperature sensor name, optional", &mut temp);
                self.set_value("DOTFILES_WIDGET_TEMP_SENSOR", temp);

                let mut disk = self.value("DOTFILES_WIDGET_DISK_PATH");
                labeled_text(ui, "Disk path", &mut disk);
                self.set_value("DOTFILES_WIDGET_DISK_PATH", disk);
            });

            self.card(ui, |ui| {
                ui.label(RichText::new("Extra widgets").strong().size(18.0));

                let mut brightness = self.value("DOTFILES_WIDGET_BRIGHTNESS_DEVICE");
                labeled_text(ui, "Brightness device, optional", &mut brightness);
                self.set_value("DOTFILES_WIDGET_BRIGHTNESS_DEVICE", brightness);

                let mut media_len = self.value("DOTFILES_WIDGET_MEDIA_LENGTH");
                labeled_text(ui, "Media title length", &mut media_len);
                self.set_value("DOTFILES_WIDGET_MEDIA_LENGTH", media_len);

                let mut updates_cmd = self.value("DOTFILES_WIDGET_UPDATES_COMMAND");
                labeled_text(ui, "Updates check", &mut updates_cmd);
                self.set_value("DOTFILES_WIDGET_UPDATES_COMMAND", updates_cmd);

                let mut kb_label = self.value("DOTFILES_WIDGET_KEYBOARD_LABEL");
                labeled_text(ui, "Keyboard label", &mut kb_label);
                self.set_value("DOTFILES_WIDGET_KEYBOARD_LABEL", kb_label);
            });
        });
    }

    fn setup_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        self.card(ui, |ui| {
            ui.label(RichText::new("Default apps").strong().size(18.0));
            ui.label(RichText::new("These are used by shortcuts and helper actions.").color(theme.muted));
            ui.add_space(10.0);

            for (label, key) in [
                ("Terminal", "DOTFILES_TERMINAL"),
                ("File manager", "DOTFILES_FILE_MANAGER"),
                ("App launcher", "DOTFILES_APP_LAUNCHER"),
                ("Editor", "DOTFILES_EDITOR"),
                ("Browser", "DOTFILES_BROWSER"),
            ] {
                let mut value = self.value(key);
                labeled_text(ui, label, &mut value);
                self.set_value(key, value);
            }
        });

        self.card(ui, |ui| {
            ui.label(RichText::new("Quick actions").strong().size(18.0));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("Open dotfiles folder").clicked() {
                    run_quiet("xdg-open", &[self.paths.root.to_string_lossy().as_ref()]);
                }
                if primary_button(ui, &theme, "Reload Hyprland").clicked() {
                    run_quiet("hyprctl", &["reload"]);
                }
            });
        });
    }
}

const WIDGET_TOGGLES: &[(&str, &str)] = &[
    ("Workspaces", "DOTFILES_BAR_SHOW_WORKSPACES"),
    ("Clock", "DOTFILES_BAR_SHOW_CLOCK"),
    ("Volume", "DOTFILES_BAR_SHOW_VOLUME"),
    ("Network", "DOTFILES_BAR_SHOW_NETWORK"),
    ("Battery", "DOTFILES_BAR_SHOW_BATTERY"),
    ("CPU", "DOTFILES_BAR_SHOW_CPU"),
    ("Memory", "DOTFILES_BAR_SHOW_MEMORY"),
    ("Temperature", "DOTFILES_BAR_SHOW_TEMP"),
    ("Disk", "DOTFILES_BAR_SHOW_DISK"),
    ("Brightness", "DOTFILES_BAR_SHOW_BRIGHTNESS"),
    ("Bluetooth", "DOTFILES_BAR_SHOW_BLUETOOTH"),
    ("Media", "DOTFILES_BAR_SHOW_MEDIA"),
    ("Updates", "DOTFILES_BAR_SHOW_UPDATES"),
    ("Keyboard", "DOTFILES_BAR_SHOW_KEYBOARD"),
];

impl eframe::App for SplinterDots {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_style(ctx);
        let theme = self.app_theme();

        self.sidebar(ctx);

        egui::TopBottomPanel::bottom("bottom_save_bar")
            .frame(Frame {
                fill: theme.sidebar,
                inner_margin: Margin::symmetric(22, 12),
                stroke: Stroke::new(1.0, theme.border),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.no_show_on_startup, "Don't show on startup");

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if primary_button(ui, &theme, "Save changes").clicked() {
                            self.save_all();
                        }

                        if !self.status.is_empty() {
                            let lower = self.status.to_ascii_lowercase();
                            let color = if lower.contains("failed")
                                || lower.contains("error")
                                || lower.contains("could not")
                            {
                                theme.danger
                            } else {
                                theme.success
                            };
                            ui.label(RichText::new(&self.status).color(color).strong());
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(Frame {
                fill: theme.bg,
                inner_margin: Margin::same(22),
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.page_header(ui);

                match self.tab {
                    Tab::Overview => self.overview_tab(ui),
                    Tab::Hyprland => self.hyprland_tab(ui),
                    Tab::Shortcuts => self.shortcuts_tab(ui),
                    Tab::Appearance => self.appearance_tab(ui),
                    Tab::Bar => self.bar_tab(ui),
                    Tab::Widgets => self.widgets_tab(ui),
                    Tab::Setup => self.setup_tab(ui),
                }
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
        "# Machine-local dotfiles settings written by SplinterDots.".to_string(),
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
        sc("SplinterDots", "W", "app", "dotctl center"),
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
        values.insert(option.path.clone(), option.default.clone().unwrap_or_default());
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

fn write_keybinds(paths: &Paths, values: &HashMap<String, String>, shortcuts: &[Shortcut]) -> Result<(), String> {
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

fn write_hyprland_settings(paths: &Paths, schema: &[HyprOption], values: &HashMap<String, String>) -> Result<(), String> {
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
        "# Generated by SplinterDots.".to_string(),
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
            if is_true(value) { "true".to_string() } else { "false".to_string() }
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
            value.parse::<i64>().unwrap_or(min).clamp(min, max).to_string()
        }
        "float" => {
            let min = option.min.unwrap_or(0.0);
            let max = option.max.unwrap_or(9999.0);
            let number = value.parse::<f64>().unwrap_or(min).clamp(min, max);
            format_float(number)
        }
        _ => value.trim().to_string(),
    }
}

struct Palette {
    accent: String,
    background: String,
    surface: String,
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
            text: "#cdd6f4".to_string(),
            muted: "#bac2de".to_string(),
            inactive_border: "#45475a".to_string(),
            bar_rgb: "1e1e2e".to_string(),
            active_text: "#11111b".to_string(),
        }
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

fn write_quickshell_bar(paths: &Paths, values: &HashMap<String, String>) -> Result<(), String> {
    ensure_dir(&paths.quickshell_dir)?;

    let palette = theme_palette(values);
    let icons = icons_for(&value_or(values, "DOTFILES_BAR_ICON_PACK"));
    let position = value_or(values, "DOTFILES_BAR_POSITION");
    let top = if position == "top" { "true" } else { "false" };
    let bottom = if position == "bottom" { "true" } else { "false" };

    let height = clamp_i(value_or(values, "DOTFILES_BAR_HEIGHT"), 34, 24, 72);
    let radius = clamp_i(value_or(values, "DOTFILES_BAR_RADIUS"), 10, 0, 28);
    let opacity = clamp_f(value_or(values, "DOTFILES_BAR_OPACITY"), 0.92, 0.2, 1.0);
    let font_size = clamp_i(value_or(values, "DOTFILES_BAR_FONT_SIZE"), 12, 8, 24);
    let spacing = clamp_i(value_or(values, "DOTFILES_BAR_SPACING"), 14, 4, 40);
    let border_width = clamp_i(value_or(values, "DOTFILES_BAR_BORDER_WIDTH"), 1, 0, 4);
    let workspace_count = clamp_i(value_or(values, "DOTFILES_BAR_WORKSPACE_COUNT"), 9, 1, 20);
    let reactive_ms = clamp_i(value_or(values, "DOTFILES_BAR_REACTIVE_MS"), 120, 40, 1000);
    let status_ms = clamp_i(value_or(values, "DOTFILES_BAR_STATUS_MS"), 1500, 500, 6000);
    let icon_font = value_or(values, "DOTFILES_BAR_ICON_FONT");

    let mut clock_format = value_or(values, "DOTFILES_WIDGET_CLOCK_FORMAT");
    if is_true(&value_or(values, "DOTFILES_WIDGET_CLOCK_SECONDS")) && !clock_format.contains("%S") {
        clock_format.push_str(":%S");
    }

    let bg_alpha = (opacity * 255.0) as i32;
    let bg_color = format!("#{:02x}{}", bg_alpha, palette.bar_rgb);

    let left_section = if is_true(&value_or(values, "DOTFILES_BAR_SHOW_WORKSPACES")) {
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
                color: (index + 1) === root.activeWorkspace ? "{accent}" : "{surface}"
                opacity: (index + 1) === root.activeWorkspace ? 1.0 : 0.72

                Text {{
                  anchors.centerIn: parent
                  text: index + 1
                  color: (index + 1) === root.activeWorkspace ? "{active_text}" : "{text}"
                  font.bold: true
                  font.pixelSize: {workspace_font}
                  font.family: "{icon_font}"
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
            accent = palette.accent,
            surface = palette.surface,
            active_text = palette.active_text,
            text = palette.text,
            workspace_font = font_size - 1,
            icon_font = icon_font,
        )
    } else {
        String::new()
    };

    let center_section = if is_true(&value_or(values, "DOTFILES_BAR_SHOW_CLOCK")) {
        format!(
            r#"
          Text {{
            id: clock
            Layout.alignment: Qt.AlignCenter
            color: "{text}"
            font.bold: true
            font.pixelSize: {font_size}
            font.family: "{icon_font}"
            text: "..."

            Process {{
              id: dateProc
              command: ["date", {clock_arg}]
              running: true
              stdout: StdioCollector {{
                onStreamFinished: clock.text = "{clock_icon} " + this.text.trim()
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
            font_size = font_size,
            icon_font = icon_font,
            clock_arg = json_string(&format!("+{clock_format}")),
            clock_icon = icons.clock,
        )
    } else {
        String::new()
    };

    let status_cmd = build_status_command(values, &icons);
    let right_section = if status_cmd.trim().is_empty() {
        String::new()
    } else {
        format!(
            r#"
          Text {{
            id: status
            Layout.alignment: Qt.AlignRight | Qt.AlignVCenter
            color: "{muted}"
            font.pixelSize: {font_size}
            font.family: "{icon_font}"
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
              interval: {status_ms}
              running: true
              repeat: true
              onTriggered: statusProc.running = true
            }}
          }}"#,
            muted = palette.muted,
            font_size = font_size,
            icon_font = icon_font,
            status_cmd = json_string(&status_cmd),
            status_ms = status_ms,
        )
    };

    let template = r#"// SplinterDots Quickshell bar.
// Generated by SplinterDots.

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
      property string screenName: modelData.name
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
        id: stateProc
        command: ["sh", "-c", "hyprctl monitors -j"]
        running: true
        stdout: StdioCollector {
          onStreamFinished: {
            try {
              var monitors = JSON.parse(this.text)
              for (var i = 0; i < monitors.length; i++) {
                if (monitors[i].name === root.screenName && monitors[i].activeWorkspace) {
                  root.activeWorkspace = monitors[i].activeWorkspace.id
                }
              }
            } catch (e) {
            }
          }
        }
      }

      Timer {
        interval: __REACTIVE_MS__
        running: true
        repeat: true
        onTriggered: stateProc.running = true
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
        .replace("__REACTIVE_MS__", &reactive_ms.to_string())
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

#[derive(Clone)]
struct IconSet {
    clock: &'static str,
    volume: &'static str,
    muted: &'static str,
    network: &'static str,
    battery: &'static str,
    cpu: &'static str,
    memory: &'static str,
    temp: &'static str,
    disk: &'static str,
    brightness: &'static str,
    bluetooth: &'static str,
    media: &'static str,
    updates: &'static str,
    keyboard: &'static str,
}

fn icons_for(pack: &str) -> IconSet {
    match pack {
        "fontawesome" => IconSet {
            clock: "",
            volume: "",
            muted: "",
            network: "",
            battery: "",
            cpu: "",
            memory: "",
            temp: "",
            disk: "",
            brightness: "",
            bluetooth: "",
            media: "",
            updates: "",
            keyboard: "",
        },
        "text" => IconSet {
            clock: "TIME",
            volume: "VOL",
            muted: "MUTE",
            network: "NET",
            battery: "BAT",
            cpu: "CPU",
            memory: "RAM",
            temp: "TEMP",
            disk: "DISK",
            brightness: "BRI",
            bluetooth: "BT",
            media: "MEDIA",
            updates: "UPD",
            keyboard: "KB",
        },
        _ => IconSet {
            clock: "",
            volume: "",
            muted: "",
            network: "󰤨",
            battery: "󰁹",
            cpu: "",
            memory: "",
            temp: "",
            disk: "󰋊",
            brightness: "󰃠",
            bluetooth: "",
            media: "",
            updates: "󰚰",
            keyboard: "",
        },
    }
}

fn build_status_command(values: &HashMap<String, String>, icons: &IconSet) -> String {
    let mut parts: Vec<String> = Vec::new();

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_VOLUME")) {
        let device = value_or(values, "DOTFILES_WIDGET_VOLUME_DEVICE");
        parts.push(format!(
            "printf '{} '; wpctl get-volume {} 2>/dev/null | awk '{{v=int($2*100); if($3==\"[MUTED]\") print \"{}\"; else print v\"%\"}}'",
            icons.volume, shell_escape(&device), icons.muted
        ));
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_NETWORK")) {
        let style = value_or(values, "DOTFILES_WIDGET_NETWORK_STYLE");
        if style == "name" {
            parts.push(format!(
                "dev=$(nmcli -t -f DEVICE,STATE device 2>/dev/null | awk -F: '$2==\"connected\"{{print $1; exit}}'); [ -n \"$dev\" ] && printf '  {} %s' \"$dev\" || printf '  {} off'",
                icons.network, icons.network
            ));
        } else {
            parts.push(format!(
                "nmcli -t -f GENERAL.STATE device show 2>/dev/null | grep -q ':100' && printf '  {} on' || printf '  {} off'",
                icons.network, icons.network
            ));
        }
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_BATTERY")) {
        let low = clamp_i(value_or(values, "DOTFILES_WIDGET_BATTERY_LOW"), 20, 1, 99);
        parts.push(format!(
            "bat=$(cat /sys/class/power_supply/BAT0/capacity 2>/dev/null || cat /sys/class/power_supply/BAT1/capacity 2>/dev/null); [ -n \"$bat\" ] && if [ \"$bat\" -le {low} ]; then printf '  {icon} %s%%!' \"$bat\"; else printf '  {icon} %s%%' \"$bat\"; fi",
            low = low,
            icon = icons.battery
        ));
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_CPU")) {
        let label = value_or(values, "DOTFILES_WIDGET_CPU_LABEL");
        parts.push(format!(
            "cpu=$(top -bn1 | awk -F'[, ]+' '/Cpu\\(s\\)/{{print int($2+$4)}}' 2>/dev/null); [ -n \"$cpu\" ] && printf '  {} {} %s%%' \"$cpu\"",
            icons.cpu,
            shell_safe_text(&label)
        ));
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_MEMORY")) {
        let label = value_or(values, "DOTFILES_WIDGET_MEMORY_LABEL");
        parts.push(format!(
            "mem=$(free -m | awk '/^Mem/{{printf \"%dMB\", $3}}' 2>/dev/null); [ -n \"$mem\" ] && printf '  {} {} %s' \"$mem\"",
            icons.memory,
            shell_safe_text(&label)
        ));
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_TEMP")) {
        let sensor = value_or(values, "DOTFILES_WIDGET_TEMP_SENSOR");
        if sensor.trim().is_empty() {
            parts.push(format!(
                "tmp=$(sensors 2>/dev/null | awk '/Package id 0|Tctl|temp1/{{gsub(/[+°C]/, \"\", $2); print int($2)\"°C\"; exit}}'); [ -n \"$tmp\" ] && printf '  {} %s' \"$tmp\"",
                icons.temp
            ));
        } else {
            parts.push(format!(
                "tmp=$(sensors 2>/dev/null | awk '/{sensor}/{{gsub(/[+°C]/, \"\", $2); print int($2)\"°C\"; exit}}'); [ -n \"$tmp\" ] && printf '  {icon} %s' \"$tmp\"",
                sensor = shell_safe_text(&sensor),
                icon = icons.temp
            ));
        }
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_DISK")) {
        let path = value_or(values, "DOTFILES_WIDGET_DISK_PATH");
        parts.push(format!(
            "disk=$(df -h {} 2>/dev/null | awk 'NR==2{{print $5}}'); [ -n \"$disk\" ] && printf '  {} %s' \"$disk\"",
            shell_escape(&path),
            icons.disk
        ));
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_BRIGHTNESS")) {
        let device = value_or(values, "DOTFILES_WIDGET_BRIGHTNESS_DEVICE");
        if device.trim().is_empty() {
            parts.push(format!(
                "bri=$(brightnessctl -m 2>/dev/null | awk -F, '{{print $4}}'); [ -n \"$bri\" ] && printf '  {} %s' \"$bri\"",
                icons.brightness
            ));
        } else {
            parts.push(format!(
                "bri=$(brightnessctl -d {} -m 2>/dev/null | awk -F, '{{print $4}}'); [ -n \"$bri\" ] && printf '  {} %s' \"$bri\"",
                shell_escape(&device),
                icons.brightness
            ));
        }
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_BLUETOOTH")) {
        parts.push(format!(
            "bluetoothctl show 2>/dev/null | grep -q 'Powered: yes' && printf '  {} on' || printf '  {} off'",
            icons.bluetooth, icons.bluetooth
        ));
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_MEDIA")) {
        let max_len = clamp_i(value_or(values, "DOTFILES_WIDGET_MEDIA_LENGTH"), 28, 8, 80);
        parts.push(format!(
            "media=$(playerctl metadata --format '{{{{artist}}}} - {{{{title}}}}' 2>/dev/null | cut -c1-{max}); [ -n \"$media\" ] && printf '  {icon} %s' \"$media\"",
            max = max_len,
            icon = icons.media
        ));
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_UPDATES")) {
        let command = value_or(values, "DOTFILES_WIDGET_UPDATES_COMMAND");
        parts.push(format!(
            "upd=$({cmd}); [ -n \"$upd\" ] && [ \"$upd\" != \"0\" ] && printf '  {icon} %s' \"$upd\"",
            cmd = command,
            icon = icons.updates
        ));
    }

    if is_true(&value_or(values, "DOTFILES_BAR_SHOW_KEYBOARD")) {
        let label = value_or(values, "DOTFILES_WIDGET_KEYBOARD_LABEL");
        parts.push(format!(
            "kb=$(hyprctl devices -j 2>/dev/null | grep -m1 -o '\"active_keymap\":\"[^\"]*' | cut -d'\"' -f4); [ -n \"$kb\" ] && printf '  {} {} %s' \"$kb\"",
            icons.keyboard,
            shell_safe_text(&label)
        ));
    }

    parts.join("; ")
}

fn primary_button(ui: &mut egui::Ui, theme: &AppTheme, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).strong().color(Color32::WHITE))
            .fill(theme.accent)
            .min_size(Vec2::new(120.0, 34.0)),
    )
}

fn pill(ui: &mut egui::Ui, theme: &AppTheme, text: &str) {
    Frame {
        fill: theme.panel_soft,
        corner_radius: CornerRadius::same(999),
        inner_margin: Margin::symmetric(12, 7),
        stroke: Stroke::new(1.0, theme.border),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.label(RichText::new(text).color(theme.muted));
    });
}

fn selectable_pill(ui: &mut egui::Ui, current: &mut String, value: &str, label: &str) {
    let selected = current == value;
    if ui.selectable_label(selected, label).clicked() {
        *current = value.to_string();
    }
}

fn labeled_text(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.label(RichText::new(label).color(ui.visuals().weak_text_color()));
    ui.text_edit_singleline(value);
    ui.add_space(6.0);
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
        values.insert(key.to_string(), format_float(current));
    }
}

fn label_from_key(key: &str) -> String {
    key.trim_start_matches("DOTFILES_")
        .trim_start_matches("BAR_")
        .trim_start_matches("WIDGET_")
        .replace('_', " ")
        .to_ascii_lowercase()
}

fn value_or(values: &HashMap<String, String>, key: &str) -> String {
    values.get(key).cloned().unwrap_or_else(|| default_value(key))
}

fn is_true(value: &str) -> bool {
    matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

fn friendly_kind(kind: &str) -> &'static str {
    if kind == "app" { "App / script" } else { "Desktop action" }
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

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

fn shell_escape(value: &str) -> String {
    shell_quote(value)
}

fn shell_safe_text(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .collect()
}

fn clamp_i(value: String, default: i32, min: i32, max: i32) -> i32 {
    value.parse::<i32>().unwrap_or(default).clamp(min, max)
}

fn clamp_f(value: String, default: f64, min: f64, max: f64) -> f64 {
    value.parse::<f64>().unwrap_or(default).clamp(min, max)
}

fn format_float(value: f64) -> String {
    let mut s = format!("{value:.3}");
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

fn parse_hex_color(value: &str) -> Option<Color32> {
    let clean = value.trim().trim_start_matches('#');
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some(Color32::from_rgb(r, g, b))
}

fn hex_to_hypr_rgba(hex: &str) -> String {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 && clean.chars().all(|c| c.is_ascii_hexdigit()) {
        format!("rgba({}ff)", clean.to_lowercase())
    } else {
        "rgba(89b4faff)".to_string()
    }
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
