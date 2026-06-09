#!/usr/bin/env bash
set -euo pipefail

if [ ! -f "install.sh" ] || [ ! -d "scripts" ]; then
  echo "Run this from the root of your SplinterDots repo."
  exit 1
fi

backup_dir=".dotfiles-change-backup/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$backup_dir/scripts" "$backup_dir/config"

cp -a scripts/dotfiles-center "$backup_dir/scripts/dotfiles-center"
if [ -f scripts/dotfiles-wallpaper ]; then
  cp -a scripts/dotfiles-wallpaper "$backup_dir/scripts/dotfiles-wallpaper"
fi
if [ -f config/dotfiles/hyprland-options.json ]; then
  cp -a config/dotfiles/hyprland-options.json "$backup_dir/config/hyprland-options.json"
fi

python - <<'PY'
from pathlib import Path
import re

root = Path.cwd()
center_path = root / "scripts" / "dotfiles-center"
wallpaper_path = root / "scripts" / "dotfiles-wallpaper"

text = center_path.read_text(encoding="utf-8")

# Constants
if 'MAKO_CONFIG = CONFIG_HOME / "mako" / "config"' not in text:
    text = text.replace(
        'QUICKSHELL_FILE = QUICKSHELL_DIR / "shell.qml"\nHYPR_SCHEMA_FILE = ROOT / "config" / "dotfiles" / "hyprland-options.json"\n',
        'QUICKSHELL_FILE = QUICKSHELL_DIR / "shell.qml"\n'
        'MAKO_CONFIG = CONFIG_HOME / "mako" / "config"\n'
        'HYPR_SCHEMA_FILE = ROOT / "config" / "dotfiles" / "hyprland-options.json"\n'
    )

if "EXCLUDED_HYPR_OPTIONS" not in text:
    text = text.replace(
        'HYPR_SCHEMA_FILE = ROOT / "config" / "dotfiles" / "hyprland-options.json"\n',
        'HYPR_SCHEMA_FILE = ROOT / "config" / "dotfiles" / "hyprland-options.json"\n\n'
        '# Hidden from the beginner-friendly Hyprland page.\n'
        'EXCLUDED_HYPR_OPTIONS = {\n'
        '    "misc:disable_hyprland_qtutils_check",\n'
        '    "debug:watchdog_timeout",\n'
        '}\n'
    )

# Only show beginner-friendly Hyprland settings. Open-ended text/path/pid-style settings stay hidden.
old = """    options = data.get("options", [])
    return [option for option in options if isinstance(option, dict) and option.get("path")]
"""
new = """    options = data.get("options", [])
    friendly_types = {"bool", "choice", "int", "float"}
    cleaned: list[dict] = []
    for option in options:
        if not isinstance(option, dict):
            continue
        path = str(option.get("path", ""))
        kind = str(option.get("type", "text"))
        if not path or path in EXCLUDED_HYPR_OPTIONS:
            continue
        if kind not in friendly_types:
            continue
        cleaned.append(option)
    return cleaned
"""
if old in text:
    text = text.replace(old, new)

