use eframe::egui::{
    self, Align, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame,
    Layout, Margin, RichText, ScrollArea, Stroke, Vec2,
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
    ("DOTFILES_UI_FONT", "CaskaydiaCove Nerd Font"),
    ("DOTFILES_ICON_THEME", "Papirus-Dark"),
    ("DOTFILES_CURSOR_THEME", "Bibata-Modern-Ice"),
    ("DOTFILES_THEME", "dark"),
    ("DOTFILES_ACCENT", "#89b4fa"),
    ("DOTFILES_WALLPAPER", ""),
    ("DOTFILES_WALLPAPER_DIR", "~/Pictures/Wallpapers"),
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
    ("DOTFILES_BAR_LEFT_WIDGETS", ""),
    ("DOTFILES_BAR_CENTER_WIDGETS", ""),
    ("DOTFILES_BAR_RIGHT_WIDGETS", ""),
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
    (
        "DOTFILES_WIDGET_UPDATES_COMMAND",
        "checkupdates 2>/dev/null | wc -l",
    ),
    ("DOTFILES_WIDGET_KEYBOARD_LABEL", "KB"),
];

const EXCLUDED_HYPR_OPTIONS: &[&str] = &[
    "misc:disable_hyprland_qtutils_check",
    "debug:watchdog_timeout",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Keybind {
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
    Keybinds,
    Appearance,
    Addons,
    Bar,
    Widgets,
    Setup,
}

impl Tab {
    fn title(self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Hyprland => "Hyprland",
            Tab::Keybinds => "Keybinds",
            Tab::Appearance => "Appearance",
            Tab::Addons => "Addons",
            Tab::Bar => "Bar & Widgets",
            Tab::Widgets => "Widgets",
            Tab::Setup => "Default Apps",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Tab::Overview => "Your desktop at a glance",
            Tab::Hyprland => "Safe visual and behavior settings",
            Tab::Keybinds => "",
            Tab::Appearance => "Theme, accent, and wallpaper",
            Tab::Addons => "Download optional fonts, icons, tools, and extras",
            Tab::Bar => "Bar layout, widgets, icons, and speed",
            Tab::Widgets => "Choose and configure every bar widget",
            Tab::Setup => "Default apps and helper actions",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Tab::Overview => "",
            Tab::Hyprland => "",
            Tab::Keybinds => "",
            Tab::Appearance => "",
            Tab::Addons => "",
            Tab::Bar => "󰖰",
            Tab::Widgets => "󰃭",
            Tab::Setup => "",
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
    input: Color32,
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
    keybinds: Vec<Keybind>,
    saved_keybinds: Vec<Keybind>,
    selected_keybind: Option<usize>,
    hypr_schema: Vec<HyprOption>,
    hypr_values: HashMap<String, String>,
    hypr_search: String,
    wallpaper_images: Vec<PathBuf>,
    wallpaper_textures: HashMap<String, egui::TextureHandle>,
    fastfetch_output: String,
    addon_search: String,
    addon_refresh_marker_seen: bool,
    addon_category_filter: String,
    preview_font_families: HashMap<String, FontFamily>,
    dragged_bar_widget: Option<(String, usize)>,
    status: String,
    no_show_on_startup: bool,
    loaded_ui_font: String,
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
            load_icon_font(&cc.egui_ctx);
            Ok(Box::new(SplinterDots::new()))
        }),
    )
}


fn load_icon_font(ctx: &egui::Context) {
    let candidates = [
        "/usr/share/fonts/TTF/SymbolsNerdFont-Regular.ttf",
        "/usr/share/fonts/TTF/SymbolsNerdFontMono-Regular.ttf",
        "/usr/share/fonts/OTF/SymbolsNerdFont-Regular.otf",
        "/usr/share/fonts/TTF/CaskaydiaCoveNerdFont-Regular.ttf",
        "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf",
    ];

    let Some(bytes) = candidates.iter().find_map(|path| fs::read(path).ok()) else {
        return;
    };

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "splinter-icons".to_string(),
        FontData::from_owned(bytes).into(),
    );
    fonts
        .families
        .entry(FontFamily::Name("splinter-icons".into()))
        .or_default()
        .push("splinter-icons".to_string());

    ctx.set_fonts(fonts);
}

fn icon_text(icon: &str, size: f32, color: Color32) -> RichText {
    RichText::new(icon)
        .font(FontId::new(size, FontFamily::Name("splinter-icons".into())))
        .color(color)
        .strong()
}

fn load_fastfetch_output() -> String {
    let output = Command::new("fastfetch")
        .args(["--logo", "none"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            let clean = strip_ansi(&text);
            if clean.trim().is_empty() {
                "fastfetch returned no output.".to_string()
            } else {
                clean
            }
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr);
            if err.trim().is_empty() {
                "fastfetch failed without an error message.".to_string()
            } else {
                format!("fastfetch failed:\n{}", strip_ansi(&err))
            }
        }
        Err(_) => "fastfetch is not installed or could not be started.".to_string(),
    }
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
        }
        out.push(ch);
    }

    out
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
            hypr_schema_file: root
                .join("config")
                .join("dotfiles")
                .join("hyprland-options.json"),
        }
    }

    fn script(&self, name: &str) -> PathBuf {
        self.root.join("scripts").join(name)
    }

    fn repo_settings(&self) -> PathBuf {
        self.root
            .join("config")
            .join("dotfiles")
            .join("settings.conf")
    }
}

impl SplinterDots {
    fn new() -> Self {
        let paths = Paths::new();
        let values = read_settings(&paths);
        let keybinds = load_keybinds(&paths);
        let saved_keybinds = keybinds.clone();
        let hypr_schema = load_hypr_schema(&paths);
        let hypr_values = load_hypr_values(&paths, &hypr_schema);
        let wallpaper_images = scan_wallpaper_dir(&value_or(&values, "DOTFILES_WALLPAPER_DIR"));
        let fastfetch_output = load_fastfetch_output();
        let no_show_on_startup = paths.disabled_file.exists();

        Self {
            paths,
            tab: Tab::Overview,
            values,
            keybinds,
            saved_keybinds,
            selected_keybind: None,
            hypr_schema,
            hypr_values,
            hypr_search: String::new(),
            wallpaper_images,
            wallpaper_textures: HashMap::new(),
            fastfetch_output,
            addon_search: String::new(),
            addon_refresh_marker_seen: false,
            addon_category_filter: "Show All".to_string(),
            status: String::new(),
            no_show_on_startup,
            loaded_ui_font: String::new(),
            preview_font_families: HashMap::new(),
            dragged_bar_widget: None,
        }
    }

    fn sync_ui_font(&mut self, ctx: &egui::Context) {
        let selected_font = self.value("DOTFILES_UI_FONT");

        if self.loaded_ui_font == selected_font && !self.preview_font_families.is_empty() {
            return;
        }

        let mut fonts = FontDefinitions::default();
        self.preview_font_families.clear();

        // 1. Icon font for sidebar/buttons.
        let icon_candidates = [
            "/usr/share/fonts/TTF/SymbolsNerdFont-Regular.ttf",
            "/usr/share/fonts/TTF/SymbolsNerdFontMono-Regular.ttf",
            "/usr/share/fonts/OTF/SymbolsNerdFont-Regular.otf",
            "/usr/share/fonts/TTF/CaskaydiaCoveNerdFont-Regular.ttf",
            "/usr/share/fonts/TTF/JetBrainsMonoNerdFont-Regular.ttf",
        ];

        for path in icon_candidates {
            if let Ok(bytes) = fs::read(path) {
                fonts.font_data.insert(
                    "splinter-icons-data".to_string(),
                    FontData::from_owned(bytes).into(),
                );

                fonts
                    .families
                    .entry(FontFamily::Name("splinter-icons".into()))
                    .or_default()
                    .push("splinter-icons-data".to_string());

                break;
            }
        }

        // 2. Selected global UI font.
        if let Some(path) = font_file_candidates(&selected_font).first() {
            if let Ok(bytes) = fs::read(path) {
                fonts.font_data.insert(
                    "splinter-ui-font".to_string(),
                    FontData::from_owned(bytes).into(),
                );

                fonts
                    .families
                    .entry(FontFamily::Proportional)
                    .or_default()
                    .insert(0, "splinter-ui-font".to_string());

                fonts
                    .families
                    .entry(FontFamily::Monospace)
                    .or_default()
                    .insert(0, "splinter-ui-font".to_string());
            }
        }

        // 3. Separate preview font families for every font card.
        for choice in font_choices() {
            let candidates = font_file_candidates(choice.value);
            let Some(path) = candidates.first() else {
                continue;
            };

            let Ok(bytes) = fs::read(path) else {
                continue;
            };

            let family_name = format!("preview-{}", normalize_font_name(choice.value));
            let font_id = format!("{}-{}", family_name, path.display());

            fonts.font_data.insert(
                font_id.clone(),
                FontData::from_owned(bytes).into(),
            );

            fonts
                .families
                .entry(FontFamily::Name(family_name.clone().into()))
                .or_default()
                .push(font_id);

            self.preview_font_families
                .insert(choice.value.to_string(), FontFamily::Name(family_name.into()));
        }

        ctx.set_fonts(fonts);
        self.loaded_ui_font = selected_font;
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
        let selected = self.value("DOTFILES_THEME");
        let fallback_accent = match selected.as_str() {
            "midnight" => "#8b98ff",
            "catppuccin" => "#cba6f7",
            "nord" => "#88c0d0",
            "gruvbox" => "#fabd2f",
            "sakura" => "#f5a3c7",
            "cyberpunk" => "#00e5ff",
            "everforest" => "#a7c080",
            "dracula" => "#bd93f9",
            "light" => "#2563eb",
            _ => "#89b4fa",
        };

        let accent = parse_hex_color(&self.value("DOTFILES_ACCENT"))
            .or_else(|| parse_hex_color(fallback_accent))
            .unwrap_or(Color32::from_rgb(137, 180, 250));

        match selected.as_str() {
            "light" => AppTheme {
                accent,
                bg: Color32::from_rgb(238, 242, 248),
                sidebar: Color32::from_rgb(248, 250, 252),
                panel: Color32::from_rgb(255, 255, 255),
                panel_soft: Color32::from_rgb(241, 245, 249),
                input: Color32::from_rgb(226, 232, 240),
                card: Color32::from_rgb(255, 255, 255),
                card_hover: Color32::from_rgb(239, 246, 255),
                text: Color32::from_rgb(15, 23, 42),
                muted: Color32::from_rgb(100, 116, 139),
                border: Color32::from_rgb(203, 213, 225),
                success: Color32::from_rgb(22, 163, 74),
                danger: Color32::from_rgb(220, 38, 38),
            },
            "nord" => AppTheme {
                accent,
                bg: Color32::from_rgb(36, 41, 51),
                sidebar: Color32::from_rgb(46, 52, 64),
                panel: Color32::from_rgb(59, 66, 82),
                panel_soft: Color32::from_rgb(67, 76, 94),
                input: Color32::from_rgb(46, 52, 64),
                card: Color32::from_rgb(59, 66, 82),
                card_hover: Color32::from_rgb(76, 86, 106),
                text: Color32::from_rgb(236, 239, 244),
                muted: Color32::from_rgb(216, 222, 233),
                border: Color32::from_rgb(76, 86, 106),
                success: Color32::from_rgb(163, 190, 140),
                danger: Color32::from_rgb(191, 97, 106),
            },
            "gruvbox" => AppTheme {
                accent,
                bg: Color32::from_rgb(29, 32, 33),
                sidebar: Color32::from_rgb(40, 40, 40),
                panel: Color32::from_rgb(50, 48, 47),
                panel_soft: Color32::from_rgb(60, 56, 54),
            input: Color32::from_rgb(50, 48, 47),
                card: Color32::from_rgb(50, 48, 47),
                card_hover: Color32::from_rgb(80, 73, 69),
                text: Color32::from_rgb(235, 219, 178),
                muted: Color32::from_rgb(189, 174, 147),
                border: Color32::from_rgb(102, 92, 84),
                success: Color32::from_rgb(184, 187, 38),
                danger: Color32::from_rgb(251, 73, 52),
            },
            "sakura" => AppTheme {
                accent,
                bg: Color32::from_rgb(25, 20, 32),
                sidebar: Color32::from_rgb(36, 27, 46),
                panel: Color32::from_rgb(45, 34, 58),
                panel_soft: Color32::from_rgb(57, 42, 73),
                input: Color32::from_rgb(54, 43, 55),
                card: Color32::from_rgb(52, 38, 66),
                card_hover: Color32::from_rgb(74, 52, 92),
                text: Color32::from_rgb(255, 235, 246),
                muted: Color32::from_rgb(221, 176, 205),
                border: Color32::from_rgb(96, 64, 111),
                success: Color32::from_rgb(164, 244, 207),
                danger: Color32::from_rgb(255, 119, 164),
            },
            "cyberpunk" => AppTheme {
                accent,
                bg: Color32::from_rgb(4, 8, 20),
                sidebar: Color32::from_rgb(8, 12, 30),
                panel: Color32::from_rgb(12, 18, 42),
                panel_soft: Color32::from_rgb(18, 26, 58),
                input: Color32::from_rgb(25, 32, 48),
                card: Color32::from_rgb(16, 23, 52),
                card_hover: Color32::from_rgb(26, 36, 78),
                text: Color32::from_rgb(232, 252, 255),
                muted: Color32::from_rgb(135, 205, 218),
                border: Color32::from_rgb(37, 77, 102),
                success: Color32::from_rgb(57, 255, 136),
                danger: Color32::from_rgb(255, 52, 118),
            },
            "everforest" => AppTheme {
                accent,
                bg: Color32::from_rgb(35, 42, 46),
                sidebar: Color32::from_rgb(45, 53, 59),
                panel: Color32::from_rgb(52, 63, 68),
                panel_soft: Color32::from_rgb(61, 72, 77),
                input: Color32::from_rgb(37, 45, 47),
                card: Color32::from_rgb(52, 63, 68),
                card_hover: Color32::from_rgb(75, 86, 89),
                text: Color32::from_rgb(211, 198, 170),
                muted: Color32::from_rgb(168, 153, 132),
                border: Color32::from_rgb(88, 99, 99),
                success: Color32::from_rgb(167, 192, 128),
                danger: Color32::from_rgb(230, 126, 128),
            },
            "dracula" => AppTheme {
                accent,
                bg: Color32::from_rgb(40, 42, 54),
                sidebar: Color32::from_rgb(33, 34, 44),
                panel: Color32::from_rgb(49, 50, 68),
                panel_soft: Color32::from_rgb(58, 59, 78),
            input: Color32::from_rgb(48, 49, 65),
                card: Color32::from_rgb(49, 50, 68),
                card_hover: Color32::from_rgb(68, 71, 90),
                text: Color32::from_rgb(248, 248, 242),
                muted: Color32::from_rgb(189, 147, 249),
                border: Color32::from_rgb(98, 114, 164),
                success: Color32::from_rgb(80, 250, 123),
                danger: Color32::from_rgb(255, 85, 85),
            },
            _ => AppTheme {
                accent,
                bg: Color32::from_rgb(10, 12, 18),
                sidebar: Color32::from_rgb(16, 19, 29),
                panel: Color32::from_rgb(20, 24, 36),
                panel_soft: Color32::from_rgb(27, 32, 48),
                input: Color32::from_rgb(37, 38, 52),
                card: Color32::from_rgb(30, 36, 54),
                card_hover: Color32::from_rgb(38, 46, 68),
                text: Color32::from_rgb(226, 232, 240),
                muted: Color32::from_rgb(148, 163, 184),
                border: Color32::from_rgb(51, 65, 85),
                success: Color32::from_rgb(134, 239, 172),
                danger: Color32::from_rgb(251, 113, 133),
            },
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

    // Avoid hard black text fields. egui uses `extreme_bg_color` for many
    // TextEdit backgrounds, so keep it theme-specific and softer.
    visuals.extreme_bg_color = theme.input;

    visuals.widgets.inactive.bg_fill = theme.input;
    visuals.widgets.hovered.bg_fill = theme.panel_soft;
    visuals.widgets.active.bg_fill = theme.panel_soft;
    visuals.widgets.open.bg_fill = theme.panel_soft;
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
        style
            .text_styles
            .insert(egui::TextStyle::Heading, FontId::proportional(25.0));
        style
            .text_styles
            .insert(egui::TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, FontId::proportional(14.0));
        ctx.set_style(style);
    }

    fn keybind_changes_pending(&self) -> bool {
        self.keybinds != self.saved_keybinds
    }

    fn update_keybind_dirty_status(&mut self) {
        if self.keybind_changes_pending() {
            self.status = "Unsaved keybind changes".to_string();
        } else {
            self.status = "No keybind changes".to_string();
        }
    }

    fn save_keybind_changes(&mut self) {
        if !self.keybind_changes_pending() {
            self.status = "No keybind changes to save".to_string();
            return;
        }

        self.save_all();
        self.saved_keybinds = self.keybinds.clone();
        self.status = "Keybind changes saved".to_string();
    }

    fn refresh_addons_if_needed(&mut self) {
        let marker = addons_refresh_marker();

        if marker.exists() {
            let _ = fs::remove_file(&marker);
            self.refresh_after_addon_install();
            self.addon_refresh_marker_seen = true;
        }
    }

    fn refresh_after_addon_install(&mut self) {
        self.values = read_settings(&self.paths);
        self.keybinds = load_keybinds(&self.paths);

        // Refresh visual/generated state that addons may affect.
        if let Err(err) = write_quickshell_bar(&self.paths, &self.values) {
            self.status = format!("Addon installed, but bar refresh failed: {err}");
            return;
        }

        self.status = "Addon installed. Refreshed installed addons.".to_string();
    }

    fn save_all(&mut self) {
        let result = (|| -> Result<(), String> {
            write_local_conf(&self.paths, &self.values)?;
            save_keybinds(&self.paths, &self.keybinds)?;
            save_hypr_values(&self.paths, &self.hypr_values)?;
            write_colors(&self.paths, &self.values)?;
            write_keybinds(&self.paths, &self.values, &self.keybinds)?;
            write_hyprland_settings(&self.paths, &self.hypr_schema, &self.hypr_values)?;
            write_quickshell_bar(&self.paths, &self.values)?;
            self.save_startup_choice()?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.saved_keybinds = self.keybinds.clone();
                self.status = "Saved".to_string();
            }
            Err(err) => {
                self.status = format!("Save failed: {err}");
            }
        }
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
        let wallpaper = self.value("DOTFILES_WALLPAPER").trim().to_string();

        if wallpaper.is_empty() {
            self.status = "Choose a wallpaper first".to_string();
            return;
        }

        let wallpaper_path = expand_home_path(&wallpaper);

        if !wallpaper_path.is_file() {
            self.status = format!("Wallpaper not found: {}", wallpaper_path.display());
            return;
        }

        self.set_value(
            "DOTFILES_WALLPAPER",
            wallpaper_path.to_string_lossy().to_string(),
        );

        if let Err(err) = write_local_conf(&self.paths, &self.values) {
            self.status = format!("Could not save wallpaper: {err}");
            return;
        }

        let result = Command::new(self.paths.script("splinter-wallpaper"))
            .arg("set")
            .arg(wallpaper_path.to_string_lossy().to_string())
            .output();

        self.status = match result {
            Ok(output) if output.status.success() => "Wallpaper applied".to_string(),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

                if !stderr.is_empty() {
                    format!("Wallpaper error: {stderr}")
                } else if !stdout.is_empty() {
                    format!("Wallpaper error: {stdout}")
                } else {
                    "Wallpaper helper returned an error".to_string()
                }
            }
            Err(err) => format!("Could not run wallpaper helper: {err}"),
        };
    }

