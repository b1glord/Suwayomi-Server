// Dosya Yolu: REPO_ROOT/deploy/burak-manga/launcher-rust/src/diagnostics.rs
// Amac: Son Docker gunlugunde Cloudflare ve indirme kuyrugu arizalarini tani
// Modul - Rust
// Version: 0.5.0
// Aciklama: Salt-okunur gunluk ozetleme, herhangi bir CAPTCHA bypass yapmaz
// Bagimli Oldugu Katman: Service
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DownloadHealth {
    pub cloudflare_error_lines: usize,
    pub connection_leak_lines: usize,
    pub running: Option<u32>,
    pub queued: Option<u32>,
    pub failed: Option<u32>,
}

fn number_after(line: &str, token: &str) -> Option<u32> {
    let rest = line.split_once(token)?.1;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

pub fn parse(logs: &str) -> DownloadHealth {
    let mut health = DownloadHealth::default();
    for line in logs.lines() {
        if line.contains("Cloudflare bypass currently disabled") {
            health.cloudflare_error_lines += 1;
        }
        if line.contains("A connection to ") && line.contains(" was leaked") {
            health.connection_leak_lines += 1;
        }
        if let (Some(running), Some(queued), Some(failed)) = (
            number_after(line, "Running: "),
            number_after(line, "Queued: "),
            number_after(line, "Failed: "),
        ) {
            health.running = Some(running);
            health.queued = Some(queued);
            health.failed = Some(failed);
        }
    }
    health
}

/// The opt-in safeguard must never guess queue activity from missing/stale counters.
/// Error-line counts are *not* unique failed chapter counts.
pub fn should_pause(health: &DownloadHealth) -> bool {
    health.cloudflare_error_lines >= 3 && health.running.is_some_and(|n| n > 0)
}

pub fn summary(health: &DownloadHealth) -> String {
    let mut fragments = vec![];
    match (health.running, health.queued, health.failed) {
        (Some(running), Some(queued), Some(failed)) => {
            fragments.push(format!(
                "İndirme: {} çalışan | {} bekleyen | {} başarısız",
                running, queued, failed,
            ));
        }
        _ => fragments.push("Kuyruk sayıları: son günlükte bulunamadı".into()),
    }
    if health.cloudflare_error_lines > 0 {
        fragments.push(format!(
            "Cloudflare erişim engeli: {} hata satırı (bölüm sayısı değil)",
            health.cloudflare_error_lines
        ));
    }
    if health.connection_leak_lines > 0 {
        fragments.push(format!(
            "Bağlantı sızıntısı uyarısı: {} satır",
            health.connection_leak_lines
        ));
    }
    if health.cloudflare_error_lines == 0 && health.connection_leak_lines == 0 {
        fragments.push("Son günlüklerde Cloudflare uyarısı görünmedi".into());
    }
    format!("Son 10 dakika: {}", fragments.join(" | "))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_realistic_suwayomi_log_without_counting_stack_frames_as_chapters() {
        let txt = "suwayomi-1 | java.io.IOException: Cloudflare bypass currently disabled\n\
                   suwayomi-1 | Caused by: java.io.IOException: Cloudflare bypass currently disabled\n\
                   suwayomi-1 | WARNING: A connection to https://example.com/ was leaked\n\
                   suwayomi-1 | DownloadManager -- Running: 0, Queued: 0, Failed: 159";
        let health = parse(txt);
        assert_eq!(health.cloudflare_error_lines, 2);
        assert_eq!(health.connection_leak_lines, 1);
        assert_eq!((health.running, health.queued, health.failed), (Some(0), Some(0), Some(159)));
        assert!(!should_pause(&health));
        assert!(summary(&health).contains("159 başarısız"));
    }
    #[test]
    fn opt_in_pause_only_when_recent_errors_and_active_downloads() {
        let log = "Cloudflare bypass currently disabled\n".repeat(3)
            + "DownloadManager -- Running: 1, Queued: 12, Failed: 5\n";
        assert!(should_pause(&parse(&log)));
        let no_status = "Cloudflare bypass currently disabled\n".repeat(25);
        assert!(!should_pause(&parse(&no_status)));
        let last_status_stopped = log + "DownloadManager -- Running: 0, Queued: 0, Failed: 12\n";
        assert!(!should_pause(&parse(&last_status_stopped)));
    }
    #[test]
    fn no_errors_does_not_trigger_safeguard() {
        assert!(!should_pause(&parse("Running: 1, Queued: 8, Failed: 0")));
        assert_eq!(parse("unrelated status"), DownloadHealth::default());
    }
}
