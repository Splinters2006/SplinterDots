"""Bar status and actions. Embedded in generated QML; no helper installation needed."""
import json
import os
from pathlib import Path
import random
import shutil
import subprocess
import sys


def run(args, **kwargs):
    return subprocess.run(args, capture_output=True, text=True, timeout=kwargs.pop('timeout', 5), **kwargs)


def choose(prompt, choices):
    result = run(['wofi', '--dmenu', '--prompt', prompt], input='\n'.join(choices), timeout=None)
    return result.stdout.strip() if result.returncode == 0 else ''


def keyboard():
    data = json.loads(run(['hyprctl', '-j', 'devices']).stdout)
    keyboards = data.get('keyboards', [])
    main = next((k for k in keyboards if k.get('main')), next(iter(keyboards), {}))
    return main.get('active_keymap', 'Unknown')


def updates(command):
    if command.strip() in ('checkupdates', 'checkupdates 2>/dev/null | wc -l'):
        result = run(['checkupdates'], timeout=120)
        if result.returncode not in (0, 2):
            return 'Updates ?'
        return 'Updates ' + str(len(result.stdout.splitlines()))
    result = run(['sh', '-c', command], timeout=120)
    return 'Updates ' + (result.stdout.strip() if result.returncode == 0 else '?')


def notifications():
    data = json.loads(run(['makoctl', 'list', '-j']).stdout)
    groups = data.get('data', []) if isinstance(data, dict) else data
    count = sum(len(group) if isinstance(group, list) else 1 for group in groups)
    return f'Notifications {count}'


def notification_menu():
    choice = choose('Notifications', ['Restore latest', 'Dismiss all'])
    if choice == 'Restore latest':
        return run(['makoctl', 'restore']).returncode
    if choice == 'Dismiss all':
        return run(['makoctl', 'dismiss', '--all']).returncode
    return 0


def power_menu():
    choice = choose('Power', ['Sleep', 'Lock', 'Reboot', 'Power off'])
    commands = {'Sleep': ['systemctl', 'suspend'], 'Reboot': ['systemctl', 'reboot'],
                'Power off': ['systemctl', 'poweroff']}
    if choice == 'Lock':
        if not shutil.which('hyprlock'):
            raise ValueError('Install hyprlock to use the Lock action')
        command = ['hyprlock']
        subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
    elif choice in commands:
        return run(commands[choice], timeout=30).returncode
    return 0


def wallpaper_choice(files, current, direction):
    if not files:
        raise ValueError('No wallpapers found in the configured folder')
    if direction == 'shuffle':
        return random.choice([p for p in files if p != current] or files)
    if current not in files:
        return files[-1] if direction == 'prev' else files[0]
    return files[(files.index(current) + (-1 if direction == 'prev' else 1)) % len(files)]


def wallpaper(direction):
    config = Path(os.environ.get('XDG_CONFIG_HOME', str(Path.home() / '.config'))) / 'dotfiles/local.conf'
    # This is the same trusted shell configuration used by splinter-wallpaper.
    result = run(['bash', '-c', 'source "$1"; printf "%s\\0%s" "${DOTFILES_WALLPAPER_DIR:-$HOME/Pictures/Wallpapers}" "${DOTFILES_WALLPAPER:-}"', '_', str(config)])
    if result.returncode:
        raise ValueError('Could not read wallpaper settings')
    folder, current = result.stdout.split('\0', 1)
    supported = {'.png', '.jpg', '.jpeg', '.webp', '.gif', '.bmp', '.tiff', '.avif'}
    files = sorted((p for p in Path(folder).expanduser().rglob('*') if p.is_file() and p.suffix.lower() in supported), key=lambda p: str(p).casefold())
    selected = wallpaper_choice(files, Path(current).expanduser(), direction)
    return run(['splinter-wallpaper', 'set', str(selected)], timeout=30).returncode


def main(args):
    action = args[0]
    try:
        if action == 'keyboard':
            print(keyboard())
        elif action == 'keyboard-next':
            return run(['hyprctl', 'switchxkblayout', 'all', 'next']).returncode
        elif action == 'updates':
            print(updates(args[1]))
        elif action == 'notifications':
            print(notifications())
        elif action == 'notification-menu':
            return notification_menu()
        elif action == 'power':
            return power_menu()
        elif action == 'wallpaper':
            return wallpaper(args[1])
        elif action == 'script':
            result = run(['sh', '-c', args[1]])
            print(result.stdout.strip()[:250] if result.returncode == 0 else 'Script error')
        else:
            raise ValueError('Unknown widget action')
        return 0
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        if action in ('keyboard', 'updates', 'notifications', 'script'):
            print({'keyboard': 'Keyboard ?', 'updates': 'Updates ?', 'notifications': 'Notifications ?', 'script': 'Script error'}[action])
        else:
            print(str(error), file=sys.stderr)
            if shutil.which('notify-send'):
                try:
                    run(['notify-send', 'SplinterDots', str(error)])
                except (OSError, subprocess.TimeoutExpired):
                    pass
        return 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