    fn refresh_wallpapers(&mut self) {
        let dir = self.value("DOTFILES_WALLPAPER_DIR");
        self.wallpaper_images = scan_wallpaper_dir(&dir);

        let valid: std::collections::HashSet<String> = self
            .wallpaper_images
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();

        self.wallpaper_textures
            .retain(|path, _| valid.contains(path));

        self.status = if self.wallpaper_images.is_empty() {
            format!("No images found in {}", expand_home_path(&dir).display())
        } else {
            format!("Found {} wallpapers", self.wallpaper_images.len())
        };
    }

    fn choose_wallpaper_dir(&mut self) {
        let mut picked: Option<String> = None;

        if command_exists("zenity") {
            if let Ok(output) = Command::new("zenity")
                .args([
                    "--file-selection",
                    "--directory",
                    "--title=Choose wallpaper folder",
                ])
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        picked = Some(path);
                    }
                }
            }
        }

        if picked.is_none() && command_exists("kdialog") {
            if let Ok(output) = Command::new("kdialog")
                .args(["--getexistingdirectory", "~", "Choose wallpaper folder"])
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        picked = Some(path);
                    }
                }
            }
        }

        if let Some(path) = picked {
            self.set_value("DOTFILES_WALLPAPER_DIR", path);
            let _ = write_local_conf(&self.paths, &self.values);
            self.refresh_wallpapers();
        } else {
            self.status =
                "Could not open folder picker. Install zenity/kdialog or type the path manually."
                    .to_string();
        }
    }

    fn wallpaper_texture(
        &mut self,
        ctx: &egui::Context,
        path: &Path,
    ) -> Option<egui::TextureHandle> {
        let key = path.to_string_lossy().to_string();

        if let Some(texture) = self.wallpaper_textures.get(&key) {
            return Some(texture.clone());
        }

        // First try to create a cropped preview PNG through ImageMagick.
        // This makes tall/wide wallpapers preview correctly instead of failing.
        let preview_path = wallpaper_preview_cache_path(path);

        if !preview_path.exists() {
            if let Some(parent) = preview_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let _ = Command::new("magick")
                .arg(path)
                .args([
                    "-auto-orient",
                    "-resize",
                    "960x540^",
                    "-gravity",
                    "center",
                    "-extent",
                    "960x540",
                ])
                .arg(&preview_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        let image_path = if preview_path.exists() {
            preview_path.as_path()
        } else {
            path
        };

        let bytes = fs::read(image_path).ok()?;
        let image = image::load_from_memory(&bytes)
            .or_else(|_| {
                image::ImageReader::open(image_path)
                    .map_err(image::ImageError::IoError)?
                    .with_guessed_format()?
                    .decode()
            })
            .ok()?;

        let image = image
            .resize_to_fill(960, 540, image::imageops::FilterType::Lanczos3)
            .to_rgba8();

        let size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);

        let texture = ctx.load_texture(
            format!("wallpaper-preview-{key}"),
            color_image,
            egui::TextureOptions::LINEAR,
        );

        self.wallpaper_textures.insert(key, texture.clone());
        Some(texture)
    }

    fn wallpaper_grid(&mut self, ui: &mut egui::Ui, theme: &AppTheme) {
        if self.wallpaper_images.is_empty() {
            Self::soft_card(theme, ui, |ui| {
                ui.label(RichText::new("No wallpapers found in this folder.").color(theme.muted));
                ui.label(
                    RichText::new("Supported formats: png, jpg, jpeg, webp, gif, bmp, tiff, avif")
                        .color(theme.muted)
                        .size(12.0),
                );
            });
            return;
        }

        let current = expand_home_path(&self.value("DOTFILES_WALLPAPER"));
        let images = self.wallpaper_images.clone();
        let mut clicked: Option<PathBuf> = None;

        for path in images.iter() {
            let selected = path == &current;
            let texture = self.wallpaper_texture(ui.ctx(), path);

            let preview_width = 340.0_f32.min(ui.available_width() - 36.0);
            let image_size = Vec2::new(preview_width, preview_width * 9.0 / 16.0);

            let padding = 8.0;
            let outer_size = Vec2::new(
                image_size.x + padding * 2.0,
                image_size.y + padding * 2.0,
            );

            // Allocate exactly one row with the exact preview-card height.
            // This prevents egui from vertically centering the wallpaper list
            // inside a larger leftover area.
            let row_width = ui.available_width();
            let (row_rect, _) = ui.allocate_exact_size(
                Vec2::new(row_width, outer_size.y),
                egui::Sense::hover(),
            );

            let outer_rect = egui::Rect::from_center_size(
                egui::pos2(row_rect.center().x, row_rect.min.y + outer_size.y / 2.0),
                outer_size,
            );

            let response = ui.interact(
                outer_rect,
                ui.make_persistent_id(path.to_string_lossy()),
                egui::Sense::click(),
            );

            ui.painter().rect(
                outer_rect,
                CornerRadius::same(14),
                if selected { theme.card_hover } else { theme.panel_soft },
                Stroke::new(
                    if selected { 2.0 } else { 1.0 },
                    if selected { theme.accent } else { theme.border },
                ),
                egui::StrokeKind::Inside,
            );

            let image_rect = egui::Rect::from_min_size(
                outer_rect.min + egui::vec2(padding, padding),
                image_size,
            );

            if let Some(texture) = texture {
                ui.painter().image(
                    texture.id(),
                    image_rect,
                    egui::Rect::from_min_max(
                        egui::pos2(0.0, 0.0),
                        egui::pos2(1.0, 1.0),
                    ),
                    Color32::WHITE,
                );
            } else {
                ui.painter().rect_filled(
                    image_rect,
                    CornerRadius::same(10),
                    theme.input,
                );
                ui.painter().text(
                    image_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Could not preview",
                    FontId::proportional(14.0),
                    theme.muted,
                );
            }

            if response.clicked() {
                clicked = Some(path.clone());
            }

            ui.add_space(6.0);
        }

        if let Some(path) = clicked {
            self.set_value("DOTFILES_WALLPAPER", path.to_string_lossy().to_string());
            self.apply_wallpaper();
        }
    }


    fn install_style_package(&mut self, package: &str) {
        let terminal = self.value("DOTFILES_TERMINAL");
        let helper = self.paths.script("splinterdots-style");
        let spicetify_setup = self.paths.script("splinter-setup-spicetify");

        let package_list = package
            .split_whitespace()
            .filter(|pkg| !pkg.trim().is_empty())
            .collect::<Vec<_>>();

        if package_list.is_empty() {
            self.status = "No package selected".to_string();
            return;
        }

        let mut commands = package_list
            .iter()
            .map(|pkg| {
                format!(
                    "{} install {}",
                    shell_quote(&helper.to_string_lossy()),
                    shell_quote(pkg),
                )
            })
            .collect::<Vec<_>>();

        if package_list.contains(&"spicetify-cli") || package_list.contains(&"spicetify") {
            commands.push(shell_quote(&spicetify_setup.to_string_lossy()).to_string());
        }

        let command = format!(
            "{}; mkdir -p \"${{XDG_CACHE_HOME:-$HOME/.cache}}/splinterdots\"; touch \"${{XDG_CACHE_HOME:-$HOME/.cache}}/splinterdots/addons-refresh\"; echo; read -rp 'Press enter to close...'",
            commands.join(" && "),
        );

        let result = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "{} -e sh -c {}",
                shell_quote(&terminal),
                shell_quote(&command),
            ))
            .spawn();

        self.status = match result {
            Ok(_) => {
                self.configure_addon_after_install(package);
                format!("Opening installer for {package}")
            }
            Err(err) => format!("Could not open installer: {err}"),
        };
    }

    fn apply_style_pack(&mut self) {
        let _ = write_local_conf(&self.paths, &self.values);

        let result = Command::new(self.paths.script("splinterdots-style"))
            .arg("apply")
            .output();

        self.status = match result {
            Ok(output) if output.status.success() => "Font and icon settings applied".to_string(),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if stderr.is_empty() {
                    "Style helper returned an error".to_string()
                } else {
                    format!("Style error: {stderr}")
                }
            }
            Err(err) => format!("Could not run style helper: {err}"),
        };
    }

    fn uninstall_addon_package(&mut self, package: &str) {
        let terminal = self.value("DOTFILES_TERMINAL");
        let command = format!(
            "sudo pacman -Rns {}; echo; read -rp 'Press enter to close...'",
            shell_quote(package)
        );

        let result = Command::new("sh")
            .arg("-c")
            .arg(format!("{} -e sh -c {}", terminal, shell_quote(&command)))
            .spawn();

        self.status = match result {
            Ok(_) => format!("Opening remover for {package}"),
            Err(err) => format!("Could not open remover: {err}"),
        };
    }

    fn configure_addon_after_install(&mut self, package: &str) {
        let packages = package.split_whitespace().collect::<Vec<_>>();

        if packages.contains(&"vesktop-bin") {
                let config_home = env::var_os("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home_dir().join(".config"));

                let theme_dir = config_home.join("vesktop").join("themes");
                let plugin_dir = config_home.join("vesktop").join("plugins");

                let _ = fs::create_dir_all(&theme_dir);
                let _ = fs::create_dir_all(&plugin_dir);

                let theme_file = theme_dir.join("SplinterDots.theme.css");
                let repo_theme_file = self
                    .paths
                    .root
                    .join("files")
                    .join("vesktop")
                    .join("SplinterDots.theme.css");

                if repo_theme_file.exists() {
                    let _ = fs::copy(repo_theme_file, theme_file);
                } else if !theme_file.exists() {
                    let _ = fs::write(
                        theme_file,
                        "/* SplinterDots Vesktop theme file was missing from the repo. */
",
                    );
                }
        }

        if packages.contains(&"spicetify-cli") {
                let config_home = env::var_os("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home_dir().join(".config"));

                let spicetify_dir = config_home.join("spicetify");
                let theme_dir = spicetify_dir.join("Themes").join("SplinterDots");
                let ext_dir = spicetify_dir.join("Extensions");

                let _ = fs::create_dir_all(&theme_dir);
                let _ = fs::create_dir_all(&ext_dir);

                let color_file = theme_dir.join("color.ini");
                if !color_file.exists() {
                    let _ = fs::write(
                        &color_file,
                        r#"[SplinterDots]
text               = cdd6f4
subtext            = bac2de
main               = 1e1e2e
sidebar            = 181825
player             = 181825
card               = 313244
shadow             = 11111b
selected-row       = 45475a
button             = 89b4fa
button-active      = 89b4fa
button-disabled    = 6c7086
tab-active         = 313244
notification       = 313244
notification-error = f38ba8
misc               = 313244
"#,
                    );
                }

                let user_css = theme_dir.join("user.css");
                if !user_css.exists() {
                    let _ = fs::write(
                        &user_css,
                        r#"/* SplinterDots Spicetify theme placeholder.
   Customize Spotify CSS here. */
"#,
                    );
                }

                let _ = Command::new("spicetify")
                    .args(["config", "current_theme", "SplinterDots"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();

                let _ = Command::new("spicetify")
                    .args(["config", "color_scheme", "SplinterDots"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
        }
    }

    fn install_addon_package(&mut self, package: &str) {
        let terminal = self.value("DOTFILES_TERMINAL");
        let helper = self.paths.script("splinterdots-style");

        let packages = package
            .split_whitespace()
            .map(shell_quote)
            .collect::<Vec<_>>()
            .join(" ");

        let command = format!(
            "for pkg in {packages}; do {helper} install \"$pkg\" || exit 1; done; \
             mkdir -p \"${{XDG_CACHE_HOME:-$HOME/.cache}}/splinterdots\"; \
             touch \"${{XDG_CACHE_HOME:-$HOME/.cache}}/splinterdots/addons-refresh\"; \
             echo; read -rp 'Press enter to close...'",
            packages = packages,
            helper = shell_quote(&helper.to_string_lossy()),
        );

        let result = Command::new("sh")
            .arg("-c")
            .arg(format!("{} -e sh -c {}", terminal, shell_quote(&command)))
            .spawn();

        self.status = match result {
            Ok(_) => {
                self.configure_addon_after_install(package);
                format!("Opening installer for {package}")
            }
            Err(err) => format!("Could not open installer: {err}"),
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

    fn keybind_hint_rows(&self) -> Vec<(String, String)> {
        let wanted = [
            ("App launcher", "Open apps"),
            ("Terminal", "Terminal"),
            ("SplinterDots", "SplinterDots"),
            ("Reload desktop", "Reload desktop"),
        ];

        let mut rows = Vec::new();
        for (name, label) in wanted {
            if let Some(sc) = self
                .keybinds
                .iter()
                .find(|sc| sc.name.eq_ignore_ascii_case(name))
            {
                rows.push((format_keybind_key(&sc.key), label.to_string()));
            }
        }

        rows.push(("Super + Left Mouse".to_string(), "Drag windows".to_string()));
        rows.push((
            "Super + Right Mouse".to_string(),
            "Resize windows".to_string(),
        ));
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
                    ui.label(
                        RichText::new("SplinterDots")
                            .heading()
                            .color(theme.text)
                            .strong(),
                    );
                    ui.label(RichText::new("Arch + Hyprland").color(theme.muted));
                });

                ui.add_space(24.0);

                for tab in [
                    Tab::Overview,
                    Tab::Hyprland,
                    Tab::Keybinds,
                    Tab::Appearance,
                    Tab::Addons,
                    Tab::Bar,
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
        let fill = if selected {
            theme.accent
        } else {
            theme.panel_soft
        };
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
                ui.label(icon_text(tab.icon(), 18.0, text));
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
                let subtitle = self.tab.subtitle();
                if !subtitle.trim().is_empty() {
                    ui.label(RichText::new(subtitle).color(theme.muted));
                }
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                pill(ui, &theme, &self.value("DOTFILES_THEME"));
            });
        });

        ui.add_space(14.0);
    }

    fn card<R>(
        theme: &AppTheme,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
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

    fn soft_card<R>(
        theme: &AppTheme,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
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

        Self::card(&theme, ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(icon_text("", 34.0, theme.accent));
                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("System overview")
                            .size(22.0)
                            .strong()
                            .color(theme.text),
                    );
                    ui.label(
                        RichText::new("Live system information from fastfetch.")
                            .color(theme.muted),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Refresh").clicked() {
                        self.fastfetch_output = load_fastfetch_output();
                    }
                });
            });
        });

        Self::card(&theme, ui, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(ui.available_height() - 20.0)
                .show(ui, |ui| {
                    Frame {
                        fill: theme.panel_soft,
                        corner_radius: CornerRadius::same(14),
                        inner_margin: Margin::same(16),
                        stroke: Stroke::new(1.0, theme.border),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        for line in self.fastfetch_output.lines() {
                            ui.monospace(line);
                        }
                    });
                });
        });
    }

    fn stat_card(&self, ui: &mut egui::Ui, label: &str, value: &str, note: &str) {
        let theme = self.app_theme();

        Self::card(&theme, ui, |ui| {
            ui.set_min_size(Vec2::new(190.0, 88.0));
            ui.label(RichText::new(label).color(theme.muted));
            ui.label(RichText::new(value).size(20.0).strong().color(theme.text));
            ui.label(RichText::new(note).color(theme.muted));
        });
    }

    fn hyprland_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        Self::card(&theme, ui, |ui| {
            ui.label(RichText::new("Safe Hyprland settings").size(18.0).strong());
            ui.label(
                RichText::new("Only beginner-friendly switches, dropdowns, and number controls are shown here.")
                    .color(theme.muted),
            );

            ui.add_space(8.0);
            ui.label(RichText::new("Search settings").color(theme.muted));
            ui.add_sized(
                [ui.available_width(), 42.0],
                egui::TextEdit::singleline(&mut self.hypr_search)
                    .hint_text("Search by name, section, path, or type...")
                    .desired_width(f32::INFINITY),
            );
        });

        let search = self.hypr_search.trim().to_ascii_lowercase();

        let mut grouped: BTreeMap<String, Vec<HyprOption>> = BTreeMap::new();
        for option in &self.hypr_schema {
            let section = option
                .section
                .clone()
                .unwrap_or_else(|| "Other".to_string());
            let label = option
                .label
                .clone()
                .unwrap_or_else(|| option.path.replace(':', " / ").replace('_', " "));
            let kind = option.kind.clone().unwrap_or_else(|| "text".to_string());

            let haystack = format!(
                "{} {} {} {}",
                section.to_ascii_lowercase(),
                label.to_ascii_lowercase(),
                option.path.to_ascii_lowercase(),
                kind.to_ascii_lowercase()
            );

            if !search.is_empty() && !haystack.contains(&search) {
                continue;
            }

            grouped.entry(section).or_default().push(option.clone());
        }

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if grouped.is_empty() {
                    Self::card(&theme, ui, |ui| {
                        ui.label(
                            RichText::new("No Hyprland settings matched your search.")
                                .color(theme.muted),
                        );
                    });
                    return;
                }

                for (section, options) in grouped {
                    Self::card(&theme, ui, |ui| {
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
                    ui.label(
                        RichText::new(path.clone())
                            .color(theme.muted)
                            .monospace()
                            .size(11.0),
                    );
                });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    match kind.as_str() {
                        "bool" => {
                            let mut checked = is_true(current);
                            if ui.checkbox(&mut checked, "").changed() {
                                *current = if checked { "true" } else { "false" }.to_string();
                            }
                        }
                        "choice" => {
                            let choices = option.choices.clone().unwrap_or_default();
                            egui::ComboBox::from_id_salt(format!("hypr-{path}"))
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
                            if ui
                                .add(egui::DragValue::new(&mut value).range(min..=max))
                                .changed()
                            {
                                *current = value.to_string();
                            }
                        }
                        "float" => {
                            let min = option.min.unwrap_or(0.0);
                            let max = option.max.unwrap_or(9999.0);
                            let mut value = current.parse::<f64>().unwrap_or(min);
                            if ui
                                .add(
                                    egui::DragValue::new(&mut value)
                                        .speed(0.05)
                                        .range(min..=max),
                                )
                                .changed()
                            {
                                *current = format_float(value);
                            }
                        }
                        _ => {}
                    }
                });
            });
        });
        ui.add_space(6.0);
    }

    fn keybinds_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        ui.label(
            RichText::new("Keybinds")
                .strong()
                .size(28.0)
                .color(theme.text),
        );
        ui.add_space(14.0);

        let panel_height = 430.0;

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.columns(2, |columns| {
                Frame {
                    fill: theme.card,
                    corner_radius: CornerRadius::same(22),
                    inner_margin: Margin::symmetric(20, 18),
                    stroke: Stroke::new(1.0, theme.border),
                    ..Default::default()
                }
                .show(&mut columns[0], |ui| {
                    ui.set_min_height(panel_height);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Current keybinds").strong().size(18.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("+ Add").clicked() {
                                self.keybinds.push(Keybind {
                                    name: "New keybind".to_string(),
                                    key: "SUPER, N".to_string(),
                                    kind: "app".to_string(),
                                    value: self.value("DOTFILES_TERMINAL"),
                                });
                                self.selected_keybind = Some(self.keybinds.len().saturating_sub(1));
                                self.update_keybind_dirty_status();
                            }
                        });
                    });

                    ui.add_space(8.0);

                    for (index, keybind) in self.keybinds.iter().enumerate() {
                        let selected = self.selected_keybind == Some(index);

                        Frame {
                            fill: if selected { theme.card_hover } else { theme.panel_soft },
                            corner_radius: CornerRadius::same(16),
                            inner_margin: Margin::symmetric(12, 9),
                            stroke: Stroke::new(
                                if selected { 2.0 } else { 1.0 },
                                if selected { theme.accent } else { theme.border },
                            ),
                            ..Default::default()
                        }
                        .show(ui, |ui| {
                            let response = ui
                                .add_sized(
                                    [ui.available_width(), 28.0],
                                    egui::SelectableLabel::new(
                                        selected,
                                        RichText::new(keybind_display(keybind)).size(14.0),
                                    ),
                                );

                            if response.clicked() {
                                self.selected_keybind = Some(index);
                            }
                        });

                        ui.add_space(8.0);
                    }

                    ui.add_space(6.0);

                    ui.horizontal_wrapped(|ui| {
                        if primary_button(ui, &theme, if self.keybind_changes_pending() { "Save changes *" } else { "Saved" }).clicked() {
                            self.save_keybind_changes();
                        }

                        if ui.button("Reset").clicked() {
                            self.keybinds = default_keybinds();
                            self.selected_keybind = None;
                            self.update_keybind_dirty_status();
                        }
                    });
                });

                Frame {
                    fill: theme.card,
                    corner_radius: CornerRadius::same(22),
                    inner_margin: Margin::symmetric(20, 18),
                    stroke: Stroke::new(1.0, theme.border),
                    ..Default::default()
                }
                .show(&mut columns[1], |ui| {
                    ui.set_min_height(panel_height);

                    ui.label(RichText::new("Edit selected").strong().size(18.0));
                    ui.add_space(8.0);

                    let Some(index) = self.selected_keybind else {
                        ui.label(RichText::new("Select a keybind on the left to edit it.").color(theme.muted));
                        return;
                    };

                    if index >= self.keybinds.len() {
                        self.selected_keybind = None;
                        return;
                    }

                    let mut changed = false;
                    let mut delete = false;

                    {
                        let keybind = &mut self.keybinds[index];

                        ui.label(RichText::new("Name").color(theme.muted));
                        changed |= ui
                            .add_sized(
                                [ui.available_width(), 34.0],
                                egui::TextEdit::singleline(&mut keybind.name),
                            )
                            .changed();

                        ui.add_space(6.0);

                        
                        changed |= ui
                            .add_sized(
                                [ui.available_width(), 34.0],
                                egui::TextEdit::singleline(&mut keybind.key),
                            )
                            .changed();

                        ui.label(
                            RichText::new("Examples: RETURN · D · SHIFT, S · CTRL, ALT, T")
                                .color(theme.muted)
                                .size(11.0),
                        );

                        ui.add_space(8.0);

                        ui.label(RichText::new("Type").color(theme.muted));
                        egui::ComboBox::from_id_salt("keybind-kind-editor")
                            .selected_text(match keybind.kind.as_str() {
                                "app" => "App / script",
                                "command" => "Command",
                                "hyprland" => "Hyprland action",
                                _ => "App / script",
                            })
                            .show_ui(ui, |ui| {
                                changed |= ui
                                    .selectable_value(&mut keybind.kind, "app".to_string(), "App / script")
                                    .changed();
                                changed |= ui
                                    .selectable_value(&mut keybind.kind, "command".to_string(), "Command")
                                    .changed();
                                changed |= ui
                                    .selectable_value(&mut keybind.kind, "hyprland".to_string(), "Hyprland action")
                                    .changed();
                            });

                        ui.add_space(8.0);

                        ui.label(RichText::new("What should happen").color(theme.muted));
                        changed |= ui
                            .add_sized(
                                [ui.available_width(), 34.0],
                                egui::TextEdit::singleline(&mut keybind.value),
                            )
                            .changed();

                        ui.add_space(18.0);

                        ui.horizontal(|ui| {
                            if primary_button(ui, &theme, if self.keybind_changes_pending() { "Save changes *" } else { "Saved" }).clicked() {
                                self.save_keybind_changes();
                            }

                            if ui.button("Delete keybind").clicked() {
                                delete = true;
                            }
                        });
                    }

                    if delete {
                        self.keybinds.remove(index);
                        self.selected_keybind = None;
                        self.update_keybind_dirty_status();
                    } else if changed {
                        self.update_keybind_dirty_status();
                    }
                });
            });

            ui.add_space(16.0);

            Self::card(&theme, ui, |ui| {
                ui.label(RichText::new("Mouse controls").strong());
                ui.label(
                    RichText::new("Super + left mouse drag moves windows. Super + right mouse drag resizes windows.")
                        .color(theme.muted),
                );
            });
        });
    }

    fn appearance_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.label(RichText::new("Appearance").strong().size(28.0));
                ui.label(
                    RichText::new("Themes, fonts, icons, cursor theme, and wallpapers.")
                        .color(theme.muted),
                );
                ui.add_space(18.0);

                ui.columns(3, |columns| {
                    // COLUMN 1: Theme
                    Self::card(&theme, &mut columns[0], |ui| {
                        ui.label(RichText::new("Theme").strong().size(18.0));
                        ui.label(
                            RichText::new("Choose the overall SplinterDots look.")
                                .color(theme.muted),
                        );
                        ui.add_space(8.0);

                        let mut theme_value = self.value("DOTFILES_THEME");

                        egui::Grid::new("theme-pill-grid")
                            .num_columns(2)
                            .spacing([10.0, 10.0])
                            .show(ui, |ui| {
                                for (i, choice) in [
                                    ("midnight", "Midnight"),
                                    ("catppuccin", "Catppuccin"),
                                    ("nord", "Nord"),
                                    ("gruvbox", "Gruvbox"),
                                    ("sakura", "Sakura"),
                                    ("cyberpunk", "Cyberpunk"),
                                    ("everforest", "Everforest"),
                                    ("dracula", "Dracula"),
                                ]
                                .iter()
                                .enumerate()
                                {
                                    selectable_pill(ui, &mut theme_value, choice.0, choice.1);
                                    if i % 2 == 1 {
                                        ui.end_row();
                                    }
                                }
                            });

                        self.set_value("DOTFILES_THEME", theme_value);

                        ui.add_space(16.0);

                        let mut accent = self.value("DOTFILES_ACCENT");
                        labeled_text(ui, "Accent color", &mut accent);
                        self.set_value("DOTFILES_ACCENT", accent);

                        ui.add_space(8.0);

                        if primary_button(ui, &theme, "Apply theme").clicked() {
                            self.save_all();
                            self.status = "Theme updated".to_string();
                        }
                    });

                    // COLUMN 2: Wallpaper
                    Self::card(&theme, &mut columns[1], |ui| {
                        ui.label(RichText::new("Wallpaper").strong().size(18.0));
                        ui.label(
                            RichText::new("Choose a folder and click a wallpaper preview.")
                                .color(theme.muted),
                        );
                        ui.add_space(6.0);

                        let mut wallpaper_dir = self.value("DOTFILES_WALLPAPER_DIR");
                        ui.label(RichText::new("Wallpaper folder").color(theme.muted));

                        if ui
                            .text_edit_singleline(&mut wallpaper_dir)
                            .on_hover_text("Example: ~/Pictures/Wallpapers")
                            .changed()
                        {
                            self.set_value("DOTFILES_WALLPAPER_DIR", wallpaper_dir);
                            let _ = write_local_conf(&self.paths, &self.values);
                        }

                        ui.add_space(8.0);

                        ui.horizontal_wrapped(|ui| {
                            if primary_button(ui, &theme, "Scan folder").clicked() {
                                self.refresh_wallpapers();
                            }

                            if ui.button("Choose folder").clicked() {
                                self.choose_wallpaper_dir();
                            }

                            if ui.button("Use ~/Pictures/Wallpapers").clicked() {
                                self.set_value(
                                    "DOTFILES_WALLPAPER_DIR",
                                    "~/Pictures/Wallpapers".to_string(),
                                );
                                let _ = write_local_conf(&self.paths, &self.values);
                                self.refresh_wallpapers();
                            }
                        });

                        ui.add_space(8.0);

                        let current = self.value("DOTFILES_WALLPAPER");
                        if !current.trim().is_empty() {
                            ui.label(
                                RichText::new(format!("Current: {}", shorten_text(&current, 48)))
                                    .color(theme.muted),
                            );
                        }

                        self.wallpaper_grid(ui, &theme);
                    });

                    // COLUMN 3: Fonts
                    Self::card(&theme, &mut columns[2], |ui| {
                        ui.label(RichText::new("Fonts and icon packs").strong().size(18.0));
                        ui.label(
                            RichText::new("Items marked as requiring download need to be installed separately.")
                                .color(theme.muted),
                        );
                        ui.add_space(14.0);

                        ScrollArea::vertical()
                            .id_salt("appearance-fonts-scroll")
                            .max_height(620.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                let mut font = self.value("DOTFILES_UI_FONT");
                                if let Some(package) = style_choice_grid(
                                    ui,
                                    &theme,
                                    "UI font",
                                    &mut font,
                                    font_choices(),
                                    &self.preview_font_families,
                                ) {
                                    self.install_style_package(package);
                                }
                                self.set_value("DOTFILES_UI_FONT", font);

                                ui.add_space(18.0);

                                let mut icon_theme = self.value("DOTFILES_ICON_THEME");
                                if let Some(package) = style_choice_grid(
                                    ui,
                                    &theme,
                                    "System icon theme",
                                    &mut icon_theme,
                                    icon_theme_choices(),
                                    &self.preview_font_families,
                                ) {
                                    self.install_style_package(package);
                                }
                                self.set_value("DOTFILES_ICON_THEME", icon_theme);

                                ui.add_space(18.0);

                                let mut cursor_theme = self.value("DOTFILES_CURSOR_THEME");
                                if let Some(package) = style_choice_grid(
                                    ui,
                                    &theme,
                                    "Cursor theme",
                                    &mut cursor_theme,
                                    cursor_theme_choices(),
                                    &self.preview_font_families,
                                ) {
                                    self.install_style_package(package);
                                }
                                self.set_value("DOTFILES_CURSOR_THEME", cursor_theme);

                                ui.add_space(18.0);

                                if primary_button(ui, &theme, "Apply font and icon settings").clicked() {
                                    self.apply_style_pack();
                                }
                            });
                    });
                });
            });
    }

    fn addons_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            Self::card(&theme, ui, |ui| {
                ui.label(RichText::new("Addons").strong().size(22.0));
                ui.label(
                    RichText::new("Optional extras are grouped, searchable, and installed only when you choose them.")
                        .color(theme.muted),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Every addon comes preconfigured after install, and you can change it yourself later in SplinterDots.")
                        .color(theme.muted)
                        .size(12.0),
                );
            });

            Self::card(&theme, ui, |ui| {
                ui.label(RichText::new("Filters").strong().size(17.0));
                ui.add_space(8.0);

                ui.add_sized(
                    [ui.available_width(), 42.0],
                    egui::TextEdit::singleline(&mut self.addon_search)
                        .hint_text("Search for an addon...")
                        .desired_width(f32::INFINITY),
                );

                ui.add_space(8.0);

                egui::ComboBox::from_id_salt("addon-category-filter")
                    .selected_text(self.addon_category_filter.clone())
                    .width(260.0)
                    .show_ui(ui, |ui| {
                        for category in addon_categories() {
                            ui.selectable_value(
                                &mut self.addon_category_filter,
                                category.to_string(),
                                *category,
                            );
                        }
                    });
            });

            ui.add_space(8.0);
            ui.label(RichText::new("Addons").strong().size(18.0));
            ui.add_space(8.0);

            let addons: Vec<AddonChoice> = addon_choices()
                .iter()
                .copied()
                .filter(|addon| {
                    addon_matches_filter(
                        addon,
                        &self.addon_search,
                        &self.addon_category_filter,
                    )
                })
                .collect();

            if addons.is_empty() {
                Self::card(&theme, ui, |ui| {
                    ui.label(RichText::new("No addons matched your search.").color(theme.muted));
                });
                return;
            }

            egui::Grid::new("addons-plugin-grid")
                .num_columns(3)
                .spacing([12.0, 12.0])
                .show(ui, |ui| {
                    for (index, addon) in addons.iter().enumerate() {
                        let installed = addon_package_installed(addon.package);

                        Frame {
                            fill: theme.panel_soft,
                            corner_radius: CornerRadius::same(14),
                            inner_margin: Margin::symmetric(14, 12),
                            stroke: Stroke::new(1.0, theme.border),
                            ..Default::default()
                        }
                        .show(ui, |ui| {
                            ui.set_min_width(260.0);
                            ui.set_max_width(260.0);
                            ui.set_min_height(145.0);

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(addon.name).strong().size(15.0));

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if installed {
                                            if ui.button("🗑").on_hover_text("Remove addon").clicked() {
                                                self.uninstall_addon_package(addon.package);
                                            }
                                        } else if ui.button("⬇").on_hover_text("Download / install").clicked() {
                                            self.install_addon_package(addon.package);
                                        }
                                    });
                                });

                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new(addon.category)
                                        .color(theme.muted)
                                        .size(11.0),
                                );

                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(addon.description)
                                        .color(theme.text)
                                        .size(12.0),
                                );

                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new("Preconfigured after install. Customizable later.")
                                        .color(theme.muted)
                                        .size(11.0),
                                );

                                ui.add_space(8.0);
                                ui.monospace(addon.package);

                                ui.add_space(6.0);
                                if installed {
                                    ui.label(
                                        RichText::new("Installed")
                                            .color(theme.success)
                                            .size(11.0)
                                            .strong(),
                                    );
                                } else {
                                    ui.label(
                                        RichText::new("Not installed")
                                            .color(theme.muted)
                                            .size(11.0),
                                    );
                                }
                            });
                        });

                        if index % 3 == 2 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn widget_zone_editor(&mut self, ui: &mut egui::Ui, theme: &AppTheme, title: &str, zone: &str) {
        let key = format!("DOTFILES_BAR_{}_WIDGETS", zone.to_uppercase());
        let mut widgets = split_csv(&self.value(&key));
        let mut drop_index: Option<usize> = None;

        Self::card(theme, ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).strong().size(16.0));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("drop zone")
                            .color(theme.muted)
                            .size(11.0),
                    );
                });
            });

            ui.add_space(10.0);

            let zone_response = Frame {
                fill: theme.panel_soft,
                corner_radius: CornerRadius::same(18),
                inner_margin: Margin::same(10),
                stroke: Stroke::new(
                    if self.dragged_bar_widget.is_some() { 2.0 } else { 1.0 },
                    if self.dragged_bar_widget.is_some() { theme.accent } else { theme.border },
                ),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.set_min_height(120.0);

                if widgets.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(22.0);
                        ui.label(
                            RichText::new("Drop widgets here")
                                .color(theme.muted)
                                .size(13.0),
                        );
                        ui.add_space(22.0);
                    });
                }

                let mut remove_index: Option<usize> = None;

                for (index, widget) in widgets.clone().iter().enumerate() {
                    let is_dragged = self
                        .dragged_bar_widget
                        .as_ref()
                        .map(|(from_zone, from_index)| from_zone == zone && *from_index == index)
                        .unwrap_or(false);

                    let card_response = Frame {
                        fill: if is_dragged {
                            theme.panel_soft
                        } else {
                            theme.card
                        },
                        corner_radius: CornerRadius::same(14),
                        inner_margin: Margin::symmetric(10, 8),
                        stroke: Stroke::new(
                            if is_dragged { 2.0 } else { 1.0 },
                            if is_dragged { theme.accent } else { theme.border },
                        ),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("󰍢").color(theme.muted));
                            ui.label(
                                RichText::new(widget_label(widget))
                                    .color(theme.text)
                                    .strong(),
                            );

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let remove = ui.add(
                                    egui::Button::new(
                                        RichText::new("×")
                                            .color(theme.danger)
                                            .strong()
                                            .size(16.0),
                                    )
                                    .min_size(Vec2::new(26.0, 24.0)),
                                );

                                if remove.clicked() {
                                    remove_index = Some(index);
                                    ui.ctx().request_repaint();
                                }
                            });
                        });
                    })
                    .response;

                    // Only the left/middle part of the card is draggable.
                    // The right side is reserved for the remove button.
                    let mut drag_rect = card_response.rect;
                    drag_rect.max.x = (drag_rect.max.x - 46.0).max(drag_rect.min.x);

                    let drag_response = ui.interact(
                        drag_rect,
                        ui.make_persistent_id(format!("bar_widget_drag_{zone}_{index}")),
                        egui::Sense::click_and_drag(),
                    );

                    if drag_response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                    }

                    if drag_response.drag_started() {
                        self.dragged_bar_widget = Some((zone.to_string(), index));
                    }

                    if self.dragged_bar_widget.is_some() && card_response.hovered() {
                        drop_index = Some(index);

                        let marker_y = card_response.rect.top();
                        ui.painter().line_segment(
                            [
                                egui::pos2(card_response.rect.left(), marker_y),
                                egui::pos2(card_response.rect.right(), marker_y),
                            ],
                            Stroke::new(2.0, theme.accent),
                        );
                    }

                    ui.add_space(6.0);
                }

                if let Some(index) = remove_index {
                    if index < widgets.len() {
                        let removed = widget_label(&widgets[index]);
                        widgets.remove(index);
                        self.set_value(&key, widgets.join(","));
                        self.sync_widget_toggles_from_layout();
                        self.status = format!("Removed {removed} widget");
                    }
                }

                ui.add_space(8.0);

                ui.menu_button("Add widget...", |ui| {
                    ui.set_min_width(260.0);

                    for choice in bar_widget_choices() {
                        let already_added = widgets.iter().any(|item| item == choice.id);
                        let label = if already_added {
                            format!("✓ {}", choice.label)
                        } else {
                            choice.label.to_string()
                        };

                        if ui
                            .add_enabled(!already_added, egui::Button::new(label))
                            .clicked()
                        {
                            widgets.push(choice.id.to_string());
                            self.set_value(&key, widgets.join(","));
                            self.set_bool_value(choice.setting, true);
                            self.status = format!("Added {} widget", choice.label);
                            ui.close();
                        }
                    }
                });
            })
            .response;

            let mouse_released = ui.input(|input| input.pointer.any_released());
            let pointer_pos = ui.input(|input| input.pointer.interact_pos());
            let pointer_inside_zone = pointer_pos
                .map(|pos| zone_response.rect.contains(pos))
                .unwrap_or(false);

            if mouse_released && pointer_inside_zone {
                if let Some((from_zone, from_index)) = self.dragged_bar_widget.take() {
                    let from_key = format!("DOTFILES_BAR_{}_WIDGETS", from_zone.to_uppercase());
                    let mut from_widgets = split_csv(&self.value(&from_key));
                    let mut to_widgets = split_csv(&self.value(&key));

                    if from_index < from_widgets.len() {
                        let moved = from_widgets.remove(from_index);

                        let mut insert_at = drop_index.unwrap_or(to_widgets.len());

                        if from_zone == zone {
                            to_widgets = from_widgets.clone();

                            if from_index < insert_at {
                                insert_at = insert_at.saturating_sub(1);
                            }
                        }

                        insert_at = insert_at.min(to_widgets.len());
                        to_widgets.insert(insert_at, moved);

                        self.set_value(&from_key, from_widgets.join(","));
                        self.set_value(&key, to_widgets.join(","));
                        self.status = "Bar widget moved".to_string();
                    }
                }
            }
        });
    }


    fn persist_bar_layout(&mut self) {
        self.sync_widget_toggles_from_layout();

        match write_local_conf(&self.paths, &self.values) {
            Ok(()) => {
                self.status = "Bar layout saved".to_string();
            }
            Err(err) => {
                self.status = format!("Could not save bar layout: {err}");
            }
        }
    }

    fn paint_dragged_bar_widget(&self, ui: &mut egui::Ui, theme: &AppTheme) {
        let Some((zone, index)) = self.dragged_bar_widget.as_ref() else {
            return;
        };

        let key = format!("DOTFILES_BAR_{}_WIDGETS", zone.to_uppercase());
        let widgets = split_csv(&self.value(&key));

        let Some(widget) = widgets.get(*index) else {
            return;
        };

        let Some(pointer_pos) = ui.input(|input| input.pointer.interact_pos()) else {
            return;
        };

        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

        let label = widget_label(widget);
        let preview_size = Vec2::new(190.0, 42.0);
        let preview_pos = pointer_pos + egui::vec2(22.0, 18.0);
        let rect = egui::Rect::from_min_size(preview_pos, preview_size);

        let painter = ui.ctx().layer_painter(egui::LayerId::new(
            egui::Order::Tooltip,
            egui::Id::new("splinter-dragged-bar-widget-preview"),
        ));

        painter.rect(
            rect,
            CornerRadius::same(16),
            theme.card_hover,
            Stroke::new(2.0, theme.accent),
            egui::StrokeKind::Inside,
        );

        painter.text(
            rect.left_center() + egui::vec2(14.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "󰍢",
            FontId::proportional(18.0),
            theme.muted,
        );

        painter.text(
            rect.left_center() + egui::vec2(42.0, 0.0),
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(15.0),
            theme.text,
        );
    }


    fn sync_widget_toggles_from_layout(&mut self) {
        let mut active: Vec<String> = Vec::new();

        for zone in ["left", "center", "right"] {
            active.extend(parse_widget_zone(&self.value(zone_key(zone))));
        }

        for choice in bar_widget_choices() {
            self.set_bool_value(choice.setting, active.iter().any(|item| item == choice.id));
        }
    }


    fn bar_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            Self::card(&theme, ui, |ui| {
                ui.label(RichText::new("Visual bar layout").strong().size(20.0));
                ui.label(
                    RichText::new(
                        "Drag widgets between the Left, Center, and Right bubbles. Drag inside a bubble to reorder them.",
                    )
                    .color(theme.muted),
                );
                ui.add_space(14.0);

                ui.columns(3, |columns| {
                    self.widget_zone_editor(&mut columns[0], &theme, "Left", "left");
                    self.widget_zone_editor(&mut columns[1], &theme, "Center", "center");
                    self.widget_zone_editor(&mut columns[2], &theme, "Right", "right");
                });

                self.paint_dragged_bar_widget(ui, &theme);

                ui.add_space(8.0);

                ui.horizontal(|ui| {

        ui.horizontal(|ui| {
            ui.label("Bar position");

            let mut pos = self
                .values
                .get("DOTFILES_BAR_POSITION")
                .cloned()
                .unwrap_or_else(|| "top".to_string());

            let top_clicked = ui.selectable_label(pos == "top", "Top").clicked();
            let bottom_clicked = ui.selectable_label(pos == "bottom", "Bottom").clicked();

            if top_clicked {
                pos = "top".to_string();
            }

            if bottom_clicked {
                pos = "bottom".to_string();
            }

            if self.values.get("DOTFILES_BAR_POSITION") != Some(&pos) {
                self.values.insert("DOTFILES_BAR_POSITION".to_string(), pos);
            }
        });

        ui.add_space(8.0);
                    if primary_button(ui, &theme, "Save and restart bar").clicked() {
                        self.sync_widget_toggles_from_layout();
                        self.save_and_restart_bar();
                    }

                    if ui.button("Restart bar only").clicked() {
                        self.restart_bar();
                    }
                });
            });

            Self::card(&theme, ui, |ui| {
                ui.label(RichText::new("Layout and speed").strong().size(18.0));

                combo_value(ui, &mut self.values, "DOTFILES_BAR_POSITION", &["top", "bottom"]);
                combo_value(
                    ui,
                    &mut self.values,
                    "DOTFILES_BAR_WORKSPACE_COUNT",
                    &["5", "6", "7", "8", "9", "10", "12", "15", "20"],
                );

                slider_value(ui, &theme, &mut self.values, "DOTFILES_BAR_REACTIVE_MS", 40.0, 1000.0);
                slider_value(ui, &theme, &mut self.values, "DOTFILES_BAR_STATUS_MS", 500.0, 6000.0);
                slider_value(ui, &theme, &mut self.values, "DOTFILES_BAR_HEIGHT", 24.0, 72.0);
                slider_value(ui, &theme, &mut self.values, "DOTFILES_BAR_RADIUS", 0.0, 28.0);
                slider_value(ui, &theme, &mut self.values, "DOTFILES_BAR_OPACITY", 0.2, 1.0);
                slider_value(ui, &theme, &mut self.values, "DOTFILES_BAR_FONT_SIZE", 8.0, 24.0);
                slider_value(ui, &theme, &mut self.values, "DOTFILES_BAR_SPACING", 4.0, 40.0);
                slider_value(ui, &theme, &mut self.values, "DOTFILES_BAR_BORDER_WIDTH", 0.0, 4.0);

                combo_value(ui, &mut self.values, "DOTFILES_BAR_ICON_PACK", &["nerd", "fontawesome", "text"]);
                combo_value(
                    ui,
                    &mut self.values,
                    "DOTFILES_BAR_ICON_FONT",
                    &["Symbols Nerd Font", "Font Awesome 6 Free", "Font Awesome 6 Brands", "Sans"],
                );
            });
        });
    }

    fn widgets_tab(&mut self, ui: &mut egui::Ui) {
        self.bar_tab(ui);
    }

    fn setup_tab(&mut self, ui: &mut egui::Ui) {
        let theme = self.app_theme();

        Self::card(&theme, ui, |ui| {
            ui.label(RichText::new("Default apps").strong().size(18.0));
            ui.label(
                RichText::new("These are used by keybinds and helper actions.").color(theme.muted),
            );
            ui.add_space(6.0);

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

        Self::card(&theme, ui, |ui| {
            ui.label(RichText::new("Quick actions").strong().size(18.0));
            ui.add_space(6.0);

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
    ("board", "DOTFILES_BAR_SHOW_KEYBOARD"),
];

impl eframe::App for SplinterDots {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.sync_ui_font(ctx);
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
                        if primary_button(
                            ui,
                            &theme,
                            if self.keybind_changes_pending() {
                                "Save changes *"
                            } else {
                                "Saved"
                            },
                        )
                        .clicked()
                        {
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
                    Tab::Keybinds => self.keybinds_tab(ui),
                    Tab::Appearance => self.appearance_tab(ui),
                    Tab::Addons => self.addons_tab(ui),
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
        let value = values
            .get(*key)
            .cloned()
            .unwrap_or_else(|| (*default).to_string());
        lines.push(format!("{key}={}", shell_quote(&value)));
    }

    fs::write(&paths.local_conf, lines.join("\n") + "\n").map_err(err_string)
}

fn shell_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn default_keybinds() -> Vec<Keybind> {
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
        sc(
            "Screenshot region",
            "S",
            "app",
            "splinter-screenshot region",
        ),
        sc(
            "Screenshot full screen",
            "SHIFT, S",
            "app",
            "splinter-screenshot full",
        ),
        sc("SplinterDots", "W", "app", "dotctl center"),
        sc("Reload desktop", "SHIFT, R", "app", "hyprctl reload"),
    ]
}

fn sc(name: &str, key: &str, kind: &str, value: &str) -> Keybind {
    Keybind {
        name: name.to_string(),
        key: key.to_string(),
        kind: kind.to_string(),
        value: value.to_string(),
    }
}

fn load_keybinds(paths: &Paths) -> Vec<Keybind> {
    let Ok(content) = fs::read_to_string(&paths.keybinds_json) else {
        return default_keybinds();
    };

    serde_json::from_str::<Vec<Keybind>>(&content).unwrap_or_else(|_| default_keybinds())
}

fn save_keybinds(paths: &Paths, keybinds: &[Keybind]) -> Result<(), String> {
    ensure_dir(&paths.dotfiles_dir)?;
    let text = serde_json::to_string_pretty(keybinds).map_err(err_string)?;
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
    keybinds: &[Keybind],
) -> Result<(), String> {
    ensure_dir(&paths.hypr_dir)?;

    let mut lines = vec![
        "$mainMod = SUPER".to_string(),
        format!("$terminal = {}", value_or(values, "DOTFILES_TERMINAL")),
        format!(
            "$fileManager = {}",
            value_or(values, "DOTFILES_FILE_MANAGER")
        ),
        format!("$menu = {}", value_or(values, "DOTFILES_APP_LAUNCHER")),
        format!("$browser = {}", value_or(values, "DOTFILES_BROWSER")),
        String::new(),
        "# Hold Super + left mouse button to drag windows.".to_string(),
        "bindm = $mainMod, mouse:272, movewindow".to_string(),
        "# Hold Super + right mouse button to resize windows.".to_string(),
        "bindm = $mainMod, mouse:273, resizewindow".to_string(),
        String::new(),
    ];

    for keybind in keybinds {
        let (modifier, key) = hypr_key_parts(&keybind.key);
        if keybind.kind == "app" {
            lines.push(format!(
                "bind = {modifier}, {key}, exec, {}",
                keybind.value
            ));
        } else {
            lines.push(format!("bind = {modifier}, {key}, {}", keybind.value));
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
            key.split_once(',')
                .map(|(_, k)| k.trim())
                .unwrap_or("")
                .to_string(),
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
            value
                .parse::<i64>()
                .unwrap_or(min)
                .clamp(min, max)
                .to_string()
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
    let selected = value_or(values, "DOTFILES_THEME");
    let accent = value_or(values, "DOTFILES_ACCENT");

    match selected.as_str() {
        "light" => Palette {
            accent,
            background: "#f8fafc".to_string(),
            surface: "#e2e8f0".to_string(),
            text: "#0f172a".to_string(),
            muted: "#475569".to_string(),
            inactive_border: "#94a3b8".to_string(),
            bar_rgb: "f8fafc".to_string(),
            active_text: "#ffffff".to_string(),
        },
        "nord" => Palette {
            accent,
            background: "#2e3440".to_string(),
            surface: "#3b4252".to_string(),
            text: "#eceff4".to_string(),
            muted: "#d8dee9".to_string(),
            inactive_border: "#4c566a".to_string(),
            bar_rgb: "2e3440".to_string(),
            active_text: "#2e3440".to_string(),
        },
        "gruvbox" => Palette {
            accent,
            background: "#282828".to_string(),
            surface: "#3c3836".to_string(),
            text: "#ebdbb2".to_string(),
            muted: "#bdae93".to_string(),
            inactive_border: "#665c54".to_string(),
            bar_rgb: "282828".to_string(),
            active_text: "#1d2021".to_string(),
        },
        "sakura" => Palette {
            accent,
            background: "#241b2e".to_string(),
            surface: "#392a49".to_string(),
            text: "#ffebf6".to_string(),
            muted: "#ddb0cd".to_string(),
            inactive_border: "#60406f".to_string(),
            bar_rgb: "241b2e".to_string(),
            active_text: "#191420".to_string(),
        },
        "cyberpunk" => Palette {
            accent,
            background: "#080c1e".to_string(),
            surface: "#121a3a".to_string(),
            text: "#e8fcff".to_string(),
            muted: "#87cdda".to_string(),
            inactive_border: "#254d66".to_string(),
            bar_rgb: "080c1e".to_string(),
            active_text: "#040814".to_string(),
        },
        "everforest" => Palette {
            accent,
            background: "#2d353b".to_string(),
            surface: "#3d484d".to_string(),
            text: "#d3c6aa".to_string(),
            muted: "#a89984".to_string(),
            inactive_border: "#586363".to_string(),
            bar_rgb: "2d353b".to_string(),
            active_text: "#232a2e".to_string(),
        },
        "dracula" => Palette {
            accent,
            background: "#282a36".to_string(),
            surface: "#3a3b4e".to_string(),
            text: "#f8f8f2".to_string(),
            muted: "#bd93f9".to_string(),
            inactive_border: "#6272a4".to_string(),
            bar_rgb: "282a36".to_string(),
            active_text: "#282a36".to_string(),
        },
        _ => Palette {
            accent,
            background: "#1e1e2e".to_string(),
            surface: "#313244".to_string(),
            text: "#cdd6f4".to_string(),
            muted: "#bac2de".to_string(),
            inactive_border: "#45475a".to_string(),
            bar_rgb: "1e1e2e".to_string(),
            active_text: "#11111b".to_string(),
        },
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

    let bg_alpha = (opacity * 255.0) as i32;
    let bg_color = format!("#{:02x}{}", bg_alpha, palette.bar_rgb);

    let left_section = build_bar_zone_qml(
        "left",
        values,
        &palette,
        &icons,
        height.into(),
        radius.into(),
        font_size.into(),
        workspace_count.into(),
        status_ms.into(),
        &icon_font,
    );

    let center_section = build_bar_zone_qml(
        "center",
        values,
        &palette,
        &icons,
        height.into(),
        radius.into(),
        font_size.into(),
        workspace_count.into(),
        status_ms.into(),
        &icon_font,
    );

    let right_section = build_bar_zone_qml(
        "right",
        values,
        &palette,
        &icons,
        height.into(),
        radius.into(),
        font_size.into(),
        workspace_count.into(),
        status_ms.into(),
        &icon_font,
    );

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
        onTriggered: { stateProc.running = false; stateProc.running = true }
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

          Row {
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
            spacing: 7
__LEFT_SECTION__
          }

          Item { Layout.fillWidth: true }

          Row {
            Layout.alignment: Qt.AlignCenter
            spacing: 7
__CENTER_SECTION__
          }

          Item { Layout.fillWidth: true }

          Row {
            Layout.alignment: Qt.AlignRight | Qt.AlignVCenter
            spacing: 7
__RIGHT_SECTION__
          }
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

fn build_bar_zone_qml(
    zone: &str,
    values: &HashMap<String, String>,
    palette: &Palette,
    icons: &IconSet,
    height: i64,
    radius: i64,
    font_size: i64,
    workspace_count: i64,
    status_ms: i64,
    icon_font: &str,
) -> String {
    let key = format!("DOTFILES_BAR_{}_WIDGETS", zone.to_uppercase());
    let raw = value_or(values, &key);

    let widgets = if raw.trim().is_empty() {
        Vec::new()
    } else {
        split_csv(&raw)
    };

    widgets
        .iter()
        .enumerate()
        .map(|(index, widget)| {
            bar_widget_qml(
                widget,
                &format!("{}_{}", zone, index),
                values,
                palette,
                icons,
                height,
                radius,
                font_size,
                workspace_count,
                status_ms,
                icon_font,
            )
        })
        .collect::<Vec<_>>()
        .join("
")
}

fn bar_widget_qml(
    widget: &str,
    id: &str,
    values: &HashMap<String, String>,
    palette: &Palette,
    icons: &IconSet,
    height: i64,
    radius: i64,
    font_size: i64,
    workspace_count: i64,
    status_ms: i64,
    icon_font: &str,
) -> String {
    match widget {
        "workspaces" => workspaces_qml(id, palette, height, radius, font_size, workspace_count, icon_font),
            "visualizer" => visualizer_widget_qml(id, palette, height, radius, font_size),
            "datetime" => datetime_widget_qml(id, palette, height, radius, font_size),
        "window-title" => command_text_qml(id, palette, font_size, icon_font, "hyprctl activewindow -j 2>/dev/null | jq -r '.title // empty' | cut -c1-60", status_ms, None),
        "submap" => command_text_qml(id, palette, font_size, icon_font, "hyprctl submap 2>/dev/null | grep -v '^$' | sed 's/^/󰌌 /'", status_ms, None),

        "easyeffects" => command_button_qml(id, palette, height, radius, font_size, icon_font, "󰓃", "pgrep -x easyeffects >/dev/null && pkill easyeffects || easyeffects --gapplication-service"),
        "volume" => command_text_qml(id, palette, font_size, icon_font, "wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null | awk '{v=int($2*100); if($3==\"[MUTED]\") print \"󰝟 muted\"; else print \" \" v \"%\"}'", status_ms, None),
        "mic" => command_text_qml(id, palette, font_size, icon_font, "wpctl get-volume @DEFAULT_AUDIO_SOURCE@ 2>/dev/null | awk '{v=int($2*100); if($3==\"[MUTED]\") print \" muted\"; else print \" \" v \"%\"}'", status_ms, None),
        "network" => command_text_qml(id, palette, font_size, icon_font, "nmcli -t -f DEVICE,STATE device 2>/dev/null | awk -F: '$2==\"connected\"{print \"󰤨 \" $1; exit}'", status_ms, None),
        "bluetooth" => command_text_qml(id, palette, font_size, icon_font, "bluetoothctl show 2>/dev/null | grep -q 'Powered: yes' && printf ' on' || printf ' off'", status_ms, None),
        "battery" => command_text_qml(id, palette, font_size, icon_font, "bat=$(cat /sys/class/power_supply/BAT0/capacity 2>/dev/null || cat /sys/class/power_supply/BAT1/capacity 2>/dev/null); [ -n \"$bat\" ] && printf '󰁹 %s%%' \"$bat\"", status_ms, None),
        "brightness" => command_text_qml(id, palette, font_size, icon_font, "brightnessctl -m 2>/dev/null | awk -F, '{print \"󰃠 \" $4}'", status_ms, None),
        "updates" => {
            let command = value_or(values, "DOTFILES_WIDGET_UPDATES_COMMAND");
            command_text_qml(id, palette, font_size, icon_font, &format!("upd=$({}); [ -n \"$upd\" ] && [ \"$upd\" != \"0\" ] && printf '󰚰 %s' \"$upd\"", command), status_ms * 4, None)
        }

        "cpu" => command_text_qml(id, palette, font_size, icon_font, r#"top -bn1 | awk -F'[, ]+' '/Cpu\(s\)/{print " " int($2+$4) "%"}'"#, status_ms, None),
        "memory" => command_text_qml(id, palette, font_size, icon_font, "free -m | awk '/^Mem/{printf \" %dMB\", $3}'", status_ms, None),
        "temp" => command_text_qml(id, palette, font_size, icon_font, "sensors 2>/dev/null | awk '/Package id 0|Tctl|temp1/{gsub(/[+°C]/, \"\", $2); print \" \" int($2) \"°C\"; exit}'", status_ms, None),
        "disk" => command_text_qml(id, palette, font_size, icon_font, "df -h / 2>/dev/null | awk 'NR==2{print \"󰋊 \" $5}'", status_ms, None),
        "gpu" => command_text_qml(id, palette, font_size, icon_font, "nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits 2>/dev/null | awk '{print \"󰢮 \" $1 \"%\"}'", status_ms, None),
        "media_controls" | "media" | "media_controller" | "media-controller" | "media_prev" | "media-play" | "media-next" => media_controls_qml(id, palette, height, radius, font_size, icon_font),

        "launcher" => command_button_qml(id, palette, height, radius, font_size, icon_font, "󰀻", "wofi --show drun"),
        "power" => command_button_qml(id, palette, height, radius, font_size, icon_font, "⏻", "wlogout || systemctl poweroff"),
        "lock" => command_button_qml(id, palette, height, radius, font_size, icon_font, "", "hyprlock || loginctl lock-session"),
        "screenshot" => command_button_qml(id, palette, height, radius, font_size, icon_font, "󰄀", "splinter-screenshot area"),
        "wallpaper" => command_button_qml(id, palette, height, radius, font_size, icon_font, "", "dir=\"$HOME/Pictures/Wallpapers\"; file=$(find \"$dir\" -maxdepth 1 -type f | shuf -n1); [ -n \"$file\" ] && splinter-wallpaper \"$file\""),
        "color-picker" => command_button_qml(id, palette, height, radius, font_size, icon_font, "", "hyprpicker -a"),
        "night-light" => command_button_qml(id, palette, height, radius, font_size, icon_font, "󰖔", "pkill wlsunset || wlsunset -t 3400 -T 6500 &"),

        "weather" => command_text_qml(id, palette, font_size, icon_font, "curl -fsS 'wttr.in/?format=1' 2>/dev/null", 900_000, None),
        "notes" => command_button_qml(id, palette, height, radius, font_size, icon_font, "󰎞", "xdg-open \"$HOME/Notes\""),
        "todo" => command_button_qml(id, palette, height, radius, font_size, icon_font, "󰄬", "xdg-open \"$HOME/todo.txt\""),
        "keyboard" => command_text_qml(id, palette, font_size, icon_font, "hyprctl devices -j 2>/dev/null | grep -m1 -o '\"active_keymap\":\"[^\"]*' | cut -d'\"' -f4 | sed 's/^/ /'", status_ms, None),

        _ => command_text_qml(id, palette, font_size, icon_font, &format!("printf {}", shell_escape(widget)), status_ms, None),
    }
}

fn command_text_qml(
    id: &str,
    palette: &Palette,
    font_size: i64,
    icon_font: &str,
    command: &str,
    interval: i64,
    prefix_icon: Option<&str>,
) -> String {
    let prefix = prefix_icon.map(|icon| format!("{icon} ")).unwrap_or_default();

    r#"
            Text {
              id: __ID__
              anchors.verticalCenter: parent.verticalCenter
              color: "__TEXT__"
              font.pixelSize: __FONT_SIZE__
              font.family: "__ICON_FONT__"
              text: ""

              Process {
                id: __PROC_ID__
                command: ["sh", "-c", __COMMAND__]
                running: true
                stdout: StdioCollector {
                  onStreamFinished: __ID__.text = "__PREFIX__" + this.text.split("\\n").join("").trim()
                }
              }

              Timer {
                interval: __INTERVAL__
                running: true
                repeat: true
                onTriggered: { __PROC_ID__.running = false; __PROC_ID__.running = true }
              }
            }
"#
    .replace("__ID__", &qml_id(id, "txt"))
    .replace("__PROC_ID__", &qml_id(id, "proc"))
    .replace("__TEXT__", &palette.text)
    .replace("__FONT_SIZE__", &font_size.to_string())
    .replace("__ICON_FONT__", icon_font)
    .replace("__COMMAND__", &json_string(command))
    .replace("__INTERVAL__", &interval.to_string())
    .replace("__PREFIX__", &qml_string_escape(&prefix))
}

fn command_button_qml(
    id: &str,
    palette: &Palette,
    height: i64,
    radius: i64,
    font_size: i64,
    icon_font: &str,
    label: &str,
    command: &str,
) -> String {
    let button_height = (height - 8).max(22);
    let button_radius = (radius - 4).max(6);

    r#"
            Rectangle {
              width: __WIDTH__
              height: __HEIGHT__
              anchors.verticalCenter: parent.verticalCenter
              radius: __RADIUS__
              color: "__SURFACE__"

              Text {
                anchors.centerIn: parent
                text: "__LABEL__"
                color: "__TEXT__"
                font.pixelSize: __FONT_SIZE__
                font.family: "__ICON_FONT__"
                font.bold: true
              }

              MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                onEntered: parent.color = "__ACCENT__"
                onExited: parent.color = "__SURFACE__"
                onClicked: { __PROC_ID__.running = false; __PROC_ID__.running = true }
              }

              Process {
                id: __PROC_ID__
                command: ["sh", "-c", __COMMAND__]
              }
            }
"#
    .replace("__WIDTH__", &((label.chars().count() as i64 * 11).max(30)).to_string())
    .replace("__HEIGHT__", &button_height.to_string())
    .replace("__RADIUS__", &button_radius.to_string())
    .replace("__SURFACE__", &palette.surface)
    .replace("__ACCENT__", &palette.accent)
    .replace("__TEXT__", &palette.text)
    .replace("__FONT_SIZE__", &font_size.to_string())
    .replace("__ICON_FONT__", icon_font)
    .replace("__LABEL__", &qml_string_escape(label))
    .replace("__PROC_ID__", &qml_id(id, "btnProc"))
    .replace("__COMMAND__", &json_string(command))
}

fn workspaces_qml(
    id: &str,
    palette: &Palette,
    height: i64,
    radius: i64,
    font_size: i64,
    workspace_count: i64,
    icon_font: &str,
) -> String {
    let item_id = qml_id(id, "workspaces");

    format!(
        r#"
            Row {{
              id: __ID__
              anchors.verticalCenter: parent.verticalCenter
              spacing: 5

              Repeater {{
                model: __WORKSPACE_COUNT__

                delegate: Rectangle {{
                  width: __BUTTON_SIZE__
                  height: __BUTTON_SIZE__
                  radius: __BUTTON_RADIUS__
                  color: "__SURFACE__"
                  border.color: "__MUTED__"
                  border.width: 1

                  Text {{
                    anchors.centerIn: parent
                    text: modelData + 1
                    color: "__TEXT__"
                    font.pixelSize: __FONT_SIZE__
                    font.family: "__ICON_FONT__"
                  }}

                  MouseArea {{
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {{
                      Hyprland.dispatch("workspace " + (modelData + 1))
                    }}
                  }}
                }}
              }}
            }}
"#,
    )
    .replace("__ID__", &item_id)
    .replace("__WORKSPACE_COUNT__", &workspace_count.to_string())
    .replace("__BUTTON_SIZE__", &(height - 8).max(20).to_string())
    .replace("__BUTTON_RADIUS__", &(radius - 4).max(4).to_string())
    .replace("__FONT_SIZE__", &font_size.to_string())
    .replace("__ICON_FONT__", &qml_string_escape(icon_font))
    .replace("__SURFACE__", &palette.surface)
    .replace("__TEXT__", &palette.text)
    .replace("__MUTED__", &palette.muted)
}





fn datetime_widget_qml(
    id: &str,
    palette: &Palette,
    height: i64,
    radius: i64,
    font_size: i64,
) -> String {
    format!(
        r#"
        Item {{
            id: {id}
            width: dateBubble.width
            height: {height}
            clip: false

            property string displayText: ""

            Process {{
                id: timeProc
                command: ["sh", "-c", "date '+%a %d %b · %H:%M'"]

                stdout: StdioCollector {{
                    onStreamFinished: {{
                        {id}.displayText = this.text.trim()
                    }}
                }}
            }}

            Process {{
                id: openCalendarProc
                command: ["sh", "-c", "$HOME/.local/bin/splinter-calendar-menu"]
            }}

            Timer {{
                interval: 1000
                running: true
                repeat: true
                onTriggered: {{
                    timeProc.running = false
                    timeProc.running = true
                }}
            }}

            Component.onCompleted: {{
                timeProc.running = true
            }}

            Rectangle {{
                id: dateBubble
                height: Math.max(22, {height} - 10)
                radius: Math.max(10, {radius} - 4)
                color: "{card}"
                border.color: "{border}"
                border.width: 1
                width: textItem.implicitWidth + 20

                anchors.verticalCenter: parent.verticalCenter

                Text {{
                    id: textItem
                    anchors.centerIn: parent
                    text: {id}.displayText
                    color: "{text}"
                    font.pixelSize: Math.max(10, {font_size} - 1)
                    font.bold: true
                }}

                MouseArea {{
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {{
                        openCalendarProc.running = false
                        openCalendarProc.running = true
                    }}
                }}
            }}
        }}
        "#,
        id = id,
        height = height,
        radius = radius,
        font_size = font_size,
        card = palette.surface,
        border = palette.muted,        text = palette.text,

    )
}







fn visualizer_widget_qml(
    id: &str,
    palette: &Palette,
    height: i64,
    radius: i64,
    _font_size: i64,
) -> String {
    format!(
        r#"
        Item {{
            id: {id}
            width: 150
            height: {height}
            clip: false

            property string rawBars: "000000000000000000"

            Process {{
                id: cavaProc
                command: ["sh", "-c", "$HOME/.local/bin/splinter-cava-read"]

                stdout: StdioCollector {{
                    onStreamFinished: {{
                        var raw = this.text.trim()
                        if (raw.length >= 18) {{
                            {id}.rawBars = raw.substring(0, 18)
                        }}
                    }}
                }}
            }}

            Timer {{
                interval: 45
                running: true
                repeat: true
                onTriggered: {{
                    cavaProc.running = false
                    cavaProc.running = true
                }}
            }}

            Component.onCompleted: {{
                cavaProc.running = true
            }}

            Rectangle {{
                id: visualizerBubble
                width: parent.width
                height: Math.max(22, {height} - 10)
                anchors.verticalCenter: parent.verticalCenter
                radius: Math.max(10, {radius} - 4)
                color: "{surface}"
                border.color: "{muted}"
                border.width: 1
                clip: true

                Row {{
                    anchors.centerIn: parent
                    height: parent.height - 8
                    spacing: 3

                    Repeater {{
                        model: 18

                        Rectangle {{
                            width: 5
                            radius: 3
                            anchors.bottom: parent.bottom
                            color: "{accent}"

                            height: Math.max(3, (parent.height * Number({id}.rawBars.charAt(index))) / 8)

                            Behavior on height {{
                                NumberAnimation {{
                                    duration: 90
                                    easing.type: Easing.OutCubic
                                }}
                            }}
                        }}
                    }}
                }}
            }}
        }}
        "#,
        id = id,
        height = height,
        radius = radius,
        surface = palette.surface,
        muted = palette.muted,
        accent = palette.accent,
    )
}




fn media_controls_qml(
    id: &str,
    palette: &Palette,
    height: i64,
    radius: i64,
    font_size: i64,
    icon_font: &str,
) -> String {
    let item_id = qml_id(id, "media");
    let proc_id = qml_id(&format!("{id}_media_menu"), "proc");
    let button_size = (height - 8).max(22);

    format!(
        r#"
            Item {{
              id: __ID__
              width: __BUTTON_SIZE__
              height: __HEIGHT__

              Process {{
                id: __PROC_ID__
                command: ["bash", "-lc", "echo clicked >> /tmp/splinter-media-button.log; exec $HOME/.local/bin/splinter-media-menu >> /tmp/splinter-media-button.log 2>&1"]
              }}

              Rectangle {{
                anchors.centerIn: parent
                width: __BUTTON_SIZE__
                height: __BUTTON_SIZE__
                radius: __BUTTON_RADIUS__
                color: mediaMouse.containsMouse ? "__ACCENT__" : "__SURFACE__"
                border.color: "__ACCENT__"
                border.width: 1

                Text {{
                  anchors.centerIn: parent
                  text: ""
                  color: "__TEXT__"
                  font.family: "__ICON_FONT__"
                  font.pixelSize: __FONT_SIZE__
                }}

                MouseArea {{
                  id: mediaMouse
                  anchors.fill: parent
                  hoverEnabled: true
                  cursorShape: Qt.PointingHandCursor
                  acceptedButtons: Qt.LeftButton

                  onClicked: {{
                    __PROC_ID__.running = false
                    __PROC_ID__.running = true
                  }}
                }}
              }}
            }}
"#,
    )
    .replace("__ID__", &item_id)
    .replace("__PROC_ID__", &proc_id)
    .replace("__HEIGHT__", &height.to_string())
    .replace("__BUTTON_SIZE__", &button_size.to_string())
    .replace("__BUTTON_RADIUS__", &(radius - 4).max(4).to_string())
    .replace("__FONT_SIZE__", &font_size.to_string())
    .replace("__ICON_FONT__", &qml_string_escape(icon_font))
    .replace("__SURFACE__", &palette.surface)
    .replace("__TEXT__", &palette.text)
    .replace("__ACCENT__", &palette.accent)
}


fn qml_id(id: &str, prefix: &str) -> String {
    let safe = id
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();

    format!("{prefix}_{safe}")
}

fn qml_string_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "")
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

fn expand_home_path(value: &str) -> PathBuf {
    if value == "~" {
        return env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
    }

    if let Some(rest) = value.strip_prefix("~/") {
        return env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rest);
    }

    PathBuf::from(value)
}



fn load_wallpaper_texture(
    ctx: &egui::Context,
    path: &Path,
) -> Option<egui::TextureHandle> {
    let bytes = fs::read(path).ok()?;
    let image = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);

    Some(ctx.load_texture(
        path.to_string_lossy(),
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}


fn cover_uv(image_size: Vec2, target_size: Vec2) -> egui::Rect {
    if image_size.x <= 0.0 || image_size.y <= 0.0 || target_size.x <= 0.0 || target_size.y <= 0.0 {
        return egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    }

    let image_aspect = image_size.x / image_size.y;
    let target_aspect = target_size.x / target_size.y;

    if image_aspect > target_aspect {
        let visible_width = target_aspect / image_aspect;
        let crop = (1.0 - visible_width) / 2.0;
        egui::Rect::from_min_max(egui::pos2(crop, 0.0), egui::pos2(1.0 - crop, 1.0))
    } else {
        let visible_height = image_aspect / target_aspect;
        let crop = (1.0 - visible_height) / 2.0;
        egui::Rect::from_min_max(egui::pos2(0.0, crop), egui::pos2(1.0, 1.0 - crop))
    }
}




fn wallpaper_runtime_path(path: &Path) -> PathBuf {
    let preview = wallpaper_preview_cache_path(path);
    if preview.exists() {
        preview
    } else {
        path.to_path_buf()
    }
}

fn wallpaper_preview_cache_path(path: &Path) -> PathBuf {
    let cache_home = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".cache"));

    let modified = fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0);

    let raw = format!("{}-{modified}", path.to_string_lossy());
    let safe = raw
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();

    cache_home
        .join("splinterdots")
        .join("wallpaper-previews")
        .join(format!("{safe}.png"))
}


fn scan_wallpaper_dir(value: &str) -> Vec<PathBuf> {
    let dir = expand_home_path(value);

    let mut images: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_file() && is_wallpaper_image(path))
            .collect(),
        Err(_) => Vec::new(),
    };

    images.sort_by_key(|path| file_name_string(path).to_ascii_lowercase());
    images
}

fn is_wallpaper_image(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };

    matches!(
        ext.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"
    )
}

fn file_name_string(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("wallpaper")
        .to_string()
}

fn shorten_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let keep = max_chars.saturating_sub(1);
    let mut shortened: String = value.chars().take(keep).collect();
    shortened.push('…');
    shortened
}

