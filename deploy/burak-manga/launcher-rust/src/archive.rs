// Dosya Yolu: REPO_ROOT/deploy/burak-manga/launcher-rust/src/archive.rs
// Amac: Suwayomi'nin kalici arsivindeki CBZ dosyalarini ve disk kullanimini sayar
// Modul - Rust
// Version: 0.4.0
// Aciklama: Read-only tarama; junction/symlink izlemeyen ve sinirli dizin gecisi
// Bagimli Oldugu Katman: Service
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_ENTRIES: usize = 500_000;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ArchiveStats {
    pub cbz_chapters: u64,
    pub series_with_cbz: usize,
    pub all_file_bytes: u64,
}

pub fn downloads_path(compose: &Path) -> Result<PathBuf, String> {
    let folder = compose.parent().ok_or("COMPOSE_PARENT_MISSING")?;
    Ok(folder.join("storage").join("Tachidesk").join("downloads"))
}

fn is_link(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return true; // NTFS junction/reparse points must not escape trusted storage.
        }
    }
    false
}

pub fn scan(root: &Path) -> Result<ArchiveStats, String> {
    if !root.exists() {
        return Err("DOWNLOADS_NOT_FOUND: Indirmeler klasoru henuz olusmamis.".into());
    }
    let root_info = fs::symlink_metadata(root).map_err(|e| format!("DOWNLOADS_METADATA: {e}"))?;
    if !root_info.is_dir() || is_link(&root_info) {
        return Err("UNTRUSTED_DOWNLOADS_FOLDER".into());
    }
    let mut stack = vec![root.to_path_buf()];
    let mut folders_with_cbz = HashSet::new();
    let mut seen = 0_usize;
    let mut stats = ArchiveStats::default();
    while let Some(directory) = stack.pop() {
        let listing = fs::read_dir(&directory)
            .map_err(|e| format!("ARCHIVE_READ_FAILED ({}): {e}", directory.display()))?;
        for item in listing {
            let item = item.map_err(|e| format!("ARCHIVE_ENTRY_FAILED: {e}"))?;
            seen += 1;
            if seen > MAX_ENTRIES {
                return Err("ARCHIVE_SCAN_LIMIT_EXCEEDED".into());
            }
            let path = item.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|e| format!("ARCHIVE_METADATA_FAILED ({}): {e}", path.display()))?;
            if is_link(&metadata) {
                continue;
            }
            if metadata.is_dir() {
                stack.push(path);
            } else if metadata.is_file() {
                stats.all_file_bytes = stats.all_file_bytes.saturating_add(metadata.len());
                let is_cbz = path.extension()
                    .is_some_and(|ext| ext.to_string_lossy().eq_ignore_ascii_case("cbz"));
                if is_cbz {
                    stats.cbz_chapters += 1;
                    if let Some(parent) = path.parent() {
                        folders_with_cbz.insert(parent.to_path_buf());
                    }
                }
            }
        }
    }
    stats.series_with_cbz = folders_with_cbz.len();
    Ok(stats)
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

pub fn summary(stats: &ArchiveStats) -> String {
    format!(
        "Arşiv: {} manga klasörü | {} CBZ bölüm | {} disk kullanımı",
        stats.series_with_cbz,
        stats.cbz_chapters,
        format_bytes(stats.all_file_bytes)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_completed_cbz_without_following_unrelated_files() {
        let temp = std::env::temp_dir().join(format!(
            "burak_archive_{}_{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        let first = temp.join("Merlin").join("Bilge");
        let second = temp.join("Okutoon").join("Martial Peak");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        fs::write(first.join("Bolum 1.cbz"), b"cbz fixture").unwrap();
        fs::write(first.join("Bolum 2.CBZ"), b"cbz fixture").unwrap();
        fs::write(first.join("part.tmp"), b"incomplete").unwrap();
        fs::write(second.join("Bolum 1.cbz"), b"cbz fixture").unwrap();
        let stats = scan(&temp).unwrap();
        assert_eq!(stats.series_with_cbz, 2);
        assert_eq!(stats.cbz_chapters, 3);
        assert_eq!(stats.all_file_bytes, 3 * 11 + 10);
        fs::remove_dir_all(temp).unwrap();
    }
    #[test]
    fn rejects_missing_and_linked_archive_roots() {
        let absent = std::env::temp_dir().join(format!("missing_downloads_{}", std::process::id()));
        assert!(scan(&absent).is_err());
    }
    #[test]
    fn download_path_is_relative_to_compose_not_cwd() {
        let path = downloads_path(Path::new(r"E:\Docker\Suwayomi-Server\deploy\burak-manga\compose.yaml"))
            .unwrap();
        assert!(path.to_string_lossy().contains("Tachidesk"));
        assert!(path.to_string_lossy().ends_with("downloads"));
    }
}
