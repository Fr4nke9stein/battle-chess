@echo off
rem Double-click: build the game and install or update it with a desktop shortcut.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
pause