helpers = """
def safe_info(master: tk.Misc, title: str, message: str) -> None:
    try:
        if master.winfo_exists():
            messagebox.showinfo(title, message, parent=master)
    except tk.TclError:
        print(f"{title}: {message}")


def safe_warning(master: tk.Misc, title: str, message: str) -> None:
    try:
        if master.winfo_exists():
            messagebox.showwarning(title, message, parent=master)
    except tk.TclError:
        print(f"{title}: {message}", file=sys.stderr)


def safe_ask_yes_no(master: tk.Misc, title: str, message: str) -> bool:
    try:
        if master.winfo_exists():
            return bool(messagebox.askyesno(title, message, parent=master))
    except tk.TclError:
        pass
    return False


def script_command(name: str) -> str:
    found = shutil.which(name)
    if found:
        return found
    return str(ROOT / "scripts" / name)


def theme_palette(values: dict[str, str]) -> dict[str, str]:
    accent = values.get("DOTFILES_ACCENT", DEFAULTS["DOTFILES_ACCENT"])
    if values.get("DOTFILES_THEME", "dark") == "light":
        return {
            "accent": accent,
            "background": "#f8fafc",
            "surface": "#e2e8f0",
            "surface_alt": "#cbd5e1",
            "text": "#0f172a",
            "muted": "#475569",
            "inactive_border": "#94a3b8",
            "bar_rgb": "f8fafc",
            "active_text": "#ffffff",
        }
    return {
        "accent": accent,
        "background": "#1e1e2e",
        "surface": "#313244",
        "surface_alt": "#45475a",
        "text": "#cdd6f4",
        "muted": "#bac2de",
        "inactive_border": "#45475a",
        "bar_rgb": "1e1e2e",
        "active_text": "#11111b",
    }


"""
if "def safe_info(master: tk.Misc" not in text:
    marker = "# ──────────────────────────────────────────────────────────────────────────────\n# Sleek ScrollFrame"
    text = text.replace(marker, helpers + "\n" + marker)

# Theme-aware colors and Mako.
old_colors = """def write_colors(accent: str) -> None:
    HYPR_DIR.mkdir(parents=True, exist_ok=True)
    HYPR_COLORS.write_text(
        f"$accent = {hex_to_hypr_rgba(accent)}\\n$inactive = rgba(45475aff)\\n",
        encoding="utf-8",
    )


"""
new_colors = """def write_mako_config(values: dict[str, str]) -> None:
    palette = theme_palette(values)
    MAKO_CONFIG.parent.mkdir(parents=True, exist_ok=True)
    MAKO_CONFIG.write_text(
        "\\n".join([
            f"background-color={palette['background']}",
            f"text-color={palette['text']}",
            f"border-color={palette['accent']}",
            "border-size=2",
            "border-radius=8",
            "padding=12",
            "default-timeout=5000",
            "",
        ]),
        encoding="utf-8",
    )


def write_colors(values: dict[str, str]) -> None:
    palette = theme_palette(values)
    HYPR_DIR.mkdir(parents=True, exist_ok=True)
    HYPR_COLORS.write_text(
        "\\n".join([
            f"$accent = {hex_to_hypr_rgba(palette['accent'])}",
            f"$inactive = {hex_to_hypr_rgba(palette['inactive_border'])}",
            f"$background = {hex_to_hypr_rgba(palette['background'])}",
            f"$text = {hex_to_hypr_rgba(palette['text'])}",
            "",
        ]),
        encoding="utf-8",
    )
    write_mako_config(values)


"""
if old_colors in text:
    text = text.replace(old_colors, new_colors)

text = text.replace('write_colors(values["DOTFILES_ACCENT"])', 'write_colors(values)')

# Theme-aware QML bar + active workspace tracking.
if 'theme = values.get("DOTFILES_THEME", "dark")' not in text:
    text = text.replace(
        '    accent = values.get("DOTFILES_ACCENT", "#89b4fa")\n',
        '    accent = values.get("DOTFILES_ACCENT", "#89b4fa")\n'
        '    theme = values.get("DOTFILES_THEME", "dark")\n'
        '    palette = theme_palette(values)\n'
    )

text = text.replace(
    '    bg_color = "#%02x1e1e2e" % bg_alpha\n',
    '    bg_color = f"#{bg_alpha:02x}{palette[\'bar_rgb\']}"\n'
)

text = text.replace(
    '                width: {height - 10}\n'
    '                height: {height - 10}\n',
    '                width: (index + 1) === root.activeWorkspace ? {height - 2} : {height - 14}\n'
    '                height: (index + 1) === root.activeWorkspace ? {height - 2} : {height - 14}\n'
    '                anchors.verticalCenter: parent.verticalCenter\n'
)

text = text.replace("index === 0", "(index + 1) === root.activeWorkspace")

