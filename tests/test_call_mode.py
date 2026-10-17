from pathlib import Path
import runpy
import unittest

ROOT = Path(__file__).resolve().parents[1]
CALL = runpy.run_path(str(ROOT / 'scripts/splinter-call-mode'))

PROFILES = {
    'off': {'priority': 0, 'available': True},
    'a2dp-sink-sbc': {'priority': 132, 'available': True},
    'a2dp-sink': {'priority': 135, 'available': True},  # aptX HD
    'a2dp-sink-ldac': {'priority': 140, 'available': False},
    'headset-head-unit-cvsd': {'priority': 7, 'available': True},
    'headset-head-unit': {'priority': 8, 'available': True},  # mSBC
}


class CallModeTests(unittest.TestCase):
    def test_best_available_profile_per_mode(self):
        self.assertEqual(CALL['best_profile'](PROFILES, CALL['PLAYBACK_PREFIX']), 'a2dp-sink')
        self.assertEqual(CALL['best_profile'](PROFILES, CALL['CALL_PREFIX']), 'headset-head-unit')
        self.assertIsNone(CALL['best_profile']({'off': {}}, CALL['CALL_PREFIX']))

    def test_mode_and_address_from_card(self):
        self.assertEqual(CALL['mode_of']({'active_profile': 'headset-head-unit'}), 'on')
        self.assertEqual(CALL['mode_of']({'active_profile': 'a2dp-sink'}), 'off')
        self.assertEqual(CALL['card_address']('bluez_card.80_C3_BA_4D_75_80'), '80:C3:BA:4D:75:80')

    def test_detects_headset_mic_linked_to_easyeffects(self):
        linked = 'bluez_input.x:capture_MONO\n  |-> ee_sie_rnnoise:input_FL\n'
        unlinked = ('bluez_input.x.0:capture_MONO\n  |-> bluez_capture_internal.x:input_MONO\n'
                    'ee_sie_rnnoise:input_FL\n  |<- bluez_input.x:capture_MONO\n')
        self.assertTrue(CALL['mic_reaches_easyeffects'](linked))
        self.assertFalse(CALL['mic_reaches_easyeffects'](unlinked))

    def test_dead_call_link_is_exact_silence(self):
        self.assertEqual(CALL['rms']([0] * 100), 0.0)
        self.assertGreater(CALL['rms']([3, -2, 4, -5]), 1.0)


if __name__ == '__main__':
    unittest.main()
