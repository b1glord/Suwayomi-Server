// Dosya Yolu: REPO_ROOT/deploy/burak-manga/launcher-rust/src/git_update.rs
// Amac: Windows Launcher icin korumali Git Pull hedefini ve on kosullari dogrula
// Modul - Rust
// Version: 0.7.0
// Aciklama: Dogru repo/branch/origin ve temiz calisma agaci; ff-only
// Bagimli Oldugu Katman: Service

use std::path::{Path, PathBuf};

pub const BRANCH: &str = "develop/burak-manga";
const REMOTES: &[&str] = &[
    "https://github.com/b1glord/Suwayomi-Server",
    "https://github.com/b1glord/Suwayomi-Server.git",
    "git@github.com:b1glord/Suwayomi-Server",
    "git@github.com:b1glord/Suwayomi-Server.git",
    "ssh://git@github.com/b1glord/Suwayomi-Server",
    "ssh://git@github.com/b1glord/Suwayomi-Server.git",
];

/// Compose path is always relative to the executable location, never CWD.
pub fn repository_root(compose: &Path) -> Result<PathBuf, String> {
    let root = compose
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or("GIT_REPO_ROOT_NOT_FOUND")?;
    std::fs::canonicalize(root)
        .map_err(|e| format!("GIT_REPO_ROOT_INVALID: {e}"))
}

/// No attempt to mutate Git may occur until all checks have passed.
pub fn validate(
    expected_root: &Path,
    actual_top_level: &str,
    branch: &str,
    origin: &str,
    porcelain: &str,
) -> Result<(), String> {
    let actual = std::fs::canonicalize(actual_top_level.trim())
        .map_err(|_| "Git depo kökü doğrulanamadı.".to_owned())?;
    let expected = std::fs::canonicalize(expected_root)
        .map_err(|_| "Launcher repo kökü doğrulanamadı.".to_owned())?;
    if actual != expected {
        return Err("GIT_WRONG_REPO: Launcher farklı bir Git deposunda çalışıyor.".into());
    }
    if branch.trim() != BRANCH {
        return Err(format!(
            "GIT_WRONG_BRANCH: '{}' gerekli. Mevcut branch değiştirilmeyecek.",
            BRANCH
        ));
    }
    if !REMOTES.contains(&origin.trim()) {
        return Err("GIT_WRONG_ORIGIN: Beklenen GitHub deposu değil; işlem durduruldu.".into());
    }
    if !porcelain.trim().is_empty() {
        return Err(
            "GIT_DIRTY_WORKTREE: Yerel değişiklikler veya izlenmeyen dosyalar var. \
             Önce bunları koruyup temizleyin; Launcher hiçbir şeyi silmez."
                .into(),
        );
    }
    Ok(())
}

pub fn pull_arguments() -> [&'static str; 4] {
    ["pull", "--ff-only", "origin", BRANCH]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "bm-git-tests-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("deploy/burak-manga")).unwrap();
        root
    }

    #[test]
    fn derives_repo_root_from_compose_not_process_cwd() {
        let root = sandbox();
        let compose = root.join("deploy/burak-manga/compose.yaml");
        assert_eq!(repository_root(&compose).unwrap(), root.canonicalize().unwrap());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn allows_clean_expected_repo_and_https_or_ssh_origin() {
        let root = sandbox();
        for remote in REMOTES {
            assert!(validate(&root, root.to_str().unwrap(), BRANCH, remote, "").is_ok());
        }
        assert_eq!(pull_arguments(), ["pull", "--ff-only", "origin", BRANCH]);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_wrong_repo_dirty_tree_remote_and_branch() {
        let root = sandbox();
        let other = sandbox();
        let actual = root.to_str().unwrap();
        let good = REMOTES[0];
        assert!(validate(&root, other.to_str().unwrap(), BRANCH, good, "").is_err());
        assert!(validate(&root, actual, "master", good, "").is_err());
        assert!(validate(&root, actual, "HEAD", good, "").is_err());
        assert!(validate(&root, actual, BRANCH, "https://github.com/other/repo", "").is_err());
        assert!(validate(&root, actual, BRANCH, good, " M compose.yaml").is_err());
        assert!(validate(&root, actual, BRANCH, good, "?? notes.txt").is_err());
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(other).unwrap();
    }
}
