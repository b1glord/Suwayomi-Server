@echo off
REM Dosya Yolu: REPO_ROOT/deploy/burak-manga/Launcher.cmd
REM Amac: Native Rust EXE'yi cift tiklama ile baslatir
REM Modul - CMD
REM Version: 0.3.0
REM Aciklama: PowerShell/WPF yerine derlenmis uygulamayi acar
REM Bagimli Oldugu Katman: View
setlocal
set "APP=%~dp0BurakMangaLauncher.exe"
if not exist "%APP%" (
  echo Rust baslatici bulunamadi: "%APP%"
  echo GitHub Actions ^> Burak Manga - Rust launcher ^> Artifacts
  echo EXE dosyasini compose.yaml dosyasinin yanina kopyalayin.
  pause
  exit /b 1
)
start "" "%APP%"
endlocal
