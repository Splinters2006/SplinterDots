"""Validate application palette files without contacting running applications."""
import configparser
import os
import subprocess
from pathlib import Path
import runpy
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SYNC = runpy.run_path(str(ROOT / 'scripts/splinter-sync-theme'))
COLORS = ['#102030', '#203040', '#eeeeee', '#bbbbbb', '#abcdef', '#405060', '#000000']


class ThemeSyncTests(unittest.TestCase):
    def test_kitty_updates_generated_file_without_overwriting_builtin(self):
        with tempfile.TemporaryDirectory() as directory, patch('subprocess.run'):
            config = Path(directory)
            builtin = config / 'kitty/themes/midnight.conf'
            builtin.parent.mkdir(parents=True)
            builtin.write_text('original theme\n')
            current = config / 'kitty/current-theme.conf'
            current.symlink_to(builtin)
            SYNC['apply_kitty'](config, COLORS)
            self.assertEqual(builtin.read_text(), 'original theme\n')
            self.assertIn('background #102030', current.read_text())
            self.assertIn('selection_background #abcdef', current.read_text())
            colors = COLORS.copy()
            colors[0] = '#eeeeee'
            SYNC['apply_kitty'](config, colors)
            self.assertIn('background #eeeeee', current.read_text())

    def test_gtk_preserves_custom_css_and_settings_across_saves(self):
        with tempfile.TemporaryDirectory() as directory, patch('subprocess.run'):
            config = Path(directory)
            for version in (3, 4):
                folder = config / f'gtk-{version}.0'
                folder.mkdir()
                (folder / 'gtk.css').write_text('button { border-radius: 8px; }\n')
                (folder / 'settings.ini').write_text('[Settings]\ngtk-font-name=My Font 12\ngtk-icon-theme-name=My Icons\n')
            SYNC['apply_gtk'](config, COLORS)
            light = ['#fafafa', '#ffffff', '#000000', '#555555', '#2563eb', '#888888', '#ffffff']
            SYNC['apply_gtk'](config, light)
            for version in (3, 4):
                folder = config / f'gtk-{version}.0'
                css = (folder / 'gtk.css').read_text()
                self.assertEqual(css.count('splinterdots-colors.css'), 1)
                self.assertIn('border-radius: 8px', css)
                self.assertEqual((folder / 'gtk.css.before-splinterdots').read_text(), 'button { border-radius: 8px; }\n')
                generated = (folder / 'splinterdots-colors.css').read_text()
                self.assertIn('@define-color window_bg_color #fafafa;', generated)
                self.assertEqual('--window-bg-color: #fafafa;' in generated, version == 4)
                settings = configparser.ConfigParser()
                settings.read(folder / 'settings.ini')
                self.assertEqual(settings['Settings']['gtk-font-name'], 'My Font 12')
                self.assertEqual(settings['Settings']['gtk-icon-theme-name'], 'My Icons')
                self.assertEqual(settings['Settings']['gtk-application-prefer-dark-theme'], '0')

    def test_applying_fonts_keeps_saved_gtk_palette_settings(self):
        # Use a separate directory and invoke only the style writer (no package commands).
        with tempfile.TemporaryDirectory() as directory:
            config = Path(directory)
            (config / 'dotfiles').mkdir()
            (config / 'dotfiles/local.conf').write_text('DOTFILES_UI_FONT="Test Font"\n')
            for version in (3, 4):
                folder = config / f'gtk-{version}.0'
                folder.mkdir()
                (folder / 'settings.ini').write_text('[Settings]\ngtk-theme-name=Adwaita-dark\ngtk-application-prefer-dark-theme=1\n')
            result = subprocess.run([str(ROOT / 'scripts/splinterdots-style'), 'apply'],
                                    env=dict(os.environ, XDG_CONFIG_HOME=str(config)),
                                    capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            for version in (3, 4):
                settings = configparser.ConfigParser()
                settings.read(config / f'gtk-{version}.0/settings.ini')
                self.assertEqual(settings['Settings']['gtk-theme-name'], 'Adwaita-dark')
                self.assertEqual(settings['Settings']['gtk-font-name'], 'Test Font 11')

    def test_browser_profiles_preserve_customizations_and_update_colors(self):
        with tempfile.TemporaryDirectory() as directory, patch('pathlib.Path.home', return_value=Path(directory)):
            config = Path(directory) / '.config'
            root = config / 'zen'
            profile = root / 'example.default'
            (profile / 'chrome').mkdir(parents=True)
            (root / 'profiles.ini').write_text('[Profile0]\nPath=example.default\nIsRelative=1\n')
            custom = profile / 'chrome/userChrome.css'
            custom.write_text('/* custom styles */\n')
            prefs = profile / 'user.js'
            prefs.write_text('user_pref("other.setting", false);\nuser_pref("toolkit.legacyUserProfileCustomizations.stylesheets", false);\n')
            SYNC['apply_browser'](config, COLORS)
            changed = COLORS.copy()
            changed[0] = '#123456'
            SYNC['apply_browser'](config, changed)
            self.assertIn('/* custom styles */', custom.read_text())
            self.assertEqual(custom.read_text().count('splinterdots-colors.css'), 1)
            self.assertIn('--zen-main-browser-background: #123456', (profile / 'chrome/splinterdots-colors.css').read_text())
            self.assertIn('user_pref("other.setting", false);', prefs.read_text())
            self.assertEqual(prefs.read_text().count('toolkit.legacyUserProfileCustomizations.stylesheets'), 1)
            self.assertIn('stylesheets", true);', prefs.read_text())
            self.assertIn('stylesheets", false);', (profile / 'user.js.before-splinterdots').read_text())

    def test_gtk_file_views_have_explicit_palette_colors(self):
        css = SYNC['gtk_css'](COLORS, 3)
        self.assertIn('.view, treeview, iconview', css)
        self.assertIn('background-color: @theme_base_color; color: @theme_text_color;', css)
        self.assertIn('.view:selected', css)

    def test_vesktop_sync_preserves_css_plugins_and_other_themes(self):
        import json
        with tempfile.TemporaryDirectory() as directory:
            config = Path(directory)
            folder = config / 'vesktop/settings'
            folder.mkdir(parents=True)
            quick = folder / 'quickCss.css'
            quick.write_text('/* personal CSS */\n.my-rule { color: pink; }\n')
            settings = folder / 'settings.json'
            original = {'useQuickCss': False, 'enabledThemes': ['Other.css'], 'plugins': {'Example': {'enabled': True}}}
            settings.write_text(json.dumps(original))
            SYNC['apply_vesktop'](config, COLORS, 'Noto Sans')
            changed = COLORS.copy()
            changed[0] = '#123456'
            SYNC['apply_vesktop'](config, changed, 'DejaVu Sans')
            data = json.loads(settings.read_text())
            self.assertTrue(data['useQuickCss'])
            self.assertEqual(data['plugins'], original['plugins'])
            self.assertEqual(data['enabledThemes'], ['Other.css'])
            css = quick.read_text()
            self.assertIn('.my-rule { color: pink; }', css)
            self.assertEqual(css.count('BEGIN SPLINTERDOTS THEME'), 1)
            self.assertIn('--background-primary: #123456 !important', css)
            self.assertIn('--font-primary: "DejaVu Sans", sans-serif !important', css)
            self.assertNotIn('Noto Sans', css)
            self.assertEqual((folder / 'quickCss.css.before-splinterdots').read_text(), '/* personal CSS */\n.my-rule { color: pink; }\n')
            self.assertEqual(json.loads((folder / 'settings.json.before-splinterdots').read_text()), original)
            self.assertIn('DejaVu Sans', (config / 'vesktop/themes/SplinterDots.theme.css').read_text())

    def test_vesktop_font_name_is_escaped_as_css_string(self):
        font = 'Font"; } body { color: red; } /*'
        quoted = SYNC['css_font_family'](font)
        self.assertEqual(quoted.count('"'), 2)
        self.assertIn(r'\22 ', quoted)
        self.assertNotIn('\n', SYNC['css_font_family']('Font\nName'))

    def test_panel_text_and_accent_text_are_readable(self):
        self.assertEqual(SYNC['readable']('#ffffff'), '#000000')
        self.assertEqual(SYNC['readable']('#000000'), '#ffffff')
        css = SYNC['gtk_css'](['#000000', '#ffffff', '#ffffff', '#aaaaaa', '#ffffff', '#777777', '#000000'], 4)
        self.assertIn('@define-color card_fg_color #000000;', css)
        self.assertIn('--accent-fg-color: #000000;', css)


if __name__ == '__main__':
    unittest.main()
