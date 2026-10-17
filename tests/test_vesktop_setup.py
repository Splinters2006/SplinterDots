import json
from pathlib import Path
import runpy
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SETUP = runpy.run_path(str(ROOT / 'scripts/splinter-setup-vesktop'))


class VesktopSetupTests(unittest.TestCase):
    def test_defaults_merge_into_existing_settings_once(self):
        with tempfile.TemporaryDirectory() as directory:
            settings = Path(directory) / 'settings/settings.json'
            settings.parent.mkdir()
            settings.write_text(json.dumps({'useQuickCss': True, 'plugins': {
                'NoTrack': {'enabled': False, 'disableAnalytics': False, 'kept': 1},
                'Other': {'enabled': True}}}))
            defaults = {'NoTrack': {'enabled': True, 'disableAnalytics': True}, 'ClearURLs': {'enabled': True}}

            self.assertTrue(SETUP['apply_default_plugins'](settings, defaults))
            data = json.loads(settings.read_text())
            self.assertEqual(data['plugins']['NoTrack'], {'enabled': True, 'disableAnalytics': True, 'kept': 1})
            self.assertEqual(data['plugins']['ClearURLs'], {'enabled': True})
            self.assertEqual(data['plugins']['Other'], {'enabled': True})
            self.assertTrue(data['useQuickCss'])
            self.assertTrue(settings.with_suffix('.json.before-splinterdots').exists())

            # A plugin the user disables later stays disabled on the next run.
            data['plugins']['ClearURLs']['enabled'] = False
            settings.write_text(json.dumps(data))
            self.assertFalse(SETUP['apply_default_plugins'](settings, defaults))
            self.assertFalse(json.loads(settings.read_text())['plugins']['ClearURLs']['enabled'])

    def test_repo_defaults_are_enabled_plugins(self):
        defaults = json.loads((ROOT / 'files/vesktop/plugins.json').read_text())
        self.assertTrue(defaults)
        self.assertTrue(all(options.get('enabled') is True for options in defaults.values()))


if __name__ == '__main__':
    unittest.main()
