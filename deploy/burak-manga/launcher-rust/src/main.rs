// Dosya Yolu: REPO_ROOT/deploy/burak-manga/launcher-rust/src/main.rs
// Amac: Native Windows GUI uzerinden Suwayomi Docker servisini yonetir
// Modul - Rust
// Version: 0.4.0
// Aciklama: Tasinabilir compose, sinirli Docker komutlari, asenkron arayuz
// Bagimli Oldugu Katman: View / Service / Tool
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

mod archive;

const CATALOG: &str = "https://raw.githubusercontent.com/b1glord/Suwayomi-Server-Extensions/repo/index.pb";
const WEB: &str = "http://127.0.0.1:4567";
const OKUTOON_MARTIAL_PEAK: &str = "https://okutoon.com/martial-peak";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action { Start, Stop, Restart, Status, Logs, Archive }

impl Action {
    fn args(self, file: &Path) -> Vec<OsString> {
        let mut args = vec![OsString::from("compose"), OsString::from("-f"), file.as_os_str().to_owned()];
        let suffix: &[&str] = match self {
            Self::Start => &["up", "-d", "suwayomi"],
            Self::Stop => &["stop", "suwayomi"],
            Self::Restart => &["restart", "suwayomi"],
            Self::Status => &["ps", "--status", "running", "--services"],
            Self::Logs => &["logs", "--tail", "100", "--no-color", "suwayomi"],
            Self::Archive => &[],
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
    use super::{archive, Action, CATALOG, OKUTOON_MARTIAL_PEAK, WEB};
    use std::collections::HashMap;
    use std::ffi::{OsStr, OsString};
    use std::mem;
    use std::os::windows::ffi::OsStrExt;
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
    }
    struct App {
        compose: PathBuf,
        controls: HashMap<usize, HWND>,
        sender: Sender<Done>,
        receiver: Receiver<Done>,
        busy: bool,
    }
    impl App {
        fn new(compose: PathBuf) -> Self {
            let (sender, receiver) = mpsc::channel();
            Self { compose, controls: HashMap::new(), sender, receiver, busy: false }
        }
        fn control(&self, id: usize) -> HWND { self.controls.get(&id).copied().unwrap_or(ptr::null_mut()) }
        fn label(&self, message: &str) { write(self.control(STATUS), message); }
        fn log(&self, message: &str) { write(self.control(LOG), message); }
        fn launch(&mut self, action: Action) {
            if self.busy { return; }
            self.busy = true;
            self.label(&format!("{} islemi devam ediyor...", action.title()));
            let path = self.compose.clone();
            let tx = self.sender.clone();
            thread::spawn(move || {
                let output = perform(action, &path);
                let status = if action == Action::Status { output.clone() } else { perform(Action::Status, &path) };
                let _ = tx.send(Done { action, output, status });
            });
        }
        fn poll(&mut self) {
            while let Ok(done) = self.receiver.try_recv() {
                self.busy = false;
                match done.output {
                    Ok(output) => {
                        if done.action == Action::Archive {
                            write(self.control(ARCHIVE_LABEL), &output);
                        }
                        self.log(&format!("{}:\r\n\r\n{output}", done.action.title()));
                    }
                    Err(reason) => {
                        if done.action == Action::Archive {
                            write(self.control(ARCHIVE_LABEL), "Arşiv okunamadı; ayrıntılar günlükte.");
                        }
                        self.log(&format!("{} HATASI:\r\n\r\n{reason}", done.action.title()));
                    }
                }
                self.label(&done.status.unwrap_or_else(|e| format!("Docker baglantisi yok: {e}")));
                if done.action == Action::Status {
                    self.launch(Action::Archive);
                }
            }
        }
    }

    fn docker(args: &[OsString], compose: &Path, limit: Duration) -> Result<String, String> {
        let root = compose.parent().ok_or("COMPOSE_PARENT_MISSING")?;
        let mut child = Command::new("docker")
            .args(args).current_dir(root).creation_flags(0x0800_0000)
            .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().map_err(|e| format!("Docker Desktop bulunamadi veya baslatilamadi: {e}"))?;
        let start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if start.elapsed() >= limit => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("DOCKER_TIMEOUT: {} saniye", limit.as_secs()));
                }
                Ok(None) => thread::sleep(Duration::from_millis(100)),
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("DOCKER_PROCESS_ERROR: {e}"));
                }
            }
        }
        let result = child.wait_with_output().map_err(|e| e.to_string())?;
        let out = String::from_utf8_lossy(&result.stdout);
        let err = String::from_utf8_lossy(&result.stderr);
        let combined = format!("{out}\n{err}").trim().to_owned();
        if !result.status.success() { Err(format!("DOCKER_COMMAND_FAILED: {combined}")) } else { Ok(combined) }
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
            if response.lines().any(|s| s.trim() == "suwayomi") { Ok("Suwayomi CALISIYOR".into()) }
            else { Ok("Suwayomi DURDURULMUS".into()) }
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
            (WEB_BUTTON, "WebUI Aç"), (LOGS, "Günlükler"), (COPY, "Kataloğu Kopyala"), (OPEN, "Kataloğu Aç"),
            (ARCHIVE_OPEN, "Arşiv Klasörünü Aç"), (ARCHIVE_REFRESH, "Arşivi Yenile"),
            (OKUTOON, "OkuToon / Martial Peak")
        ] { make(hwnd, state, id, "BUTTON", caption, WS_TABSTOP, 0); }
        make(hwnd, state, URL, "EDIT", CATALOG, ES_READONLY, WS_EX_CLIENTEDGE);
        make(hwnd, state, ARCHIVE_LABEL, "STATIC", "Arşiv taranıyor...", 0, 0);
        make(hwnd, state, LOG, "EDIT", "İşlem günlükleri burada görünecek.",
            ES_READONLY | ES_MULTILINE | ES_AUTOVSCROLL | WS_VSCROLL, WS_EX_CLIENTEDGE);
    }
    unsafe fn layout(hwnd: HWND, state: &App) {
        let mut area: windows_sys::Win32::Foundation::RECT = mem::zeroed();
        GetClientRect(hwnd, &mut area);
        let width = (area.right - area.left).max(740);
        let height = (area.bottom - area.top).max(510);
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
        pos(URL, margin, 231, full, 32);
        pos(COPY, margin, 278, bw + 55, 39);
        pos(OPEN, margin + bw + 67, 278, bw, 39);
        pos(OKUTOON, margin + 2 * bw + 78, 278, bw + 20, 39);
        pos(ARCHIVE_LABEL, margin, 329, full, 26);
        pos(ARCHIVE_OPEN, margin, 360, bw + 80, 39);
        pos(ARCHIVE_REFRESH, margin + bw + 92, 360, bw, 39);
        pos(LOG, margin, 414, full, (height - 445).max(100));
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
                    WEB_BUTTON => if let Err(e) = browse(hwnd, WEB) { gui.log(&e); },
                    OPEN => if let Err(e) = browse(hwnd, CATALOG) { gui.log(&e); },
                    OKUTOON => if let Err(e) = browse(hwnd, OKUTOON_MARTIAL_PEAK) { gui.log(&e); },
                    ARCHIVE_REFRESH => gui.launch(Action::Archive),
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
                850, 620, ptr::null_mut(), ptr::null_mut(), instance, ptr::null(),
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
    fn missing_compose_fails_clearly() {
        assert!(compose_file(Path::new("Z:/not-present/BurakMangaLauncher.exe")).is_err());
    }
}
