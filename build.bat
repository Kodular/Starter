@echo off
pyinstaller --onefile --debug --icon=icon.ico --noconfirm --clean windows.py
pause
