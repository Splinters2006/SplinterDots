"""Check both Zsh entry points with isolated configuration and a fake prompt binary."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


@unittest.skipUnless(shutil.which('zsh'), 'zsh is required')
class ShellStartupTests(unittest.TestCase):
    def start_shell(self, xdg_entry=False, enabled=True):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = root / 'config'
            zsh_dir = config / 'zsh'
            zsh_dir.mkdir(parents=True)
            (config / 'dotfiles').mkdir()
            (config / 'dotfiles/settings.conf').write_text(
                f'DOTFILES_ENABLE_STARSHIP="{int(enabled)}"\nDOTFILES_EDITOR="shell-test-editor"\n'
            )
            shutil.copy(ROOT / 'home/.config/zsh/.zshrc', zsh_dir / '.zshrc')
            shutil.copy(ROOT / 'home/.zshrc', root / '.zshrc')
            shutil.copy(ROOT / 'home/.config/starship.toml', config / 'starship.toml')
            # Source the tested profile explicitly, independent of the host's .profile.
            canonical = zsh_dir / '.zshrc'
            canonical.write_text(canonical.read_text().replace(
                '"$HOME/.profile"', '"' + str(ROOT / 'home/.profile') + '"'))
            bin_dir = root / 'bin'
            bin_dir.mkdir()
            starship = bin_dir / 'starship'
            starship.write_text('#!/bin/sh\nprintf "init\\n" >> "$TEST_CALLS"\nprintf "export TEST_PROMPT_READY=yes\\n"\n')
            starship.chmod(0o755)
            env = dict(os.environ, ZDOTDIR=str(zsh_dir if xdg_entry else root),
                       XDG_CONFIG_HOME=str(config), XDG_STATE_HOME=str(root / 'state'),
                       XDG_CACHE_HOME=str(root / 'cache'), TEST_CALLS=str(root / 'calls'),
                       PATH=str(bin_dir) + ':' + os.environ['PATH'], TERM='xterm-256color')
            env.pop('STARSHIP_CONFIG', None)
            env.pop('TEST_PROMPT_READY', None)
            result = subprocess.run(['zsh', '-ic',
                'print -r -- "editor=$EDITOR" "config=$STARSHIP_CONFIG" "prompt=$TEST_PROMPT_READY"; alias ..; [[ -o auto_cd ]]'],
                env=env, capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('editor=shell-test-editor', result.stdout)
            self.assertIn("..='cd ..'", result.stdout)
            if enabled:
                self.assertIn('prompt=yes', result.stdout)
                self.assertIn('config=' + str(config / 'starship.toml'), result.stdout)
                self.assertEqual((root / 'calls').read_text().splitlines(), ['init'])
            else:
                self.assertFalse((root / 'calls').exists())

    def test_home_entry_loads_full_zsh_and_starship_once(self):
        self.start_shell()

    def test_xdg_entry_loads_the_same_configuration(self):
        self.start_shell(xdg_entry=True)

    def test_starship_can_be_disabled(self):
        self.start_shell(enabled=False)


if __name__ == '__main__':
    unittest.main()
