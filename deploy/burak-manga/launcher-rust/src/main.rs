// Dosya Yolu: REPO_ROOT/deploy/burak-manga/launcher-rust/src/main.rs
// Amac: Native Windows GUI uzerinden Suwayomi Docker servisini yonetir
// Modul - Rust
// Version: 0.9.0
// Aciklama: Tasinabilir compose, sinirli Docker komutlari, asenkron arayuz
// Bagimli Oldugu Katman: View / Service / Tool
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

mod archive;
mod diagnostics;
mod git_update;
mod webui_install;
mod lan_access;

const CATALOG: &str = "https://raw.githubusercontent.com/b1glord/Suwayomi-Server-Extensions/repo/index.pb";
const WEB: &str = "http://127.0.0.1:4567";
const OKUTOON_MARTIAL_PEAK: &str = "https://okutoon.com/martial-peak";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action { Start, Stop, Restart, Status, Logs, Archive, Diagnostics, PauseDownloads, GitPull, TurkishInstall, TurkishRestore, LanEnable, LanDisable }

impl Action {
    fn args(self, file: &Path) -> Vec<OsString> {
        let mut args = lan_access::compose_args(file);
        let suffix: &[&str] = match self {
            // Docker Compose starts both helpers via cf-router dependencies.
            Self::Start => &["up", "-d", "suwayomi"],
            Self::Stop => &["stop", "suwayomi", "cf-router", "flaresolverr", "flaresolverr-official"],
            Self::Restart => &["restart", "flaresolverr", "flaresolverr-official", "cf-router", "suwayomi"],
            Self::Status => &["ps", "--status", "running", "--services"],
            Self::Logs => &["logs", "--tail", "100", "--no-color", "suwayomi", "cf-router", "flaresolverr", "flaresolverr-official"],
            Self::Archive | Self::Diagnostics | Self::PauseDownloads | Self::GitPull | Self::TurkishInstall | Self::TurkishRestore | Self::LanEnable | Self::LanDisable => &[],
        };
        args.extend(suffix.iter().map(OsString::from));
        args
    }
    fn title(self) -> &'static str {
        match self {
            Self::Start => "Başlatma",
            Self::Stop => "Durdurma",
            Self::Restart => "Yeniden baslatma",
            Self::Status => "Sunucu durumu",
            Self::Logs => "Günlükler",
            Self::Archive => "Arşiv istatistikleri",
            Self::Diagnostics => "İndirme tanılama",
            Self::PauseDownloads => "İndirmeleri duraklatma",
            Self::GitPull => "Git Pull",
            Self::TurkishInstall => "Türkçe WebUI kurulumu",
            Self::TurkishRestore => "Türkçe WebUI geri alma",
            Self::LanEnable => "Telefon erişimini açma",
            Self::LanDisable => "Telefon erişimini kapatma",
        }
    }
}

fn compose_file(exe: &Path) -> Result<PathBuf, String> {
    let parent = exe.parent().ok_or("EXE_FOLDER_MISSING")?;
    let file = parent.join("compose.yaml");
    if !file.is_file() {
        return Err(format!("COMPOSE_NOT_FOUND: {}. EXE'yi compose.yaml yanina kopyalayin.", file.display()));
    }
    let contents = std::fs::read_to_string(&file).map_err(|e| format!("COMPOSE_READ_FAILED: {e}"))?;
    if !contents.contains("suwayomi:") || !contents.contains("127.0.0.1:4567:4567") {
        return Err("UNEXPECTED_COMPOSE: Bu proje icin olusturulan localhost ayari gerekli".into());
    }
    Ok(file)
}