fn command_exists(program: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!(
            "command -v {} >/dev/null 2>&1",
            shell_quote(program)
        ))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn theme_choices() -> &'static [&'static str] {
    &[
        "midnight",
        "catppuccin",
        "nord",
        "gruvbox",
        "sakura",
        "cyberpunk",
        "everforest",
        "dracula",
    ]
}

fn theme_label(theme: &str) -> &'static str {
    match theme {
        "midnight" => "Midnight",
        "catppuccin" => "Catppuccin",
        "nord" => "Nord",
        "gruvbox" => "Gruvbox",
        "sakura" => "Sakura",
        "cyberpunk" => "Cyberpunk",
        "everforest" => "Everforest",
        "dracula" => "Dracula",
        _ => "Custom",
    }
}

fn install_global_egui_font(ctx: &egui::Context, preferred_font: &str) {
    let mut fonts = egui::FontDefinitions::default();

    let candidates = font_file_candidates(preferred_font);

    for path in candidates {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };

        let font_id = format!("splinter-ui-font-{}", path.display());

        fonts.font_data.insert(
            font_id.clone(),
            FontData::from_owned(bytes).into(),
        );

        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, font_id.clone());

        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, font_id);

        ctx.set_fonts(fonts);
        return;
    }
}

fn font_file_candidates(preferred_font: &str) -> Vec<PathBuf> {
    let wanted = normalize_font_name(preferred_font);
    let mut candidates = Vec::new();

    for dir in font_search_dirs() {
        collect_font_candidates(&dir, &wanted, &mut candidates);
    }

    candidates
}

