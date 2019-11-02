import os
import platform
import shutil
import sys


def get_resource(file=''):
    if getattr(sys, 'frozen', False):
        base_dir = getattr(sys, '_MEIPASS', '')
    else:
        base_dir = os.path.dirname(os.path.abspath(__file__))
    return os.path.join(base_dir, file)


def get_adb_exe():
    pre_installed_adb_exe = shutil.which('adb')
    if pre_installed_adb_exe is None:
        local_adb_exe = os.path.join('tools', platform.system(), 'adb')
        return get_resource(local_adb_exe)
    return pre_installed_adb_exe