text = text.replace(
    '    PanelWindow {{\n'
    '      required property var modelData\n',
    '    PanelWindow {{\n'
    '      id: root\n'
    '      property int activeWorkspace: 1\n'
    '      required property var modelData\n'
)

active_workspace_block = """
      Process {
        id: activeWorkspaceProc
        command: ["sh", "-c", "hyprctl activeworkspace -j 2>/dev/null | sed -n 's/.*\\\"id\\\"[[:space:]]*:[[:space:]]*\\\\([0-9][0-9]*\\\\).*/\\\\1/p'"]
        running: true
        stdout: StdioCollector {
          onStreamFinished: {
            var value = parseInt(this.text.trim())
            if (!isNaN(value)) root.activeWorkspace = value
          }
        }
      }

      Timer {
        interval: 700
        running: true
        repeat: true
        onTriggered: activeWorkspaceProc.running = true
      }

"""
if "id: activeWorkspaceProc" not in text:
    text = text.replace(
        '      color: "transparent"\n\n'
        '      Rectangle {{\n',
        '      color: "transparent"\n' + active_workspace_block + '\n'
        '      Rectangle {{\n'
    )

if "QML theme replacements" not in text:
    text = text.replace(
        '    QUICKSHELL_FILE.write_text(qml, encoding="utf-8")\n',
        """    # QML theme replacements
    if theme == "light":
        qml = (qml
            .replace("#cdd6f4", palette["text"])
            .replace("#bac2de", palette["muted"])
            .replace("#6c7086", palette["muted"])
            .replace("#313244", palette["surface"])
            .replace("#45475a", palette["surface_alt"])
            .replace("#11111b", palette["active_text"])
        )
    QUICKSHELL_FILE.write_text(qml, encoding="utf-8")\n"""
    )

# Overview follows shortcut changes.
if "self.overview_grid" not in text:
    text = text.replace(
        '        self.shortcut_tree: ttk.Treeview | None = None\n',
        '        self.shortcut_tree: ttk.Treeview | None = None\n'
        '        self.overview_grid: tk.Frame | None = None\n'
    )

overview_methods = """
    def format_shortcut_key(self, key: str) -> str:
        parts = [part.strip() for part in key.split(",") if part.strip()]
        pretty = []
        for part in parts:
            upper = part.upper()
            if upper == "SHIFT":
                pretty.append("Shift")
            elif upper == "CTRL":
                pretty.append("Ctrl")
            elif upper == "ALT":
                pretty.append("Alt")
            else:
                pretty.append(part)
        return "Super + " + " + ".join(pretty) if pretty else "Super"

    def shortcut_by_name(self, name: str) -> dict | None:
        wanted = name.lower()
        for shortcut in self.shortcuts:
            if shortcut.get("name", "").lower() == wanted:
                return shortcut
        return None

    def overview_shortcuts_hint(self) -> list[tuple[str, str]]:
        wanted = [
            ("App launcher", "Open apps"),
            ("Terminal", "Terminal"),
            ("Dotfiles Center", "Dotfiles Center"),
            ("Reload desktop", "Reload desktop"),
        ]
        rows: list[tuple[str, str]] = []
        for shortcut_name, label in wanted:
            shortcut = self.shortcut_by_name(shortcut_name)
            if shortcut:
                rows.append((self.format_shortcut_key(shortcut.get("key", "")), label))
        rows.extend([
            ("Super + Left Mouse", "Drag windows"),
            ("Super + Right Mouse", "Resize windows"),
        ])
        return rows[:6]

    def refresh_overview_shortcuts(self) -> None:
        if self.overview_grid is None:
            return
        for child in self.overview_grid.winfo_children():
            child.destroy()
        for col, (key, label) in enumerate(self.overview_shortcuts_hint()):
            cell = tk.Frame(self.overview_grid, bg=self.card_alt, padx=10, pady=8)
            cell.grid(row=0, column=col, sticky="ew", padx=(0 if col == 0 else 6, 0))
            tk.Label(cell, text=key,   bg=self.card_alt, fg=self.text,  font=("Sans", 10, "bold")).pack(anchor="w")
            tk.Label(cell, text=label, bg=self.card_alt, fg=self.muted, font=("Sans", 9)).pack(anchor="w", pady=(2, 0))
            self.overview_grid.columnconfigure(col, weight=1)

"""
if "def format_shortcut_key(self, key: str)" not in text:
    text = text.replace("    # ── Overview tab", overview_methods + "\n    # ── Overview tab")

