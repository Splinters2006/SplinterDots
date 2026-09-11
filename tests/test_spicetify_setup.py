"""Exercise fresh installs without downloading code or modifying Spotify."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'scripts/splinter-setup-spicetify'


class SpicetifySetupTests(unittest.TestCase):
    def run_setup(self, installed=False, backup_fails=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            bin_dir = root / 'bin'
            bin_dir.mkdir()
            config = root / 'config'
            if installed:
                marketplace = config / 'spicetify/CustomApps/marketplace'
                marketplace.mkdir(parents=True)
                (marketplace / 'index.js').touch()
            commands = {
                'spotify': '#!/bin/sh\nexit 0\n',
                'sudo': '#!/bin/sh\nexit 0\n',
                'spicetify': '''#!/bin/sh
printf '%s\\n' "$*" >> "$TEST_ROOT/calls"
case "$1" in
  backup)
    [ "$FAIL_BACKUP" = 0 ] || exit 1
    touch "$TEST_ROOT/backup"
    ;;
  apply) [ -f "$TEST_ROOT/backup" ] || exit 1 ;;
esac
''',
                'curl': '''#!/bin/sh
printf 'download\\n' >> "$TEST_ROOT/calls"
while [ "$#" -gt 0 ]; do
  if [ "$1" = -o ]; then
    shift
    cat > "$1" <<'INSTALLER'
#!/bin/sh
set -e
spicetify config custom_apps marketplace
spicetify apply
INSTALLER
    exit 0
  fi
  shift
done
exit 1
''',
            }
            for name, contents in commands.items():
                path = bin_dir / name
                path.write_text(contents)
                path.chmod(0o755)
            env = dict(os.environ, XDG_CONFIG_HOME=str(config), TEST_ROOT=str(root),
                       PATH=str(bin_dir) + ':' + os.environ['PATH'],
                       FAIL_BACKUP='1' if backup_fails else '0')
            result = subprocess.run([str(SCRIPT)], env=env, capture_output=True, text=True)
            return result, (root / 'calls').read_text().splitlines()

    def test_fresh_marketplace_has_backup_before_its_apply(self):
        result, calls = self.run_setup()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertLess(calls.index('backup'), calls.index('download'))
        self.assertEqual(calls.count('apply'), 2)
        self.assertEqual(calls[-1], 'apply')
        self.assertLess(calls.index('config current_theme SplinterDots'), len(calls) - 1)

    def test_existing_marketplace_does_not_download_again(self):
        result, calls = self.run_setup(installed=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn('download', calls)
        self.assertEqual(calls[0], 'backup')
        self.assertEqual(calls[-1], 'apply')

    def test_backup_failure_stops_before_marketplace(self):
        result, calls = self.run_setup(backup_fails=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, ['backup'])
        self.assertNotIn('configured and applied.', result.stdout)


if __name__ == '__main__':
    unittest.main()
