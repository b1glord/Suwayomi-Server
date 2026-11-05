// Dosya Yolu: REPO_ROOT/deploy/burak-manga/launcher-rust/src/webui_install.rs
// Amac: Turkce WebUI kurulumunun kaynak ve PowerShell komut sinirlarini belirler
// Modul - Rust
// Version: 0.8.0
// Aciklama: Sabit script yolu, kip allowlist'i, OS shell injection olmadan argv
// Bagimli Oldugu Katman: Service / Tool

use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const WORKFLOW_URL: &str =
    "https://github.com/b1glord/Suwayomi-Server/actions/workflows/suwayomi_webui_tr_preview.yml";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operation {
    Install,
    Restore,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScriptMode {
    PreflightInstall,
    Install,
    PreflightRestore,
    Restore,
}

impl ScriptMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreflightInstall => "PreflightInstall",
            Self::Install => "Install",
            Self::PreflightRestore => "PreflightRestore",
            Self::Restore => "Restore",
        }
    }

    fn needs_zip(self) -> bool {
        matches!(self, Self::PreflightInstall | Self::Install)
    }
}

impl Operation {
    pub fn preflight(self) -> ScriptMode {
        match self {
            Self::Install => ScriptMode::PreflightInstall,
            Self::Restore => ScriptMode::PreflightRestore,
        }
    }

    pub fn apply(self) -> ScriptMode {
        match self {
            Self::Install => ScriptMode::Install,
            Self::Restore => ScriptMode::Restore,
        }
    }
}

pub fn script_path(compose: &Path) -> Result<PathBuf, String> {
    let dir = compose.parent().ok_or("COMPOSE_PARENT_MISSING")?;
    let scripts = dir.join("scripts");
    let script = scripts.join("install-turkish-webui.ps1");
    for path in [&scripts, &script] {
        let meta = std::fs::symlink_metadata(path).map_err(|e| {
            format!("TURKISH_WEBUI_SCRIPT_MISSING: {} ({e}). Once Git Pull yapin.", path.display())
        })?;
        if meta.file_type().is_symlink() {
            return Err(format!("TURKISH_WEBUI_SCRIPT_LINK_NOT_ALLOWED: {}", path.display()));
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err(format!("TURKISH_WEBUI_SCRIPT_REPARSE_POINT: {}", path.display()));
            }
        }
    }
    if !script.is_file() || !scripts.is_dir() {
        return Err("TURKISH_WEBUI_SCRIPT_INVALID".into());
    }
    Ok(script)
}

pub fn script_arguments(
    mode: ScriptMode,
    script: &Path,
    zip: Option<&Path>,
) -> Result<Vec<OsString>, String> {
    let mut args = vec![
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
    if mode.needs_zip() {
        let chosen = zip.ok_or("TURKISH_PREVIEW_ZIP_NOT_SELECTED")?;
        args.push(OsString::from("-ZipPath"));
        args.push(chosen.as_os_str().to_owned());
    }
    Ok(args)
}

pub fn is_running(output: &str, service: &str) -> bool {
    output.lines().any(|line| line.trim() == service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_have_explicit_preflight_and_apply() {
        assert_eq!(Operation::Install.preflight(), ScriptMode::PreflightInstall);
        assert_eq!(Operation::Install.apply(), ScriptMode::Install);
        assert_eq!(Operation::Restore.preflight(), ScriptMode::PreflightRestore);
        assert_eq!(Operation::Restore.apply(), ScriptMode::Restore);
    }

    #[test]
    fn script_arguments_are_literal_and_keep_unicode_paths() {
        let script = Path::new(r"E:\Docker\Suwayomi Server\scripts\install-turkish-webui.ps1");
        let zip = Path::new(r"C:\Kullanıcı\Türkçe\Suwayomi-WebUI-TR-preview.zip");
        let args = script_arguments(ScriptMode::PreflightInstall, script, Some(zip)).unwrap();
        assert_eq!(args[0], "-NoLogo");
        assert_eq!(args[4], "RemoteSigned");
        assert_eq!(args[6], script.as_os_str());
        assert_eq!(args[8], "PreflightInstall");
        assert_eq!(args[10], zip.as_os_str());
        assert!(script_arguments(ScriptMode::Install, script, None).is_err());
        assert_eq!(script_arguments(ScriptMode::Restore, script, None).unwrap().len(), 9);
        for value in &args {
            assert_ne!(value, "Invoke-Expression");
            assert_ne!(value, "cmd.exe");
        }
    }

    #[test]
    fn running_service_match_uses_whole_lines_only() {
        assert!(is_running("suwayomi\ncf-router\n", "suwayomi"));
        assert!(!is_running("suwayomi-old\n", "suwayomi"));
        assert!(!is_running("", "suwayomi"));
    }

    #[test]
    fn missing_script_fails_without_guessing_paths() {
        let compose = Path::new("Z:/missing/compose.yaml");
        assert!(script_path(compose).is_err());
    }
}
