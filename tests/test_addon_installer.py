"""Verify font/add-on installation outcomes without installing system packages."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class AddonInstallerTests(unittest.TestCase):
    def check_install(self, exit_code):
        with tempfile.TemporaryDirectory(prefix='splinter install ') as directory:
            root = Path(directory)
            scripts = root / 'scripts'
            scripts.mkdir()
            installer = scripts / 'splinter-install-addon'
            shutil.copy(ROOT / 'scripts/splinter-install-addon', installer)
            helper = scripts / 'splinterdots-style'
            helper.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$TEST_ARGS"\nexit "$TEST_EXIT"\n')
            helper.chmod(0o755)
            env = dict(os.environ, XDG_CACHE_HOME=str(root / 'cache'),
                       TEST_ARGS=str(root / 'args'), TEST_EXIT=str(exit_code))
            result = subprocess.run([str(installer), 'ttf-test-font'], env=env,
                                    capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, exit_code, result.stderr)
            self.assertEqual((root / 'args').read_text().splitlines(), ['install', 'ttf-test-font'])
            marker = root / 'cache/splinterdots/addons-refresh'
            self.assertEqual(marker.exists(), exit_code == 0)
            if exit_code:
                self.assertIn('Installation failed', result.stderr)
            else:
                self.assertIn('completed successfully', result.stdout)

    def test_success_refreshes_font_picker(self):
        self.check_install(0)

    def test_failure_is_reported_without_success_marker(self):
        self.check_install(7)


if __name__ == '__main__':
    unittest.main()
