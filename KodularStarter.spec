# -*- mode: python ; coding: utf-8 -*-
import os
import platform

OS = platform.system()

block_cipher = None

binaries = [(os.path.join('tools', str.lower(OS), '*'), '.')]

a = Analysis(['KodularStarter.py', 'utils.py'],
             pathex=['.'],
             binaries=binaries,
             datas=[],
             hiddenimports=[],
             hookspath=[],
             runtime_hooks=[],
             excludes=[],
             win_no_prefer_redirects=False,
             win_private_assemblies=False,
             cipher=block_cipher,
             noarchive=False)

pyz = PYZ(a.pure,
          a.zipped_data,
          cipher=block_cipher)

exe = EXE(pyz,
          a.scripts,
          a.binaries,
          a.zipfiles,
          a.datas,
          [],
          name='KodularStarter',
          debug=False,
          bootloader_ignore_signals=False,
          strip=False,
          upx=True,
          upx_exclude=[],
          runtime_tmpdir=None,
          console=True,
          icon='icon.ico')

coll = COLLECT(exe,
               a.binaries,
               a.zipfiles,
               a.datas,
               strip=False,
               upx=False,
               name='KodularStarter')

if OS == 'Darwin':
    info_plist = {
        'NSHighResolutionCapable': 'True',
        'NSPrincipalClass': 'NSApplication',
        'CFBundleName': 'KodularStarter',
        'CFBundleDisplayName': 'Kodular Starter',
        'CFBundleIdentifier': 'io.kodular.starter',
        'CFBundleVersion': '1',
        'CFBundleShortVersionString': '1',
        'LSMinimumSystemVersion': '10.12',
    }
    app = BUNDLE(coll,
                 name='KodularStarter.app',
                 icon='icon.icns',
                 bundle_identifier=None,
                 info_plist=info_plist
                 )