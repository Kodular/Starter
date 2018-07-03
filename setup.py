from cx_Freeze import setup, Executable

include_files = ["adb.exe", "AdbWinApi.dll", "AdbWinUsbApi.dll"]

options = {"include_files": include_files}

executables = [Executable("MakeroidStarter.py",
               icon = "icon.ico")]

setup(name = "Makeroid Starter",
      version = "1",
      description = "Starter for USB Connection",
      options = {"build_exe": options},
      executables = executables)
