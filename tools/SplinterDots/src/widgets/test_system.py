import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import system


class SystemSummaryTests(unittest.TestCase):
    def test_combines_sampled_cpu_memory_preferred_cpu_sensor_and_disk(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'proc').mkdir()
            (root / 'proc/meminfo').write_text('MemTotal: 8388608 kB\nMemAvailable: 3145728 kB\n')
            for index, (name, value) in enumerate([('coretemp', '55000'), ('amdgpu', '80000')]):
                sensor = root / f'sys/class/hwmon/hwmon{index}'
                sensor.mkdir(parents=True)
                (sensor / 'name').write_text(name)
                (sensor / 'temp1_input').write_text(value)
            with patch.object(system, 'Path', side_effect=lambda p: root / p.lstrip('/')), \
                 patch.object(system, 'cpu_sample', side_effect=[(100, 60), (200, 100)]), \
                 patch.object(system.time, 'sleep'), \
                 patch.object(system.shutil, 'disk_usage', return_value=system.shutil._ntuple_diskusage(100, 40, 60)):
                self.assertEqual(system.summary(), 'CPU 60% · RAM 5.0/8.0 GiB · Temp 55°C · Disk 40%')

    def test_missing_hardware_does_not_break_summary(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(system, 'Path', side_effect=lambda p: Path(directory) / p.lstrip('/')), \
                 patch.object(system.shutil, 'disk_usage', side_effect=OSError):
                self.assertEqual(system.summary('/missing'), 'CPU N/A · RAM N/A · Temp N/A · Disk N/A')

    def test_zero_cpu_delta_is_finite(self):
        with patch.object(system, 'cpu_sample', side_effect=[(100, 60), (100, 60)]), \
             patch.object(system.time, 'sleep'):
            self.assertIn('CPU 0%', system.summary())


if __name__ == '__main__':
    unittest.main()