fn font_search_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/usr/share/fonts"),
        PathBuf::from("/usr/local/share/fonts"),
    ];

    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        dirs.push(home.join(".local/share/fonts"));
        dirs.push(home.join(".fonts"));
    }

    dirs
}

fn collect_font_candidates(dir: &Path, wanted: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect_font_candidates(&path, wanted, out);
            continue;
        }

        let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };

        let ext = ext.to_ascii_lowercase();
        if ext != "ttf" && ext != "otf" {
            continue;
        }

        let name = normalize_font_name(
            &path.file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or_default(),
        );

        if name.contains(wanted)
            || wanted.contains(&name)
            || name.contains(&wanted.replace("nerdfont", ""))
        {
            out.push(path);
        }
    }
}

fn normalize_font_name(name: &str) -> String {
    name.to_ascii_lowercase()
        .replace(" ", "")
        .replace("-", "")
        .replace("_", "")
}

#[derive(Clone, Copy)]
struct StyleChoice {
    label: &'static str,
    value: &'static str,
    package: Option<&'static str>,
}

fn font_choices() -> &'static [StyleChoice] {
    &[
        StyleChoice {
            label: "CaskaydiaCove Nerd Font",
            value: "CaskaydiaCove Nerd Font",
            package: Some("ttf-cascadia-code-nerd"),
        },
        StyleChoice {
            label: "JetBrainsMono Nerd Font",
            value: "JetBrainsMono Nerd Font",
            package: Some("ttf-jetbrains-mono-nerd"),
        },
        StyleChoice {
            label: "FiraCode Nerd Font",
            value: "FiraCode Nerd Font",
            package: Some("ttf-firacode-nerd"),
        },
        StyleChoice {
            label: "Hack Nerd Font",
            value: "Hack Nerd Font",
            package: Some("ttf-hack-nerd"),
        },
        StyleChoice {
            label: "Iosevka Nerd Font",
            value: "Iosevka Nerd Font",
            package: Some("ttf-iosevka-nerd"),
        },
        StyleChoice {
            label: "Ubuntu",
            value: "Ubuntu",
            package: Some("ttf-ubuntu-font-family"),
        },
        StyleChoice {
            label: "Noto Sans",
            value: "Noto Sans",
            package: Some("noto-fonts"),
        },
        StyleChoice {
            label: "DejaVu Sans",
            value: "DejaVu Sans",
            package: Some("ttf-dejavu"),
        },
        StyleChoice {
            label: "Cantarell",
            value: "Cantarell",
            package: Some("cantarell-fonts"),
        },
    ]
}