#[cfg(windows)]
mod win {
    use super::{archive, diagnostics, git_update, lan_access, webui_install, Action, CATALOG, OKUTOON_MARTIAL_PEAK, WEB};
    use std::collections::HashMap;
    use std::ffi::{OsStr, OsString};
    use std::mem;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::os::windows::fs::MetadataExt;
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::ptr;
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::thread;
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{GetStockObject, DEFAULT_GUI_FONT};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::Controls::Dialogs::{CommDlgExtendedError, GetOpenFileNameW, OPENFILENAMEW, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_PATHMUSTEXIST};
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    const HEADER: usize = 101;
    const STATUS: usize = 102;
    const URL: usize = 103;
    const LOG: usize = 104;
    const START: usize = 110;
    const STOP: usize = 111;
    const RESTART: usize = 112;
    const REFRESH: usize = 113;
    const WEB_BUTTON: usize = 114;
    const LOGS: usize = 115;
    const COPY: usize = 116;
    const OPEN: usize = 117;
    const ARCHIVE_LABEL: usize = 118;
    const ARCHIVE_OPEN: usize = 119;
    const ARCHIVE_REFRESH: usize = 120;
    const OKUTOON: usize = 121;
    const DIAG_LABEL: usize = 122;
    const DIAG_REFRESH: usize = 123;
    const GUARD_TOGGLE: usize = 124;
    const PAUSE_DOWNLOADS: usize = 125;
    const GIT_PULL: usize = 126;
    const TURKISH_PACKAGE: usize = 127;
    const TURKISH_INSTALL: usize = 128;
    const TURKISH_RESTORE: usize = 129;
    const LAN_LABEL: usize = 130;
    const LAN_INPUT: usize = 131;
    const LAN_ENABLE: usize = 132;
    const LAN_DISABLE: usize = 133;
    const DIAG_POLL_INTERVAL: Duration = Duration::from_secs(60);
    const TIMER_ID: usize = 5;
    const GUI_SMOKE_TIMER: usize = 6;
    const EM_SETSEL: u32 = 0x00B1;
    const WM_COPY: u32 = 0x0301;
    const WM_SETFONT: u32 = 0x0030;
    const ES_READONLY: u32 = 0x0800;
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(Some(0)).collect()
    }
    fn write(handle: HWND, text: &str) {
        if !handle.is_null() {
            unsafe { SetWindowTextW(handle, wide(text).as_ptr()); }
        }
    }
    pub fn error(message: &str) {
        unsafe { MessageBoxW(ptr::null_mut(), wide(message).as_ptr(), wide("Burak Manga Launcher").as_ptr(), MB_OK | MB_ICONERROR); }
    }

    struct Done {
        action: Action,
        output: Result<String, String>,
        status: Result<String, String>,
        health: Option<diagnostics::DownloadHealth>,
    }
    struct App {
        compose: PathBuf,
        controls: HashMap<usize, HWND>,
        sender: Sender<Done>,
        receiver: Receiver<Done>,
        busy: bool,
        guard_enabled: bool,
        selected_turkish_zip: Option<PathBuf>,
        selected_lan_ip: Option<String>,
        auto_pause_issued: bool,
        last_diagnostics: Instant,
    }
    impl App {
        fn new(compose: PathBuf) -> Self {
            let (sender, receiver) = mpsc::channel();
            Self { compose, controls: HashMap::new(), sender, receiver, busy: false,
                guard_enabled: false, selected_turkish_zip: None, selected_lan_ip: None, auto_pause_issued: false,
                last_diagnostics: Instant::now() }
        }
        fn control(&self, id: usize) -> HWND { self.controls.get(&id).copied().unwrap_or(ptr::null_mut()) }
        fn label(&self, message: &str) { write(self.control(STATUS), message); }
        fn log(&self, message: &str) { write(self.control(LOG), message); }
        fn launch(&mut self, action: Action) {
            if self.busy { return; }
            self.busy = true;
            if action == Action::Diagnostics {
                self.last_diagnostics = Instant::now();
            }
            self.label(&format!("{} islemi devam ediyor...", action.title()));
            let path = self.compose.clone();
            let zip = self.selected_turkish_zip.clone();
            let ip = self.selected_lan_ip.clone();
            let tx = self.sender.clone();
            thread::spawn(move || {
                let raw = match action {
                    Action::TurkishInstall => webui_manage(webui_install::Operation::Install, &path, zip.as_deref()),
                    Action::TurkishRestore => webui_manage(webui_install::Operation::Restore, &path, None),
                    Action::LanEnable => lan_manage(lan_access::Mode::Enable, &path, ip.as_deref()),
                    Action::LanDisable => lan_manage(lan_access::Mode::Disable, &path, None),
                    _ => perform(action, &path),
                };
                let health = if action == Action::Diagnostics {
                    raw.as_ref().ok().map(|logs| diagnostics::parse(logs))
                } else { None };
                let output = if let Some(ref data) = health {
                    Ok(diagnostics::summary(data))
                } else { raw };
                let status = if action == Action::GitPull {
                    Ok(String::new())
                } else if action == Action::Status {
                    output.clone()
                } else {
                    perform(Action::Status, &path)
                };
                let _ = tx.send(Done { action, output, status, health });
            });
        }
        fn poll(&mut self) {
            while let Ok(done) = self.receiver.try_recv() {
                self.busy = false;
                let git_ok = done.action == Action::GitPull && done.output.is_ok();
                let turkish_ok = matches!(done.action, Action::TurkishInstall | Action::TurkishRestore) && done.output.is_ok();
                match done.output {
                    Ok(output) => {
                        match done.action {
                            Action::Archive => write(self.control(ARCHIVE_LABEL), &output),
                            Action::Diagnostics => write(self.control(DIAG_LABEL), &output),
                            Action::PauseDownloads => {
                                write(self.control(DIAG_LABEL),
                                    "İndirmeler duraklatıldı; kuyruk ve dosyalar korunuyor.");
                            }
                            _ => {}
                        }
                        self.log(&format!("{}:\r\n\r\n{output}", done.action.title()));
                    }
                    Err(reason) => {
                        match done.action {
                            Action::Archive => write(self.control(ARCHIVE_LABEL), "Arşiv okunamadı; ayrıntılar günlükte."),
                            Action::Diagnostics => write(self.control(DIAG_LABEL), "Tanılama kullanılamıyor. Docker günlüğünü kontrol edin."),
                            Action::PauseDownloads => write(self.control(DIAG_LABEL), "Duraklatma doğrulanamadı. WebUI üzerinden deneyin."),
                            _ => {}
                        }
                        self.log(&format!("{} HATASI:\r\n\r\n{reason}", done.action.title()));
                    }
                }
                if done.action == Action::GitPull {
                    self.label(if git_ok {
                        "Git Pull tamamlandı. Docker için Başlat düğmesini kullanın."
                    } else {
                        "Git Pull başarısız. Ayrıntılar işlem günlüğünde."
                    });
                } else if matches!(done.action, Action::TurkishInstall | Action::TurkishRestore) {
                    self.label(if turkish_ok {
                        "WebUI dosyaları güncellendi; dil için WebUI Ayarları'nı kontrol edin."
                    } else {
                        "WebUI işlemi tamamlanamadı. Ayrıntılar günlükte."
                    });
                } else {
                    self.label(&done.status.unwrap_or_else(|e| format!("Docker baglantisi yok: {e}")));
                }
                if done.action == Action::Status {
                    self.launch(Action::Archive);
                } else if done.action == Action::Archive {
                    self.launch(Action::Diagnostics);
                } else if done.action == Action::Diagnostics
                    && self.guard_enabled && !self.auto_pause_issued
                    && done.health.as_ref().is_some_and(diagnostics::should_pause)
                {
                    self.auto_pause_issued = true;
                    self.launch(Action::PauseDownloads);
                }
            }
            if !self.busy && self.last_diagnostics.elapsed() >= DIAG_POLL_INTERVAL {
                self.launch(Action::Diagnostics);
            }
        }
    }

    fn collect_capped<R: Read>(mut input: R) -> Result<Vec<u8>, String> {
        const LIMIT: usize = 2 * 1024 * 1024;
        let mut output = Vec::new();
        let mut buffer = [0u8; 8192];
        loop {
            let n = input.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 { break; }
            let room = LIMIT.saturating_sub(output.len());
            output.extend_from_slice(&buffer[..n.min(room)]);
        }
        Ok(output)
    }

    fn run_process(
        program: &str, args: &[OsString], root: &Path, limit: Duration,
    ) -> Result<String, String> {
        let mut command = Command::new(program);
        if program == "git" {
            // Avoid interactive credential prompts inside a background GUI worker.
            command.env("GIT_TERMINAL_PROMPT", "0");
        }
        let mut child = command
            .args(args).current_dir(root).creation_flags(0x0800_0000)
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|e| format!("{program} baslatilamadi: {e}"))?;
        // Drain both pipes concurrently: large stack traces otherwise fill pipe buffers
        // before try_wait completes and make the GUI report a false timeout.
        let stdout = child.stdout.take().ok_or("DOCKER_STDOUT_MISSING")?;
        let stderr = child.stderr.take().ok_or("DOCKER_STDERR_MISSING")?;
        let reader_out = thread::spawn(move || collect_capped(stdout));
        let reader_err = thread::spawn(move || collect_capped(stderr));
        let start = Instant::now();
        let result = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) if start.elapsed() >= limit => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(format!("DOCKER_TIMEOUT: {} saniye", limit.as_secs()));
                }
                Ok(None) => thread::sleep(Duration::from_millis(100)),
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(format!("DOCKER_PROCESS_ERROR: {e}"));
                }
            }
        };
        let out = reader_out.join().map_err(|_| "DOCKER_STDOUT_THREAD_FAILED")??;
        let err = reader_err.join().map_err(|_| "DOCKER_STDERR_THREAD_FAILED")??;
        let status = result?;
        let out = String::from_utf8_lossy(&out);
        let err = String::from_utf8_lossy(&err);
        let combined = format!("{out}\n{err}").trim().to_owned();
        if !status.success() { Err(format!("{program}_COMMAND_FAILED: {combined}")) } else { Ok(combined) }
    }

    fn docker(args: &[OsString], compose: &Path, limit: Duration) -> Result<String, String> {
        let root = compose.parent().ok_or("COMPOSE_PARENT_MISSING")?;
        run_process("docker", args, root, limit)
    }

    fn git(args: &[&str], root: &Path, limit: Duration) -> Result<String, String> {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        run_process("git", &args, root, limit)
    }

    fn git_pull(compose: &Path) -> Result<String, String> {
        const CHECK_TIMEOUT: Duration = Duration::from_secs(20);
        let repo = git_update::repository_root(compose)?;
        let actual = git(&["rev-parse", "--show-toplevel"], &repo, CHECK_TIMEOUT)?;
        let branch = git(&["branch", "--show-current"], &repo, CHECK_TIMEOUT)?;
        let origin = git(&["remote", "get-url", "origin"], &repo, CHECK_TIMEOUT)?;
        let status = git(
            &["status", "--porcelain=v1", "--untracked-files=normal"],
            &repo, CHECK_TIMEOUT,
        )?;
        git_update::validate(&repo, &actual, &branch, &origin, &status)?;
        let updated = git(&git_update::pull_arguments(), &repo, Duration::from_secs(150))?;
        Ok(format!(
            concat!(
                "Git Pull tamamlandı (fast-forward only):\n",
                "{}\n\n",
                "Bu işlem Docker image veya Launcher EXE dosyasını otomatik güncellemez.\n",
                "Yeni Compose ayarları için Başlat'a basabilirsiniz.\n",
                "Yeni EXE için GitHub Actions test ZIP'ini kullanın."
            ),
            updated
        ))
    }

    // BM-LAN-009: IP is passed as an argv element, never interpolated into a shell.
    // PowerShell validates private IPv4, local NIC ownership, authentication,
    // Compose overlay content, and restores localhost on a recreate failure.
    fn lan_manage(mode: lan_access::Mode, compose: &Path, ip: Option<&str>) -> Result<String, String> {
        let script = lan_access::script_path(compose)?;
        let arguments = lan_access::script_arguments(mode, &script, ip)?;
        let root = compose.parent().ok_or("COMPOSE_PARENT_MISSING")?;
        run_process("powershell.exe", &arguments, root, Duration::from_secs(180))
    }

    // BM-WEBUI-008: Preflight before stopping containers; restart only if previously running.
    // Script and Docker command arguments are fixed. The user-selected ZIP is one argv element.
    fn webui_script(
        mode: webui_install::ScriptMode,
        compose: &Path,
        zip: Option<&Path>,
    ) -> Result<String, String> {
        let script = webui_install::script_path(compose)?;
        let args = webui_install::script_arguments(mode, &script, zip)?;
        let root = compose.parent().ok_or("COMPOSE_PARENT_MISSING")?;
        run_process("powershell.exe", &args, root, Duration::from_secs(240))
    }

    fn webui_manage(
        operation: webui_install::Operation,
        compose: &Path,
        zip: Option<&Path>,
    ) -> Result<String, String> {
        let preflight = webui_script(operation.preflight(), compose, zip)?;
        let was_running = webui_install::is_running(
            &docker(&Action::Status.args(compose), compose, Duration::from_secs(20))?,
            "suwayomi",
        );

        if was_running {
            // Requires API confirmation. If auth blocks pausing, leave server untouched.
            pause_downloads()?;
            docker(
                &[
                    OsString::from("compose"), OsString::from("-f"),
                    compose.as_os_str().to_owned(), OsString::from("stop"),
                    OsString::from("suwayomi"),
                ],
                compose,
                Duration::from_secs(60),
            )?;
        }

        let apply = webui_script(operation.apply(), compose, zip);
        let restart = if was_running {
            docker(
                &[
                    OsString::from("compose"), OsString::from("-f"),
                    compose.as_os_str().to_owned(), OsString::from("up"),
                    OsString::from("-d"), OsString::from("--no-deps"),
                    OsString::from("suwayomi"),
                ],
                compose,
                Duration::from_secs(120),
            )
        } else {
            Ok("Suwayomi daha once kapaliydi; kapali birakildi.".into())
        };

        match (apply, restart) {
            (Ok(applied), Ok(started)) => Ok(format!(
                "{preflight}\n\n{applied}\n\n{started}\n\nSuwayomi acilista indirme kuyrugunu otomatik devam ettirebilir.\nWebUI'den indirici durumunu kontrol edin ve Turkce dilini secin.",
            )),
            (Err(error), Ok(started)) => Err(format!(
                "WEBUI_APPLY_FAILED: {error}\nSuwayomi onceki calisma durumuna donduruldu: {started}"
            )),
            (Ok(applied), Err(error)) => Err(format!(
                "WEBUI_APPLIED_BUT_SERVER_START_FAILED:\n{applied}\n{error}\nVeri ve yedekler korunuyor; Suwayomi durumunu kontrol edin."
            )),
            (Err(apply_error), Err(restart_error)) => Err(format!(
                "WEBUI_APPLY_AND_RESTART_FAILED:\n{apply_error}\n{restart_error}\nYedekleri silmeyin."
            )),
        }
    }

    fn pause_downloads() -> Result<String, String> {
        // The upstream stopDownloader mutation preserves queued/downloaded chapters.
        // Communicate only with the known loopback GraphQL endpoint; never bypass auth.
        const BODY: &str = r#"{"query":"mutation LauncherPause { stopDownloader(input: {}) { downloadStatus { state } } }"}"#;
        let target: SocketAddr = "127.0.0.1:4567".parse().map_err(|e| format!("INVALID_LOOPBACK: {e}"))?;
        let mut stream = TcpStream::connect_timeout(&target, Duration::from_secs(4))
            .map_err(|e| format!("SUWAYOMI_NOT_REACHABLE: {e}"))?;
        stream.set_read_timeout(Some(Duration::from_secs(35))).map_err(|e| e.to_string())?;
        stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;
        let request = format!(
            "POST /api/graphql HTTP/1.1\r\nHost: 127.0.0.1:4567\r\nContent-Type: application/json\r\nAccept: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
            BODY.len(), BODY
        );
        stream.write_all(request.as_bytes()).map_err(|e| format!("SUWAYOMI_API_WRITE_FAILED: {e}"))?;
        let response = collect_capped(&mut stream).map_err(|e| format!("SUWAYOMI_API_READ_FAILED: {e}"))?;
        let text = String::from_utf8_lossy(&response);
        let header = text.lines().next().unwrap_or_default();
        if !(header.starts_with("HTTP/1.1 200") || header.starts_with("HTTP/1.0 200")) {
            return Err("SUWAYOMI_API_REJECTED: Giriş veya API yetkisi gerekebilir. WebUI > Downloads üzerinden duraklatın.".into());
        }
        if text.contains("\"errors\"") || !text.contains("\"stopDownloader\"")
            || !text.contains("\"STOPPED\"")
        {
            return Err("SUWAYOMI_PAUSE_UNCONFIRMED: WebUI'den indirme durumunu kontrol edin.".into());
        }
        Ok("Suwayomi STOPPED bildirdi; kuyruk silinmedi.".into())
    }
    fn verify_folder(folder: &Path) -> Result<(), String> {
        if folder.exists() {
            let metadata = std::fs::symlink_metadata(folder).map_err(|e| e.to_string())?;
            if metadata.file_attributes() & 0x400 != 0 || !metadata.is_dir() {
                return Err(format!("INVALID_STORAGE_PATH: {}", folder.display()));
            }
        }
        Ok(())
    }
    fn perform(action: Action, compose: &Path) -> Result<String, String> {
        if action == Action::GitPull {
            return git_pull(compose);
        }
        if action == Action::PauseDownloads {
            return pause_downloads();
        }
        if action == Action::Diagnostics {
            return docker(
                &[OsString::from("compose"), OsString::from("-f"),
                  compose.as_os_str().to_owned(), OsString::from("logs"),
                  OsString::from("--since"), OsString::from("10m"),
                  OsString::from("--tail"), OsString::from("1200"),
                  OsString::from("--no-color"), OsString::from("suwayomi"),
                  OsString::from("cf-router")],
                compose, Duration::from_secs(20)
            );
        }
        if action == Action::Archive {
            let path = archive::downloads_path(compose)?;
            let stats = archive::scan(&path)?;
            return Ok(archive::summary(&stats));
        }
        if action == Action::Start {
            let root = compose.parent().ok_or("COMPOSE_PARENT_MISSING")?;
            let storage = root.join("storage");
            verify_folder(&storage)?;
            verify_folder(&storage.join("Tachidesk"))?;
            std::fs::create_dir_all(storage.join("Tachidesk")).map_err(|e| e.to_string())?;
            docker(&[OsString::from("info"), OsString::from("--format"), OsString::from("{{.ServerVersion}}")], compose, Duration::from_secs(20))?;
            let mut check = Action::Status.args(compose);
            check.truncate(3);
            check.extend([OsString::from("config"), OsString::from("--quiet")]);
            docker(&check, compose, Duration::from_secs(25))?;
        }
        let time = if action == Action::Start { Duration::from_secs(180) } else { Duration::from_secs(45) };
        let response = docker(&action.args(compose), compose, time)?;
        if action == Action::Status {
            let suwayomi_up = response.lines().any(|s| s.trim() == "suwayomi");
            let active = |name: &str| response.lines().any(|s| s.trim() == name);
            let server = if suwayomi_up { "Suwayomi: AÇIK" } else { "Suwayomi: KAPALI" };
            let router = if active("cf-router") { "Router: AÇIK" } else { "Router: KAPALI" };
            let byparr = if active("flaresolverr") { "Byparr: AÇIK" } else { "Byparr: KAPALI" };
            let flare = if active("flaresolverr-official") { "FlareSolverr: AÇIK" } else { "FlareSolverr: KAPALI" };
            Ok(format!("{server} | {router} | {byparr} | {flare}"))
        } else { Ok(response) }
    }

    unsafe fn make(hwnd: HWND, state: &mut App, id: usize, class: &str, label: &str, style: u32, border: u32) {
        let child = CreateWindowExW(border, wide(class).as_ptr(), wide(label).as_ptr(),
            style | WS_CHILD | WS_VISIBLE, 0, 0, 12, 12, hwnd, id as _,
            GetModuleHandleW(ptr::null()), ptr::null());
        if !child.is_null() {
            SendMessageW(child, WM_SETFONT, GetStockObject(DEFAULT_GUI_FONT) as usize, 1);
            state.controls.insert(id, child);
        }
    }
    unsafe fn children(hwnd: HWND, state: &mut App) {
        make(hwnd, state, HEADER, "STATIC", &format!("BURAK MANGA  |  RUST v{}", env!("CARGO_PKG_VERSION")), 0, 0);
        make(hwnd, state, STATUS, "STATIC", "Sunucu durumu kontrol ediliyor...", 0, 0);
        for (id, caption) in [
            (START, "Başlat"), (STOP, "Durdur"), (RESTART, "Yeniden Başlat"), (REFRESH, "Durumu Yenile"),
            (WEB_BUTTON, "WebUI Aç"), (LOGS, "Günlükler"), (GIT_PULL, "Git Pull"), (TURKISH_PACKAGE, "Türkçe Paketi Al"), (COPY, "Kataloğu Kopyala"), (OPEN, "Kataloğu Aç"),
            (TURKISH_INSTALL, "Türkçe WebUI Kur"), (TURKISH_RESTORE, "Türkçeyi Geri Al"),
            (LAN_ENABLE, "Telefon Erişimi Aç"), (LAN_DISABLE, "Telefon Erişimi Kapat"),
            (ARCHIVE_OPEN, "Arşiv Klasörünü Aç"), (ARCHIVE_REFRESH, "Arşivi Yenile"),
            (OKUTOON, "OkuToon / Martial Peak"),
            (DIAG_REFRESH, "İndirmeleri Kontrol Et"),
            (GUARD_TOGGLE, "Otomatik Koruma: Kapalı"),
            (PAUSE_DOWNLOADS, "İndirmeleri Duraklat")
        ] { make(hwnd, state, id, "BUTTON", caption, WS_TABSTOP, 0); }
        make(hwnd, state, URL, "EDIT", CATALOG, ES_READONLY, WS_EX_CLIENTEDGE);
        make(hwnd, state, LAN_LABEL, "STATIC", "Telefon IP (ipconfig):", 0, 0);
        make(hwnd, state, LAN_INPUT, "EDIT", "", WS_TABSTOP, WS_EX_CLIENTEDGE);
        make(hwnd, state, ARCHIVE_LABEL, "STATIC", "Arşiv taranıyor...", 0, 0);
        make(hwnd, state, DIAG_LABEL, "STATIC", "İndirme kontrolü başlatılacak...", 0, 0);
        make(hwnd, state, LOG, "EDIT", "İşlem günlükleri burada görünecek.",
            ES_READONLY | ES_MULTILINE | ES_AUTOVSCROLL | WS_VSCROLL, WS_EX_CLIENTEDGE);
    }
    unsafe fn layout(hwnd: HWND, state: &App) {
        let mut area: windows_sys::Win32::Foundation::RECT = mem::zeroed();
        GetClientRect(hwnd, &mut area);
        let width = (area.right - area.left).max(820);
        let height = (area.bottom - area.top).max(735);
        let margin = 25;
        let full = width - 2 * margin;
        let bw = (full - 36) / 4;
        let pos = |id, x, y, w, h| {
            let c = state.control(id);
            if !c.is_null() { MoveWindow(c, x, y, w, h, 1); }
        };
        pos(HEADER, margin, 20, full, 34);
        pos(STATUS, margin, 70, full, 28);
        for (index, id) in [START, STOP, RESTART, REFRESH].iter().enumerate() {
            pos(*id, margin + index as i32 * (bw + 12), 113, bw, 39);
        }
        pos(WEB_BUTTON, margin, 166, bw, 39);
        pos(LOGS, margin + bw + 12, 166, bw, 39);
        pos(GIT_PULL, margin + 2 * (bw + 12), 166, bw, 39);
        pos(TURKISH_PACKAGE, margin + 3 * (bw + 12), 166, bw, 39);
        pos(URL, margin, 231, full, 32);
        pos(COPY, margin, 278, bw, 39);
        pos(OPEN, margin + bw + 12, 278, bw, 39);
        pos(TURKISH_INSTALL, margin + 2 * (bw + 12), 278, bw, 39);
        pos(TURKISH_RESTORE, margin + 3 * (bw + 12), 278, bw, 39);
        pos(ARCHIVE_LABEL, margin, 329, full, 26);
        pos(ARCHIVE_OPEN, margin, 360, bw, 39);
        pos(ARCHIVE_REFRESH, margin + bw + 12, 360, bw, 39);
        pos(OKUTOON, margin + 2 * (bw + 12), 360, 2 * bw + 12, 39);
        pos(DIAG_LABEL, margin, 412, full, 33);
        pos(DIAG_REFRESH, margin, 455, bw + 70, 39);
        pos(GUARD_TOGGLE, margin + bw + 82, 455, bw + 35, 39);
        pos(PAUSE_DOWNLOADS, margin + 2 * bw + 128, 455, bw + 28, 39);
        pos(LAN_LABEL, margin, 520, bw, 32);
        pos(LAN_INPUT, margin + bw + 12, 513, bw, 37);
        pos(LAN_ENABLE, margin + 2 * (bw + 12), 513, bw, 39);
        pos(LAN_DISABLE, margin + 3 * (bw + 12), 513, bw, 39);
        pos(LOG, margin, 566, full, (height - 590).max(100));
    }
    fn browse_folder(hwnd: HWND, compose: &Path) -> Result<(), String> {
        let root = archive::downloads_path(compose)?;
        if !root.is_dir() {
            return Err("İndirme klasörü henüz oluşmamış.".into());
        }
        let meta = std::fs::symlink_metadata(&root).map_err(|e| e.to_string())?;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("ARCHIVE_REPARSE_POINT_NOT_ALLOWED".into());
        }
        let native: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
        let value = unsafe {
            ShellExecuteW(hwnd, wide("open").as_ptr(), native.as_ptr(),
                ptr::null(), ptr::null(), SW_SHOWNORMAL)
        };
        if (value as isize) <= 32 { Err("Arşiv klasörü açılamadı.".into()) } else { Ok(()) }
    }
    fn browse(hwnd: HWND, url: &str) -> Result<(), String> {
        let value = unsafe {
            ShellExecuteW(hwnd, wide("open").as_ptr(), wide(url).as_ptr(),
                ptr::null(), ptr::null(), SW_SHOWNORMAL)
        };
        if (value as isize) <= 32 { Err("Tarayici acilamadi".into()) } else { Ok(()) }
    }

    fn read_input(handle: HWND) -> String {
        if handle.is_null() { return String::new(); }
        let length = unsafe { GetWindowTextLengthW(handle) };
        let mut chars = vec![0u16; (length as usize).min(128) + 1];
        let read = unsafe { GetWindowTextW(handle, chars.as_mut_ptr(), chars.len() as i32) };
        String::from_utf16_lossy(&chars[..read as usize])
    }

    fn select_turkish_zip(hwnd: HWND) -> Result<Option<PathBuf>, String> {
        let mut buffer = vec![0u16; 32768];
        let filter: Vec<u16> =
            "Türkçe WebUI paketi\0Suwayomi-WebUI-TR-preview.zip\0ZIP dosyaları\0*.zip\0\0"
                .encode_utf16()
                .collect();
        let title = wide("Suwayomi Türkçe WebUI ZIP paketini seçin");
        let mut dialog: OPENFILENAMEW = unsafe { mem::zeroed() };
        dialog.lStructSize = mem::size_of::<OPENFILENAMEW>() as u32;
        dialog.hwndOwner = hwnd;
        dialog.lpstrFilter = filter.as_ptr();
        dialog.lpstrFile = buffer.as_mut_ptr();
        dialog.nMaxFile = buffer.len() as u32;
        dialog.lpstrTitle = title.as_ptr();
        dialog.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
        if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
            let code = unsafe { CommDlgExtendedError() };
            return if code == 0 {
                Ok(None)
            } else {
                Err(format!("WEBUI_ZIP_FILE_DIALOG_FAILED: {code}"))
            };
        }
        let end = buffer.iter().position(|&unit| unit == 0).unwrap_or(buffer.len());
        Ok(Some(PathBuf::from(OsString::from_wide(&buffer[..end]))))
    }

    unsafe extern "system" fn procedure(hwnd: HWND, message: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App;
        match message {
            WM_COMMAND if !state.is_null() => {
                let gui = &mut *state;
                match wp & 0xffff {
                    START => gui.launch(Action::Start),
                    STOP => gui.launch(Action::Stop),
                    RESTART => gui.launch(Action::Restart),
                    REFRESH => gui.launch(Action::Status),
                    LOGS => gui.launch(Action::Logs),
                    GIT_PULL => {
                        let answer = MessageBoxW(
                            hwnd,
                            wide("develop/burak-manga dalını sadece fast-forward ile güncelle.\nYerel değişiklik varsa işlem duracak. Devam edilsin mi?").as_ptr(),
                            wide("Git Pull onayı").as_ptr(),
                            MB_YESNO | MB_ICONQUESTION,
                        );
                        if answer == IDYES {
                            gui.launch(Action::GitPull);
                        }
                    },
                    LAN_ENABLE => {
                        if !gui.busy {
                            let ip = read_input(gui.control(LAN_INPUT));
                            let answer = MessageBoxW(
                                hwnd,
                                wide("Suwayomi yalnızca seçtiğiniz özel ağ IP adresinde telefona açılacak.\nÖNCE Suwayomi'de Authentication / UI Login etkin ve parola ayarlı olmalı.\n\nBu işlem yalnızca Suwayomi konteynerini yeniden oluşturur. Açılışta indirme kuyruğu devam edebilir.\n\nAynı Wi-Fi ağındaki cihazlar erişebilir; HTTP şifreli değildir.\nOnaylıyor musunuz?").as_ptr(),
                                wide("Telefon erişimi güvenlik onayı").as_ptr(),
                                MB_YESNO | MB_ICONQUESTION,
                            );
                            if answer == IDYES {
                                gui.selected_lan_ip = Some(ip);
                                gui.launch(Action::LanEnable);
                            }
                        }
                    },
                    LAN_DISABLE => {
                        if !gui.busy {
                            let answer = MessageBoxW(
                                hwnd,
                                wide("Telefon erişimi kapatılacak. Yalnızca localhost kalacak.\nSuwayomi yeniden oluşturulacak; diğer servisler etkilenmez.\nDevam edilsin mi?").as_ptr(),
                                wide("Telefon erişimini kapat").as_ptr(),
                                MB_YESNO | MB_ICONQUESTION,
                            );
                            if answer == IDYES { gui.launch(Action::LanDisable); }
                        }
                    },
                    TURKISH_PACKAGE => {
                        if let Err(e) = browse(hwnd, webui_install::WORKFLOW_URL) { gui.log(&e); }
                        else { gui.log("Actions > son başarılı Türkçe WebUI derlemesi > Artifacts bölümünden paketi indirin ve içindeki ZIP ile SHA256SUMS.txt dosyalarını aynı klasörde tutun."); }
                    },
                    TURKISH_INSTALL => {
                        if !gui.busy {
                            let answer = MessageBoxW(
                                hwnd,
                                wide("Türkçe test paketi kurulacak.\n\nÖnce WebUI > Settings > WebUI > Flavor = Custom seçin.\n\nİndirmeler duraklatılacak, yalnızca Suwayomi geçici durdurulacak; mevcut WebUI yedeklenecek.\nSuwayomi tekrar başladığında indirme kuyruğu otomatik devam edebilir.\n\nOnaylıyor musunuz?").as_ptr(),
                                wide("Türkçe WebUI kurulum onayı").as_ptr(),
                                MB_YESNO | MB_ICONQUESTION,
                            );
                            if answer == IDYES {
                                match select_turkish_zip(hwnd) {
                                    Ok(Some(zip)) => {
                                        gui.selected_turkish_zip = Some(zip);
                                        gui.launch(Action::TurkishInstall);
                                    }
                                    Ok(None) => gui.log("Dosya seçimi iptal edildi. Hiçbir değişiklik yapılmadı."),
                                    Err(e) => gui.log(&e),
                                }
                            }
                        }
                    },
                    TURKISH_RESTORE => {
                        if !gui.busy {
                            let answer = MessageBoxW(
                                hwnd,
                                wide("Türkçe WebUI yerine son kaydedilen orijinal WebUI geri getirilecek.\n\nİndirmeler duraklatılacak; sadece Suwayomi geçici duracak.\nAçılışta indirme kuyruğu otomatik devam edebilir.\n\nDevam edilsin mi?").as_ptr(),
                                wide("Türkçe WebUI geri alma onayı").as_ptr(),
                                MB_YESNO | MB_ICONQUESTION,
                            );
                            if answer == IDYES { gui.launch(Action::TurkishRestore); }
                        }
                    },
                    WEB_BUTTON => if let Err(e) = browse(hwnd, WEB) { gui.log(&e); },
                    OPEN => if let Err(e) = browse(hwnd, CATALOG) { gui.log(&e); },
                    OKUTOON => if let Err(e) = browse(hwnd, OKUTOON_MARTIAL_PEAK) { gui.log(&e); },
                    ARCHIVE_REFRESH => gui.launch(Action::Archive),
                    DIAG_REFRESH => gui.launch(Action::Diagnostics),
                    PAUSE_DOWNLOADS => gui.launch(Action::PauseDownloads),
                    GUARD_TOGGLE => {
                        gui.guard_enabled = !gui.guard_enabled;
                        gui.auto_pause_issued = false;
                        let caption = if gui.guard_enabled {
                            "Otomatik Koruma: Açık"
                        } else {
                            "Otomatik Koruma: Kapalı"
                        };
                        write(gui.control(GUARD_TOGGLE), caption);
                        gui.log("Koruma yalnızca aktif indirmelerde ve art arda Cloudflare hata satırlarında duraklatır; otomatik yeniden başlatmaz.");
                    },
                    ARCHIVE_OPEN => if let Err(e) = browse_folder(hwnd, &gui.compose) { gui.log(&e); },
                    COPY => {
                        SendMessageW(gui.control(URL), EM_SETSEL, 0, -1);
                        SendMessageW(gui.control(URL), WM_COPY, 0, 0);
                        gui.log("İmzalı katalog adresi panoya kopyalandı.");
                    }
                    _ => {}
                }
                0
            }
            WM_TIMER if wp == GUI_SMOKE_TIMER => { DestroyWindow(hwnd); 0 }
            WM_TIMER if !state.is_null() => { (&mut *state).poll(); 0 }
            WM_SIZE if !state.is_null() => { layout(hwnd, &*state); 0 }
            WM_DESTROY => { KillTimer(hwnd, TIMER_ID); PostQuitMessage(0); 0 }
            WM_NCDESTROY => {
                if !state.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    drop(Box::from_raw(state));
                }
                DefWindowProcW(hwnd, message, wp, lp)
            }
            _ => DefWindowProcW(hwnd, message, wp, lp),
        }
    }
    pub fn run(compose: PathBuf, smoke_gui: bool) -> Result<(), String> {
        unsafe {
            let instance = GetModuleHandleW(ptr::null());
            if instance.is_null() { return Err("GET_INSTANCE_FAILED".into()); }
            let name = wide("BurakMangaRustLauncherWindow");
            let mut class: WNDCLASSW = mem::zeroed();
            class.hInstance = instance;
            class.lpfnWndProc = Some(procedure);
            class.lpszClassName = name.as_ptr();
            class.hbrBackground = 6 as _; // COLOR_WINDOW + 1
            if RegisterClassW(&class) == 0 {
                return Err(format!("REGISTER_CLASS_FAILED: {}", std::io::Error::last_os_error()));
            }
            let hwnd = CreateWindowExW(
                0, name.as_ptr(), wide(&format!("Burak Manga Launcher v{}", env!("CARGO_PKG_VERSION"))).as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN, CW_USEDEFAULT, CW_USEDEFAULT,
                950, 855, ptr::null_mut(), ptr::null_mut(), instance, ptr::null(),
            );
            if hwnd.is_null() { return Err(format!("CREATE_WINDOW_FAILED: {}", std::io::Error::last_os_error())); }
            let ptr = Box::into_raw(Box::new(App::new(compose)));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, ptr as isize);
            children(hwnd, &mut *ptr);
            layout(hwnd, &*ptr);
            SetTimer(hwnd, TIMER_ID, 200, None);
            if smoke_gui && SetTimer(hwnd, GUI_SMOKE_TIMER, 1250, None) == 0 {
                DestroyWindow(hwnd);
                return Err("GUI_SMOKE_TIMER_FAILED".into());
            }
            ShowWindow(hwnd, SW_SHOW);
            (*ptr).launch(Action::Status);
            let mut msg: MSG = mem::zeroed();
            while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            Ok(())
        }
    }
}

