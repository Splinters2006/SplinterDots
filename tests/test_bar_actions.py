import importlib.util
import subprocess
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('bar_actions', Path(__file__).resolve().parents[1] / 'tools/SplinterDots/src/widgets/actions.py')
actions = importlib.util.module_from_spec(spec)
spec.loader.exec_module(actions)


def result(stdout='', code=0):
    return subprocess.CompletedProcess([], code, stdout, '')


class BarActionsTests(unittest.TestCase):
    def test_keyboard_prefers_main_device_with_formatted_json(self):
        with patch.object(actions, 'run', return_value=result('{"keyboards": [{"active_keymap": "Virtual"}, {"main": true, "active_keymap": "English (US)"}]}')):
            self.assertEqual(actions.keyboard(), 'English (US)')

    def test_updates_zero_and_failure_are_distinct(self):
        for response, expected in [(result('', 2), 'Updates 0'), (result('a 1 -> 2\nb 3 -> 4\n'), 'Updates 2'), (result('', 1), 'Updates ?')]:
            with patch.object(actions, 'run', return_value=response):
                self.assertEqual(actions.updates('checkupdates'), expected)

    def test_notifications_flat_and_grouped_json(self):
        for raw in ['[{"id": 1}, {"id": 2}]', '{"data": [[{"id": 1}], [{"id": 2}]]}']:
            with patch.object(actions, 'run', return_value=result(raw)):
                self.assertEqual(actions.notifications(), 'Notifications 2')

    def test_wallpaper_wraps_and_shuffle_skips_current(self):
        files = [Path('a.png'), Path('b.png'), Path('c.png')]
        self.assertEqual(actions.wallpaper_choice(files, files[0], 'prev'), files[2])
        self.assertEqual(actions.wallpaper_choice(files, files[2], 'next'), files[0])
        self.assertNotEqual(actions.wallpaper_choice(files, files[1], 'shuffle'), files[1])
        self.assertEqual(actions.wallpaper_choice(files[:1], files[0], 'shuffle'), files[0])
        with self.assertRaises(ValueError):
            actions.wallpaper_choice([], Path('none'), 'next')

    def test_wallpaper_uses_configured_directory_and_safe_argument(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            image = root / 'picture with spaces.png'
            image.touch()
            (root / 'ignore.txt').touch()
            with patch.object(actions, 'run', side_effect=[result(str(root) + '\0'), result()]) as run:
                self.assertEqual(actions.wallpaper('next'), 0)
                self.assertEqual(run.call_args.args[0], ['splinter-wallpaper', 'set', str(image)])

    def test_power_cancel_and_unknown_selection_do_nothing(self):
        for choice in ['', 'Cancel', 'poweroff', 'arbitrary command']:
            with patch.object(actions, 'choose', return_value=choice), patch.object(actions, 'run') as run, patch.object(actions.subprocess, 'Popen') as spawn:
                self.assertEqual(actions.power_menu(), 0)
                run.assert_not_called()
                spawn.assert_not_called()

    def test_power_actions_are_explicit(self):
        for label, verb in [('Sleep', 'suspend'), ('Reboot', 'reboot'), ('Power off', 'poweroff')]:
            with patch.object(actions, 'choose', return_value=label), patch.object(actions, 'run', return_value=result()) as run:
                self.assertEqual(actions.power_menu(), 0)
                run.assert_called_once_with(['systemctl', verb], timeout=30)
        with patch.object(actions, 'choose', return_value='Lock'), patch.object(actions.shutil, 'which', return_value='/usr/bin/hyprlock'), patch.object(actions.subprocess, 'Popen') as spawn:
            actions.power_menu()
            self.assertEqual(spawn.call_args.args[0], ['hyprlock'])

    def test_menu_cancel_and_notification_actions(self):
        with patch.object(actions, 'run', return_value=result('Sleep', 1)):
            self.assertEqual(actions.choose('Power', ['Sleep']), '')
        with patch.object(actions, 'choose', return_value='Restore latest'), patch.object(actions, 'run', return_value=result()) as run:
            actions.notification_menu()
            run.assert_called_once_with(['makoctl', 'restore'])

    def test_custom_command_timeout_is_bounded(self):
        with patch.object(actions.subprocess, 'run', return_value=result()) as run:
            actions.main(['script', 'printf hello'])
            self.assertEqual(run.call_args.kwargs['timeout'], 5)
            self.assertEqual(run.call_args.args[0], ['sh', '-c', 'printf hello'])


if __name__ == '__main__':
    unittest.main()