fn icon_theme_choices() -> &'static [StyleChoice] {
    &[
        StyleChoice {
            label: "Papirus Dark",
            value: "Papirus-Dark",
            package: Some("papirus-icon-theme"),
        },
        StyleChoice {
            label: "Papirus",
            value: "Papirus",
            package: Some("papirus-icon-theme"),
        },
        StyleChoice {
            label: "Breeze Dark",
            value: "breeze-dark",
            package: Some("breeze-icons"),
        },
        StyleChoice {
            label: "Breeze",
            value: "breeze",
            package: Some("breeze-icons"),
        },
        StyleChoice {
            label: "Adwaita",
            value: "Adwaita",
            package: Some("adwaita-icon-theme"),
        },
        StyleChoice {
            label: "Tela Dark",
            value: "Tela-dark",
            package: Some("tela-icon-theme"),
        },
        StyleChoice {
            label: "Qogir Dark",
            value: "Qogir-dark",
            package: Some("qogir-icon-theme"),
        },
        StyleChoice {
            label: "Vimix Dark",
            value: "Vimix-dark",
            package: Some("vimix-icon-theme"),
        },
        StyleChoice {
            label: "Kora",
            value: "kora",
            package: Some("kora-icon-theme"),
        },
        StyleChoice {
            label: "Nordzy Dark",
            value: "Nordzy-dark",
            package: Some("nordzy-icon-theme"),
        },
    ]
}