fn main() {
    #[cfg(windows)]
    {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let check = args.iter().any(|arg| arg == "--check-layout");
        let smoke_gui = args.iter().any(|arg| arg == "--smoke-gui");
        let outcome = std::env::current_exe().map_err(|e| e.to_string())
            .and_then(|exe| compose_file(&exe))
            .and_then(|compose| if check { Ok(()) } else { win::run(compose, smoke_gui) });
        if let Err(e) = outcome {
            eprintln!("BURAK_MANGA_ERROR: {e}");
            if !check { win::error(&format!("Başlatici acilamadi:\n\n{e}")); }
            std::process::exit(1);
        }
    }
    #[cfg(not(windows))]
    {
        eprintln!("Burak Manga Launcher Windows 10/11 icindir.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compose_operations_are_service_scoped_and_non_destructive() {
        let path = Path::new("E:/Ornek Klasor/compose.yaml");
        for action in [Action::Start, Action::Stop, Action::Restart, Action::Status, Action::Logs] {
            let words: Vec<String> = action.args(path).iter().map(|v| v.to_string_lossy().into_owned()).collect();
            assert_eq!(words[0], "compose");
            assert_eq!(words[1], "-f");
            assert_eq!(words[2], "E:/Ornek Klasor/compose.yaml");
            assert!(!words.iter().any(|s| ["down", "-v", "prune", "rm", "pull"].contains(&s.as_str())));
        }
    }
    #[test]
    fn compose_operates_all_four_services_as_a_single_stack() {
        let compose = Path::new("E:/Manga Repo/compose.yaml");
        let words = |action: Action| -> Vec<String> {
            action.args(compose).iter()
                .map(|v| v.to_string_lossy().into_owned()).collect()
        };
        assert_eq!(&words(Action::Start)[3..], &["up", "-d", "suwayomi"]);
        assert_eq!(&words(Action::Stop)[3..],
            &["stop", "suwayomi", "cf-router", "flaresolverr", "flaresolverr-official"]);
        assert_eq!(&words(Action::Restart)[3..],
            &["restart", "flaresolverr", "flaresolverr-official", "cf-router", "suwayomi"]);
        assert_eq!(&words(Action::Logs)[3..],
            &["logs", "--tail", "100", "--no-color",
              "suwayomi", "cf-router", "flaresolverr", "flaresolverr-official"]);
    }
    #[test]
    fn missing_compose_fails_clearly() {
        assert!(compose_file(Path::new("Z:/not-present/BurakMangaLauncher.exe")).is_err());
    }
}