old_overview_block = """        grid = tk.Frame(hero, bg=self.card)
        grid.pack(fill="x")
        shortcuts_hint = [
            ("Super + D",           "App launcher"),
            ("Super + Return",      "Terminal"),
            ("Super + Left Mouse",  "Drag window"),
            ("Super + Right Mouse", "Resize window"),
            ("Super + W",           "Dotfiles Center"),
            ("Super + Shift + R",   "Reload Hyprland"),
        ]
        for col, (key, label) in enumerate(shortcuts_hint):
            cell = tk.Frame(grid, bg=self.card_alt, padx=10, pady=8)
            cell.grid(row=0, column=col, sticky="ew", padx=(0 if col == 0 else 6, 0))
            tk.Label(cell, text=key,   bg=self.card_alt, fg=self.text,  font=("Sans", 10, "bold")).pack(anchor="w")
            tk.Label(cell, text=label, bg=self.card_alt, fg=self.muted, font=("Sans", 9)).pack(anchor="w", pady=(2,0))
            grid.columnconfigure(col, weight=1)
"""
new_overview_block = """        self.overview_grid = tk.Frame(hero, bg=self.card)
        self.overview_grid.pack(fill="x")
        self.refresh_overview_shortcuts()
"""
if old_overview_block in text:
    text = text.replace(old_overview_block, new_overview_block)

text = text.replace(
    """            self.shortcuts.append(sc)
            self.refresh_shortcut_tree()
""",
    """            self.shortcuts.append(sc)
            self.refresh_shortcut_tree()
            self.refresh_overview_shortcuts()
"""
)
text = text.replace(
    """            self.shortcuts[idx] = updated
            self.refresh_shortcut_tree()
""",
    """            self.shortcuts[idx] = updated
            self.refresh_shortcut_tree()
            self.refresh_overview_shortcuts()
"""
)
text = text.replace(
    """            del self.shortcuts[idx]
            self.refresh_shortcut_tree()
""",
    """            del self.shortcuts[idx]
            self.refresh_shortcut_tree()
            self.refresh_overview_shortcuts()
"""
)
text = text.replace(
    """            self.shortcuts = default_shortcuts()
            self.refresh_shortcut_tree()
""",
    """            self.shortcuts = default_shortcuts()
            self.refresh_shortcut_tree()
            self.refresh_overview_shortcuts()
"""
)

# Taller selected tab.
tab_map_block = """        mp("TNotebook.Tab",
           background=[("selected", self.card)],
           foreground=[("selected", self.text), ("active", self.text)])
"""
if tab_map_block in text and 'padding=[("selected", (16, 13))' not in text:
    text = text.replace(
        tab_map_block,
        tab_map_block + '        mp("TNotebook.Tab",\n           padding=[("selected", (16, 13)), ("!selected", (16, 9))])\n'
    )

# Better labels.
text = text.replace(
    '"Toggles for on/off · dropdowns for multiple choices · text boxes for numbers, colors, and paths."',
    '"Switches for on/off, dropdowns for choices, and simple number boxes for safe settings."'
)
text = text.replace('self._card_title(cmds, "Terminal commands")', 'self._card_title(cmds, "Helpful terminal actions")')

# Add Apply theme button.
if 'text="Apply theme"' not in text:
    text = text.replace(
        """        ttk.Button(btns, text="Apply wallpaper", style="Accent.TButton",
                   command=self.apply_wallpaper).pack(side="left")
""",
        """        ttk.Button(btns, text="Apply wallpaper", style="Accent.TButton",
                   command=self.apply_wallpaper).pack(side="left")
        ttk.Button(btns, text="Apply theme", style="Ghost.TButton",
                   command=self.save_all).pack(side="left", padx=8)
"""
    )

