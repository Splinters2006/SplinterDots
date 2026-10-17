import os
from pathlib import Path
import runpy
import shutil
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def load(data_home, cache_home):
    with patch.dict(os.environ, {'XDG_DATA_HOME': str(data_home), 'XDG_CACHE_HOME': str(cache_home)}):
        return runpy.run_path(str(ROOT / 'scripts/splinter-add-style'))


def make_icon_theme(folder, name='MyIcons'):
    theme = folder / name
    (theme / '48x48/apps').mkdir(parents=True)
    (theme / 'index.theme').write_text('[Icon Theme]\nName=My Icons\nDirectories=48x48/apps\n')
    (theme / '48x48/apps/firefox.png').write_bytes(b'png')
    return theme


def make_cursor_theme(folder, name='MyCursors'):
    theme = folder / name
    (theme / 'cursors').mkdir(parents=True)
    (theme / 'cursors/left_ptr').write_bytes(b'Xcur')
    (theme / 'index.theme').write_text('[Icon Theme]\nName=My Cursors\n')
    return theme


class AddStyleTests(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.data, self.cache = self.tmp / 'data', self.tmp / 'cache'
        self.add = load(self.data, self.cache)
        self.add['notify'] = lambda message: None
        self.add['main'].__globals__['notify'] = lambda message: None

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def test_icon_theme_from_tar_archive(self):
        src = self.tmp / 'src'
        make_icon_theme(src)
        archive = self.tmp / 'icons.tar.gz'
        with tarfile.open(archive, 'w:gz') as bundle:
            bundle.add(src / 'MyIcons', arcname='MyIcons')
        self.assertEqual(self.add['main'](['icons', str(archive)]), 0)
        self.assertTrue((self.data / 'icons/MyIcons/48x48/apps/firefox.png').exists())
        self.assertTrue((self.cache / 'splinterdots/addons-refresh').exists())

    def test_cursor_theme_from_zip_and_from_index_file(self):
        src = self.tmp / 'src'
        make_cursor_theme(src)
        archive = self.tmp / 'cursors.zip'
        with zipfile.ZipFile(archive, 'w') as bundle:
            for path in (src / 'MyCursors').rglob('*'):
                bundle.write(path, path.relative_to(src))
        self.assertEqual(self.add['main'](['cursors', str(archive)]), 0)
        self.assertTrue((self.data / 'icons/MyCursors/cursors/left_ptr').exists())

        other = make_cursor_theme(self.tmp / 'other', 'OtherCursors')
        self.assertEqual(self.add['main'](['cursors', str(other / 'index.theme')]), 0)
        self.assertTrue((self.data / 'icons/OtherCursors/cursors/left_ptr').exists())

    def test_rejects_non_themes_and_unsafe_archives(self):
        cursors = make_cursor_theme(self.tmp / 'src')
        # A cursor-only theme is not an icon theme.
        self.assertEqual(self.add['main'](['icons', str(cursors / 'index.theme')]), 1)
        archive = self.tmp / 'evil.zip'
        with zipfile.ZipFile(archive, 'w') as bundle:
            bundle.writestr('../escape/index.theme', '[Icon Theme]\nDirectories=a\n')
        self.assertEqual(self.add['main'](['icons', str(archive)]), 1)
        self.assertFalse((self.tmp / 'escape').exists())

    @unittest.skipUnless(shutil.which('fc-scan') and shutil.which('fc-match'), 'fontconfig not installed')
    def test_fonts_are_copied_and_non_fonts_skipped(self):
        font = Path(subprocess.run(['fc-match', '-f', '%{file}', 'sans'], capture_output=True, text=True).stdout)
        if not font.is_file() or font.suffix.lower() not in ('.ttf', '.otf'):
            self.skipTest('no TTF/OTF system font to copy')
        text = self.tmp / 'notes.txt'
        text.write_text('not a font')
        self.assertEqual(self.add['main'](['fonts', str(font), str(text)]), 0)
        self.assertTrue((self.data / 'fonts/SplinterDots' / font.name).exists())
        self.assertFalse((self.data / 'fonts/SplinterDots/notes.txt').exists())


if __name__ == '__main__':
    unittest.main()
