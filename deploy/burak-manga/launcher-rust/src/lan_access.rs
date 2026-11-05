// Dosya Yolu: REPO_ROOT/deploy/burak-manga/launcher-rust/src/lan_access.rs
// Amac: Telefon erisimine ait guvenilir script ve Compose ek dosya yollarini cozer
// Modul - Rust
// Version: 0.9.0
// Aciklama: Sabit argumanlar, shell injection yok, local override sadece yonetilen format
// Bagimli Oldugu Katman: Service / Tool

use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Status,
    Enable,
    Disable,
}
impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Enable => "Enable",
            Self::Disable => "Disable",
        }
    }
}

pub fn script_path(compose: &Path) -> Result<PathBuf, String> {
    let dir = compose.parent().ok_or("COMPOSE_PARENT_MISSING")?;
    let scripts = dir.join("scripts");
    let script = scripts.join("configure-lan-access.ps1");
    for path in [&scripts, &script] {
        let meta = std::fs::symlink_metadata(path)
            .map_err(|e| format!("LAN_SCRIPT_MISSING: {} ({e})", path.display()))?;
        if meta.file_type().is_symlink() {
            return Err("LAN_SCRIPT_SYMLINK_NOT_ALLOWED".into());
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err("LAN_SCRIPT_REPARSE_POINT_NOT_ALLOWED".into());
            }
        }
    }
    if !script.is_file() || !scripts.is_dir() {
        return Err("LAN_SCRIPT_INVALID".into());
    }
    Ok(script)
}

pub fn script_arguments(
    mode: Mode,
    script: &Path,
    ip: Option<&str>,
) -> Result<Vec<OsString>, String> {
    let mut result = vec![
        OsString::from("-NoLogo"),
        OsString::from("-NoProfile"),
        OsString::from("-NonInteractive"),
        OsString::from("-ExecutionPolicy"),
        OsString::from("RemoteSigned"),
        OsString::from("-File"),
        script.as_os_str().to_owned(),
        OsString::from("-Mode"),
        OsString::from(mode.as_str()),
    ];
    if mode == Mode::Enable {
        let ip = ip.ok_or("LAN_IP_REQUIRED")?.trim();
        if ip.is_empty() || ip.len() > 45 || !ip.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return Err("LAN_IP_MUST_BE_IPV4".into());
        }
        result.push(OsString::from("-IpAddress"));
        result.push(OsString::from(ip));
    }
    Ok(result)
}

/// Append the machine-local overlay only when the full managed format matches.
/// We never accept arbitrary injected Compose content as a launcher input.
pub fn compose_args(compose: &Path) -> Vec<OsString> {
    let mut result = vec![
        OsString::from("compose"),
        OsString::from("-f"),
        compose.as_os_str().to_owned(),
    ];
    let Some(parent) = compose.parent() else {
        return result;
    };
    let overlay = parent.join("compose.lan.yaml");
    if let Ok(text) = std::fs::read_to_string(&overlay) {
        let valid = text.replace("\r\n", "\n");
        let parts: Vec<&str> = valid.trim().lines().collect();
        if parts.len() == 5
            && parts[0] == "# BURAK_MANGA_LAN_V1"
            && parts[1] == "services:"
            && parts[2] == "  suwayomi:"
            && parts[3] == "    ports:"
            && parts[4].starts_with("      - \"")
            && parts[4].ends_with(":4567:4567\"")
        {
            let addr = &parts[4][9..parts[4].len() - 11];
            if addr.parse::<std::net::Ipv4Addr>().is_ok_and(|ip| ip.is_private()) {
                result.push(OsString::from("-f"));
                result.push(overlay.into_os_string());
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_args_never_accept_shell_characters() {
        let script = Path::new(r"E:\Manga Server\scripts\configure-lan-access.ps1");
        let args = script_arguments(Mode::Enable, script, Some("192.168.1.10")).unwrap();
        assert_eq!(args[8], "Enable");
        assert_eq!(args[10], "192.168.1.10");
        assert!(script_arguments(Mode::Enable, script, Some("192.168.1.10;rm")).is_err());
        assert!(script_arguments(Mode::Enable, script, None).is_err());
        assert_eq!(script_arguments(Mode::Disable, script, None).unwrap().len(), 9);
    }

    #[test]
    fn compose_uses_only_managed_private_override() {
        let root = std::env::temp_dir().join(format!("bm-lan-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let compose = root.join("compose.yaml");
        let overlay = root.join("compose.lan.yaml");
        assert_eq!(compose_args(&compose).len(), 3);
        std::fs::write(
            &overlay,
            "# BURAK_MANGA_LAN_V1\nservices:\n  suwayomi:\n    ports:\n      - \"192.168.1.10:4567:4567\"\n",
        ).unwrap();
        let args = compose_args(&compose);
        assert_eq!(args.len(), 5);
        assert_eq!(args[3], "-f");
        std::fs::write(&overlay, "services:\n  suwayomi:\n    ports:\n      - \"0.0.0.0:4567:4567\"\n").unwrap();
        assert_eq!(compose_args(&compose).len(), 3);
        std::fs::write(
            &overlay,
            "# BURAK_MANGA_LAN_V1\nservices:\n  suwayomi:\n    ports:\n      - \"8.8.8.8:4567:4567\"\n",
        ).unwrap();
        assert_eq!(compose_args(&compose).len(), 3);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
