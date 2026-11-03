@echo off
REM Dosya Yolu: REPO_ROOT/deploy/burak-manga/Launcher.cmd
REM Amac: Windows grafik baslaticisini cift tiklamayla acar
REM Modul - CMD
REM Version: 0.2.0
REM Aciklama: Tasinabilir dosya yolu ile PowerShell WPF GUI girisi
REM Bagimli Oldugu Katman: View
setlocal
set "LAUNCHER=%~dp0scripts\launcher.ps1"
if not exist "%LAUNCHER%" (
    echo LAUNCHER_NOT_FOUND: "%LAUNCHER%"
    pause
    exit /b 1
)
start "Burak Manga Launcher" powershell.exe -NoLogo -NoProfile -STA -ExecutionPolicy Bypass -WindowStyle Hidden -File "%LAUNCHER%"
endlocal