fn cursor_theme_choices() -> &'static [StyleChoice] {
    &[
        StyleChoice {
            label: "Bibata Modern Ice",
            value: "Bibata-Modern-Ice",
            package: Some("bibata-cursor-theme"),
        },
        StyleChoice {
            label: "Bibata Modern Classic",
            value: "Bibata-Modern-Classic",
            package: Some("bibata-cursor-theme"),
        },
        StyleChoice {
            label: "Breeze",
            value: "Breeze",
            package: Some("breeze"),
        },
        StyleChoice {
            label: "Adwaita",
            value: "Adwaita",
            package: Some("adwaita-cursors"),
        },
        StyleChoice {
            label: "Capitaine",
            value: "capitaine-cursors",
            package: Some("capitaine-cursors"),
        },
        StyleChoice {
            label: "Qogir",
            value: "Qogir",
            package: Some("qogir-cursor-theme"),
        },
    ]
}

fn is_package_installed(package: &str) -> bool {
    package_installed_cached(package)
}

fn style_choice_grid(
    ui: &mut egui::Ui,
    theme: &AppTheme,
    title: &str,
    current: &mut String,
    choices: &[StyleChoice],
    preview_font_families: &HashMap<String, FontFamily>,
) -> Option<&'static str> {
    let mut package_to_install = None;
    let is_font_picker = title.to_ascii_lowercase().contains("font");

    ui.horizontal(|ui| {
        ui.label(RichText::new(title).strong().size(16.0));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new("Click a card to select it")
                    .color(theme.muted)
                    .size(11.0),
            );
        });
    });

    ui.add_space(8.0);

    egui::Grid::new(format!("style-choice-grid-{title}"))
        .num_columns(if is_font_picker { 1 } else { 2 })
        .spacing([8.0, 8.0])
        .show(ui, |ui| {
            for (index, choice) in choices.iter().enumerate() {
                let selected = current == choice.value;
                let installed = choice.package.map(is_package_installed).unwrap_or(true);

                let response = Frame {
                    fill: if selected {
                        theme.card_hover
                    } else {
                        theme.panel_soft
                    },
                    corner_radius: CornerRadius::same(16),
                    inner_margin: Margin::symmetric(12, if is_font_picker { 16 } else { 10 }),
                    stroke: Stroke::new(
                        if selected { 2.0 } else { 1.0 },
                        if selected { theme.accent } else { theme.border },
                    ),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    if is_font_picker {
                        ui.set_min_width(ui.available_width() - 10.0);
                        ui.set_min_height(92.0);
                    } else {
                        ui.set_min_width(200.0);
                    }

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(choice.label)
                                    .strong()
                                    .size(15.0)
                                    .color(theme.text),
                            );

                            if is_font_picker {
                                ui.add_space(5.0);
                                let preview_family = preview_font_families
                                    .get(choice.value)
                                    .cloned()
                                    .unwrap_or(FontFamily::Proportional);

                                ui.label(
                                    RichText::new("AaBbCc 123")
                                        .font(FontId::new(17.0, preview_family))
                                        .color(theme.text),
                                );
                            }

                            if let Some(package) = choice.package {
                                ui.add_space(5.0);
                                ui.label(
                                    RichText::new(package)
                                        .color(theme.muted)
                                        .size(10.0)
                                        .monospace(),
                                );
                            }
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if selected {
                                ui.label(
                                    RichText::new("Selected")
                                        .color(theme.accent)
                                        .size(11.0)
                                        .strong(),
                                );
                            } else if !installed {
                                if ui.button("Install").clicked() {
                                    package_to_install = choice.package;
                                }
                            } else {
                                ui.label(
                                    RichText::new("Installed")
                                        .color(theme.success)
                                        .size(11.0),
                                );
                            }
                        });
                    });
                })
                .response
                .interact(egui::Sense::click());

                if response.clicked() {
                    *current = choice.value.to_string();
                }

                if !is_font_picker && index % 2 == 1 {
                    ui.end_row();
                } else if is_font_picker {
                    ui.end_row();
                }
            }
        });

    package_to_install
}

