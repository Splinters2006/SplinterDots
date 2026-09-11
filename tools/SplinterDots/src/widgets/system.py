"""One portable Linux system summary; missing sensors are shown as unavailable."""
from pathlib import Path
import shutil
import time


def cpu_sample():
    fields = list(map(int, Path('/proc/stat').read_text().splitlines()[0].split()[1:9]))
    return sum(fields), fields[3] + fields[4]


def summary(disk_path='/'):
    cpu = ram = temperature = disk = 'N/A'
    try:
        total, idle = cpu_sample()
        time.sleep(0.2)
        total2, idle2 = cpu_sample()
        delta = total2 - total
        used = 100 * (1 - (idle2 - idle) / delta) if delta > 0 else 0
        cpu = f'{max(0, min(100, round(used)))}%'
    except (OSError, ValueError, IndexError):
        pass
    try:
        memory = {line.split(':')[0]: int(line.split()[1]) for line in Path('/proc/meminfo').read_text().splitlines()}
        ram = f"{(memory['MemTotal'] - memory['MemAvailable']) / 1048576:.1f}/{memory['MemTotal'] / 1048576:.1f} GiB"
    except (OSError, ValueError, KeyError):
        pass
    readings = []
    for sensor in Path('/sys/class/hwmon').glob('hwmon*/temp*_input'):
        try:
            value = int(sensor.read_text()) / 1000
            if 0 < value < 150:
                name = (sensor.parent / 'name').read_text().strip()
                readings.append((name in ('coretemp', 'k10temp', 'zenpower', 'cpu_thermal'), value))
        except (OSError, ValueError):
            continue
    if readings:
        preferred = [value for cpu_sensor, value in readings if cpu_sensor]
        temperature = f'{max(preferred or [value for _, value in readings]):.0f}°C'
    try:
        usage = shutil.disk_usage(disk_path)
        disk = f'{usage.used / usage.total:.0%}'
    except OSError:
        pass
    return f'CPU {cpu} · RAM {ram} · Temp {temperature} · Disk {disk}'


if __name__ == '__main__':
    import sys
    print(summary(sys.argv[1] if len(sys.argv) > 1 else '/'))
