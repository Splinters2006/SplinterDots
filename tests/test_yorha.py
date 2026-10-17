import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class YorhaTerminalTests(unittest.TestCase):
    def run_yorha(self, home, *args):
        env = dict(os.environ, HOME=str(home), XDG_CONFIG_HOME=str(home / '.config'),
                   XDG_DATA_HOME=str(home / '.local/share'), XDG_CACHE_HOME=str(home / '.cache'))
        return subprocess.run([str(ROOT / 'scripts/splinter-yorha'), *args], env=env,
                              capture_output=True, text=True, timeout=30)

    def test_install_writes_launcher_and_themed_profile_then_uninstall_removes_them(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            wallpaper = home / 'wall.png'
            wallpaper.write_bytes(b'png')
            (home / '.config/dotfiles').mkdir(parents=True)
            (home / '.config/dotfiles/local.conf').write_text(f'DOTFILES_WALLPAPER="{wallpaper}"\n')

            self.assertEqual(self.run_yorha(home, 'install').returncode, 0)
            desktop = home / '.local/share/applications/splinterdots-yorha-terminal.desktop'
            self.assertIn('Name=YoRHa Terminal', desktop.read_text())
            config = (home / '.config/splinterdots/yorha/kitty.conf').read_text()
            self.assertIn(f'background_image {wallpaper}', config)
            self.assertIn('background #cdc8b0', config)
            self.assertIn('env SPLINTER_YORHA=1', config)
            self.assertIn(f'include {home}/.config/kitty/kitty.conf', config)

            self.assertEqual(self.run_yorha(home, 'uninstall').returncode, 0)
            self.assertFalse(desktop.exists())
            self.assertFalse((home / '.config/splinterdots/yorha').exists())

    @unittest.skipUnless(shutil.which('magick'), 'ImageMagick not installed')
    def test_dedicated_dark_background_gets_light_text(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            dark = home / 'YoRHa_bg.jpg'
            subprocess.run(['magick', '-size', '32x32', 'xc:#151311', str(dark)], check=True)
            (home / '.config/dotfiles').mkdir(parents=True)
            (home / '.config/dotfiles/local.conf').write_text(
                f'DOTFILES_WALLPAPER="{home}/missing.png"\nDOTFILES_YORHA_BACKGROUND="{dark}"\n')
            config = self.run_yorha(home, 'config').stdout
            self.assertIn('background_image ' + str(home / '.cache/splinterdots/yorha-background-'), config)
            self.assertIn('foreground #cdc8b0', config)
            self.assertIn('background_tint 0.15', config)

    def test_missing_wallpaper_still_gives_a_readable_profile(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            result = self.run_yorha(home, 'config')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertNotIn('background_image', result.stdout)
            self.assertIn('foreground #4e4b42', result.stdout)


class LocalAddonInstallTests(unittest.TestCase):
    def test_splinterdots_addons_skip_the_package_manager(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'scripts').mkdir()
            shutil.copy(ROOT / 'scripts/splinter-install-addon', root / 'scripts/splinter-install-addon')
            for name, body in [('splinterdots-style', 'echo style "$@" >> "$LOG"'), ('splinter-yorha', 'echo yorha "$@" >> "$LOG"')]:
                script = root / 'scripts' / name
                script.write_text(f'#!/bin/sh\n{body}\n')
                script.chmod(0o755)
            env = dict(os.environ, XDG_CACHE_HOME=str(root / 'cache'), LOG=str(root / 'log'))
            result = subprocess.run([str(root / 'scripts/splinter-install-addon'), 'splinterdots-yorha-terminal'],
                                    env=env, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((root / 'log').read_text().splitlines(), ['yorha install'])


if __name__ == '__main__':
    unittest.main()