#[derive(Clone, Copy)]
struct AddonChoice {
    category: &'static str,
    name: &'static str,
    description: &'static str,
    package: &'static str,
}

fn addon_choices() -> &'static [AddonChoice] {
    &[
        // Fonts
        // Icons
        // Cursors
        // Desktop utilities
        AddonChoice {
            category: "Desktop utilities",
            name: "Clipboard Manager",
            description: "Clipboard history for Wayland using cliphist.",
            package: "cliphist",
        },
        AddonChoice {
            category: "Desktop utilities",
            name: "Zenity",
            description: "GUI dialogs for folder and file pickers.",
            package: "zenity",
        },
        AddonChoice {
            category: "Desktop utilities",
            name: "Audio Control",
            description: "Simple PipeWire/PulseAudio volume GUI.",
            package: "pavucontrol",
        },
        AddonChoice {
            category: "Desktop utilities",
            name: "EasyEffects",
            description: "Audio effects and equalizer.",
            package: "easyeffects",
        },
        // Apps and theming

        AddonChoice {
            category: "Apps and theming",
            name: "Vesktop themed bundle",
            description: "Installs Vesktop and creates SplinterDots-ready Vencord theme/plugin folders.",
            package: "vesktop-bin",
        },
        AddonChoice {
            category: "Apps and theming",
            name: "Spotify themed bundle",
            description: "Installs Spotify + Spicetify and creates a matching SplinterDots theme.",
            package: "spotify spicetify-cli",
        },

        // Terminal tools
        AddonChoice {
            category: "Terminal tools",
            name: "Better ls: eza",
            description: "Nicer directory listings.",
            package: "eza",
        },
        AddonChoice {
            category: "Terminal tools",
            name: "Better cat: bat",
            description: "Syntax-highlighted file viewer.",
            package: "bat",
        },
        AddonChoice {
            category: "Terminal tools",
            name: "Fuzzy finder",
            description: "Fast fuzzy search in terminal.",
            package: "fzf",
        },
        AddonChoice {
            category: "Terminal tools",
            name: "Ripgrep",
            description: "Fast recursive text search.",
            package: "ripgrep",
        },
        AddonChoice {
            category: "Terminal tools",
            name: "Zoxide",
            description: "Smarter cd command.",
            package: "zoxide",
        },
        // Diagnostics
        AddonChoice {
            category: "Diagnostics",
            name: "Mesa Utils",
            description: "OpenGL information and debugging tools.",
            package: "mesa-utils",
        },
        AddonChoice {
            category: "Diagnostics",
            name: "Vulkan Tools",
            description: "Vulkan information and debugging tools.",
            package: "vulkan-tools",
        },
        AddonChoice {
            category: "Diagnostics",
            name: "Sensors",
            description: "Temperature and hardware sensor readings.",
            package: "lm_sensors",
        },
    ]
}