# Safe messageboxes and wallpaper path.
text = text.replace(
    'messagebox.showinfo("Saved", "Settings saved. Press Super + Shift + R or log back in to apply.")',
    'safe_info(self, "Saved", "Settings saved. Press Super + Shift + R or log back in to apply.")'
)
text = text.replace(
    'messagebox.showinfo("QML bar", "Bar settings saved and restarted.")',
    'safe_info(self, "QML bar", "Bar settings saved and restarted.")'
)
text = text.replace(
    'messagebox.showinfo("Shortcuts", "Select a shortcut first.")',
    'safe_info(self, "Shortcuts", "Select a shortcut first.")'
)
text = text.replace(
    'messagebox.askyesno("Remove", f"Remove \'{self.shortcuts[idx].get(\'name\', \'this shortcut\')}\'?")',
    'safe_ask_yes_no(self, "Remove", f"Remove \'{self.shortcuts[idx].get(\'name\', \'this shortcut\')}\'?")'
)
text = text.replace(
    'messagebox.askyesno("Restore defaults", "Replace all shortcuts with defaults?")',
    'safe_ask_yes_no(self, "Restore defaults", "Replace all shortcuts with defaults?")'
)
text = text.replace(
    'messagebox.showwarning("Wallpaper", f"Could not apply wallpaper: {exc}")',
    'safe_warning(self, "Wallpaper", f"Could not apply wallpaper: {exc}")'
)

old_apply = """    def apply_wallpaper(self, show_errors: bool = True) -> None:
        wallpaper = self.vars["DOTFILES_WALLPAPER"].get()
        try:
            subprocess.run(["dotfiles-wallpaper", "set", wallpaper], check=True)
        except (OSError, subprocess.CalledProcessError) as exc:
            if show_errors:
                messagebox.showwarning("Wallpaper", f"Could not apply wallpaper: {exc}")
"""
new_apply = """    def apply_wallpaper(self, show_errors: bool = True) -> None:
        wallpaper = self.vars["DOTFILES_WALLPAPER"].get().strip()
        try:
            write_local_conf(self.current_values())
            subprocess.run([script_command("dotfiles-wallpaper"), "set", wallpaper], check=True)
        except (OSError, subprocess.CalledProcessError) as exc:
            if show_errors:
                safe_warning(self, "Wallpaper", f"Could not apply wallpaper: {exc}")
"""
if old_apply in text:
    text = text.replace(old_apply, new_apply)
else:
    text = text.replace(
        'subprocess.run(["dotfiles-wallpaper", "set", wallpaper], check=True)',
        'subprocess.run([script_command("dotfiles-wallpaper"), "set", wallpaper], check=True)'
    )

center_path.write_text(text, encoding="utf-8")

# Fix dotfiles-wallpaper stale variable bug.
if wallpaper_path.exists():
    w = wallpaper_path.read_text(encoding="utf-8")
    if 'DOTFILES_WALLPAPER="$image"\n\n  apply_wallpaper' not in w:
        w = w.replace('  apply_wallpaper\n}', '  DOTFILES_WALLPAPER="$image"\n\n  apply_wallpaper\n}')
    wallpaper_path.write_text(w, encoding="utf-8")

print("Patched scripts/dotfiles-center")
print("Patched scripts/dotfiles-wallpaper")
PY

chmod +x scripts/dotfiles-center
chmod +x scripts/dotfiles-wallpaper 2>/dev/null || true

python -m py_compile scripts/dotfiles-center

echo
echo "Done."
echo "Backups are in: $backup_dir"
echo
echo "Recommended next commands:"
echo "  git diff"
echo "  ./install.sh --dry-run"
echo "  ./install.sh"
echo "  dotctl center --force"
echo
echo "In Dotfiles Center, press Save changes once so it regenerates the Hyprland, Mako, and QML files."
