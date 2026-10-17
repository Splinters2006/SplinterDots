"""Check palette synchronization without touching Spotify or running installers."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'scripts/splinter-sync-spicetify'
COLORS = ['#102030', '#203040', '#eeeeee', '#bbbbbb', '#abcdef', '#405060', '#000000']


class SpicetifySyncTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.config = self.root / 'spicetify/config-xpui.ini'
        self.config.parent.mkdir()
        self.config.write_text('[Setting]\ncurrent_theme = Old\ncolor_scheme = Old\nextensions = keep.js\ncustom_apps = marketplace\n')
        binaries = self.root / 'bin'
        binaries.mkdir()
        spicetify = binaries / 'spicetify'
        spicetify.write_text('''#!/usr/bin/env python3
import configparser, json, os, pathlib, sys
root = pathlib.Path(os.environ['TEST_ROOT'])
args = sys.argv[1:]
with (root / 'calls').open('a') as log: log.write(json.dumps(args) + '\\n')
if args == ['-c']:
    print(root / 'spicetify/config-xpui.ini')
elif args[0] == 'config':
    config = configparser.ConfigParser()
    file = root / 'spicetify/config-xpui.ini'
    config.read(file)
    for key, value in zip(args[1::2], args[2::2]): config['Setting'][key] = value
    with file.open('w') as out: config.write(out)
elif args[0] == 'apply' and os.environ.get('FAIL_APPLY') == '1':
    print('Backup is outdated', file=sys.stderr)
    sys.exit(1)
''')
        spicetify.chmod(0o755)
        pgrep = binaries / 'pgrep'
        pgrep.write_text('#!/bin/sh\n[ "$SPOTIFY_RUNNING" = 1 ]\n')
        pgrep.chmod(0o755)
        self.env = dict(os.environ, TEST_ROOT=str(self.root), SPOTIFY_RUNNING='1',
                        PATH=str(binaries) + ':' + os.environ['PATH'])

    def sync(self, colors=COLORS):
        return subprocess.run([str(SCRIPT), *colors], env=self.env, capture_output=True, text=True)

    def calls(self):
        return [json.loads(line) for line in (self.root / 'calls').read_text().splitlines()]

    def test_saved_colors_apply_immediately_without_changing_extensions(self):
        result = self.sync()
        self.assertEqual(result.returncode, 0, result.stderr)
        theme = self.config.parent / 'Themes/SplinterDots'
        self.assertIn('main = 102030', (theme / 'color.ini').read_text())
        self.assertIn('button = abcdef', (theme / 'color.ini').read_text())
        self.assertIn('var(--spice-main)', (theme / 'user.css').read_text())
        self.assertEqual(self.calls()[-1], ['apply'])
        self.assertIn('extensions = keep.js', self.config.read_text())
        self.assertIn('custom_apps = marketplace', self.config.read_text())
        self.assertEqual(self.sync().returncode, 0)
        self.assertEqual(sum(call[0] == 'apply' for call in self.calls()), 1)
        colors = COLORS.copy()
        colors[4] = '#ff9988'
        self.assertEqual(self.sync(colors).returncode, 0)
        self.assertEqual(sum(call[0] == 'apply' for call in self.calls()), 2)

    def test_closed_spotify_is_not_launched(self):
        self.env['SPOTIFY_RUNNING'] = '0'
        self.assertEqual(self.sync().returncode, 0)
        self.assertEqual(self.calls()[-1], ['apply', '--no-restart'])

    def test_failed_apply_is_reported_and_retried(self):
        self.env['FAIL_APPLY'] = '1'
        result = self.sync()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Backup is outdated', result.stderr)
        self.env['FAIL_APPLY'] = '0'
        self.assertEqual(self.sync().returncode, 0)
        self.assertEqual(sum(call[0] == 'apply' for call in self.calls()), 2)

    def test_invalid_color_does_not_touch_spicetify(self):
        result = self.sync(['bad color', *COLORS[1:]])
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / 'calls').exists())


if __name__ == '__main__':
    unittest.main()