fn addon_categories() -> &'static [&'static str] {
    &[
        "Show All",
        "Fonts",
        "Icon packs",
        "Cursor themes",
        "Desktop utilities",
        "Apps and theming",
        "Terminal tools",
        "Diagnostics",
    ]
}

fn addon_matches_filter(addon: &AddonChoice, search: &str, category: &str) -> bool {
    let category_ok = category == "Show All" || addon.category == category;

    if !category_ok {
        return false;
    }

    let search = search.trim().to_ascii_lowercase();

    if search.is_empty() {
        return true;
    }

    let haystack = format!(
        "{} {} {} {}",
        addon.category, addon.name, addon.description, addon.package
    )
    .to_ascii_lowercase();

    haystack.contains(&search)
}

fn addon_package_installed(package: &str) -> bool {
    let packages = package.split_whitespace().collect::<Vec<_>>();

    if packages.is_empty() {
        return false;
    }

    packages.iter().all(|pkg| match *pkg {
        "vesktop-bin" => {
            package_installed_cached("vesktop-bin")
                || package_installed_cached("vesktop")
        }
        "spicetify-cli" => {
            package_installed_cached("spicetify-cli")
                || package_installed_cached("spicetify")
        }
        other => package_installed_cached(other),
    })
}

fn package_installed_cached(package: &str) -> bool {
    static INSTALLED_PACKAGES: std::sync::OnceLock<std::collections::HashSet<String>> =
        std::sync::OnceLock::new();

    let packages = INSTALLED_PACKAGES.get_or_init(|| {
        Command::new("pacman")
            .arg("-Qq")
            .output()
            .ok()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect::<std::collections::HashSet<String>>()
            })
            .unwrap_or_default()
    });

    packages.contains(package)
}

#[derive(Clone, Copy)]
struct BarWidgetChoice {
    id: &'static str,
    label: &'static str,
    setting: &'static str,
}


fn bar_widget_choices() -> &'static [BarWidgetChoice] {
    &[
        BarWidgetChoice { id: "workspaces",         label: "Workspaces",            setting: "DOTFILES_WIDGET_WORKSPACES" },
        BarWidgetChoice {
            id: "datetime",
            label: "Date & Time",
            setting: "DOTFILES_WIDGET_DATETIME",
        },
        BarWidgetChoice { id: "easyeffects",        label: "EasyEffects",          setting: "DOTFILES_WIDGET_EASYEFFECTS" },
        BarWidgetChoice { id: "media_controls",     label: "Media Controls",       setting: "DOTFILES_WIDGET_MEDIA" },
        BarWidgetChoice {
            id: "visualizer",
            label: "Visualizer",
            setting: "DOTFILES_WIDGET_VISUALIZER",
        },
        BarWidgetChoice { id: "volume",             label: "Volume",                setting: "DOTFILES_WIDGET_VOLUME" },
        BarWidgetChoice { id: "microphone",         label: "Microphone",            setting: "DOTFILES_WIDGET_MICROPHONE" },
        BarWidgetChoice { id: "network",            label: "Network",               setting: "DOTFILES_WIDGET_NETWORK" },
        BarWidgetChoice { id: "bluetooth",          label: "Bluetooth",             setting: "DOTFILES_WIDGET_BLUETOOTH" },
        BarWidgetChoice { id: "battery",            label: "Battery",               setting: "DOTFILES_WIDGET_BATTERY" },
        BarWidgetChoice { id: "brightness",         label: "Brightness",            setting: "DOTFILES_WIDGET_BRIGHTNESS" },

        BarWidgetChoice { id: "cpu",                label: "CPU",                   setting: "DOTFILES_WIDGET_CPU" },
        BarWidgetChoice { id: "memory",             label: "Memory",                setting: "DOTFILES_WIDGET_MEMORY" },
        BarWidgetChoice { id: "temp",               label: "Temperature",           setting: "DOTFILES_WIDGET_TEMP" },
        BarWidgetChoice { id: "disk",               label: "Disk",                  setting: "DOTFILES_WIDGET_DISK" },
        BarWidgetChoice { id: "updates",            label: "Updates",               setting: "DOTFILES_WIDGET_UPDATES" },

        BarWidgetChoice { id: "keyboard",           label: "Keyboard Layout",       setting: "DOTFILES_WIDGET_KEYBOARD" },
        BarWidgetChoice { id: "window_title",       label: "Window Title",          setting: "DOTFILES_WIDGET_WINDOW_TITLE" },
        BarWidgetChoice { id: "notifications",      label: "Notifications",         setting: "DOTFILES_WIDGET_NOTIFICATIONS" },
        BarWidgetChoice { id: "idle_inhibitor",     label: "Idle Inhibitor",        setting: "DOTFILES_WIDGET_IDLE_INHIBITOR" },

        BarWidgetChoice { id: "wallpaper_prev",     label: "Wallpaper Previous",    setting: "DOTFILES_WIDGET_WALLPAPER" },
        BarWidgetChoice { id: "wallpaper_shuffle",  label: "Wallpaper Shuffle",     setting: "DOTFILES_WIDGET_WALLPAPER" },
        BarWidgetChoice { id: "wallpaper_next",     label: "Wallpaper Next",        setting: "DOTFILES_WIDGET_WALLPAPER" },

        BarWidgetChoice { id: "custom_text",        label: "Custom Text",           setting: "DOTFILES_WIDGET_CUSTOM_TEXT" },
        BarWidgetChoice { id: "custom_script",      label: "Custom Script",         setting: "DOTFILES_WIDGET_CUSTOM_SCRIPT" },
        BarWidgetChoice { id: "spacer",             label: "Spacer",                setting: "DOTFILES_WIDGET_SPACER" },
        BarWidgetChoice { id: "power",              label: "Power Menu",            setting: "DOTFILES_WIDGET_POWER" },
    ]
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect()
}

fn normalize_bar_widget_id(id: &str) -> &str {
    match id.trim() {
        "clock" | "date" | "calendar" => "datetime",
        other => other,
    }
}


fn parse_widget_zone(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn widget_label(id: &str) -> String {
    for choice in bar_widget_choices() {
        if choice.id == id {
            return choice.label.to_string();
        }
    }

    id.replace('_', " ")
        .replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn widget_setting(id: &str) -> Option<&'static str> {
    bar_widget_choices()
        .iter()
        .find(|widget| widget.id == id)
        .map(|widget| widget.setting)
}

fn zone_key(zone: &str) -> &'static str {
    match zone {
        "left" => "DOTFILES_BAR_LEFT_WIDGETS",
        "center" => "DOTFILES_BAR_CENTER_WIDGETS",
        _ => "DOTFILES_BAR_RIGHT_WIDGETS",
    }
}

fn write_widget_zone(values: &mut HashMap<String, String>, zone: &str, widgets: &[String]) {
    values.insert(zone_key(zone).to_string(), widgets.join(","));
}

fn remove_widget_from_zones(values: &mut HashMap<String, String>, widget: &str) {
    for zone in ["left", "center", "right"] {
        let key = zone_key(zone);
        let mut list = parse_widget_zone(&value_or(values, key));
        list.retain(|item| item != widget);
        write_widget_zone(values, zone, &list);
    }
}

fn keybind_display(keybind: &Keybind) -> String {
    format!("{}  {}", format_keybind_key(&keybind.key), keybind.name)
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
        corner_radius: CornerRadius::same(255),
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

fn combo_value(
    ui: &mut egui::Ui,
    values: &mut HashMap<String, String>,
    key: &str,
    choices: &[&str],
) {
    let mut current = values
        .get(key)
        .cloned()
        .unwrap_or_else(|| default_value(key));
    ui.horizontal(|ui| {
        ui.label(label_from_key(key));
        egui::ComboBox::from_id_salt(key)
            .selected_text(&current)
            .show_ui(ui, |ui| {
                for choice in choices {
                    ui.selectable_value(&mut current, (*choice).to_string(), *choice);
                }
            });
    });
    values.insert(key.to_string(), current);
}

fn slider_value(
    ui: &mut egui::Ui,
    theme: &AppTheme,
    values: &mut HashMap<String, String>,
    key: &str,
    min: f64,
    max: f64,
) {
    let mut current = values
        .get(key)
        .cloned()
        .unwrap_or_else(|| default_value(key))
        .parse::<f64>()
        .unwrap_or(min)
        .clamp(min, max);

    let mut changed = false;

    Frame {
        fill: theme.panel_soft,
        corner_radius: CornerRadius::same(14),
        inner_margin: Margin::symmetric(14, 10),
        outer_margin: Margin::symmetric(0, 5),
        stroke: Stroke::new(1.0, theme.border),
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(label_from_key(key))
                        .color(theme.text)
                        .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format_float(current))
                            .color(theme.muted)
                            .monospace(),
                    );
                });
            });

            changed |= ui
                .add(
                    egui::Slider::new(&mut current, min..=max)
                        .show_value(false)
                        .text(""),
                )
                .changed();
        });
    });

    if changed {
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
    values
        .get(key)
        .cloned()
        .unwrap_or_else(|| default_value(key))
}

fn is_true(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn friendly_kind(kind: &str) -> &'static str {
    if kind == "app" {
        "App / script"
    } else {
        "Desktop action"
    }
}

fn format_keybind_key(key: &str) -> String {
    let normalized = key
        .replace(',', "+")
        .replace("  ", " ");

    let mut parts = normalized
        .split('+')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let upper = part.to_uppercase();
            match upper.as_str() {
                "SUPER" | "META" | "WIN" | "$MOD" | "MOD" => "SUPER".to_string(),
                "SHIFT" => "SHIFT".to_string(),
                "CTRL" | "CONTROL" => "CTRL".to_string(),
                "ALT" => "ALT".to_string(),
                "RETURN" | "ENTER" => "ENTER".to_string(),
                "SPACE" => "SPACE".to_string(),
                other => other.to_string(),
            }
        })
        .collect::<Vec<_>>();

    parts.retain(|part| part != "SUPER");

    if parts.is_empty() {
        "SUPER".to_string()
    } else {
        format!("SUPER + {}", parts.join(" + "))
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


fn addons_refresh_marker() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".cache"))
        .join("splinterdots")
        .join("addons-refresh")
}


fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
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
