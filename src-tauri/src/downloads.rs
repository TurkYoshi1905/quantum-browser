//! QuantumBrowser — indirme yöneticisi
//! Tauri'nin kararlı `on_download` API'sini kullanır: dosya "Downloads/QuantumBrowser"a
//! yönlendirilir, bittiğinde SQLite'a yazılır ve `downloads-changed` olayı yayınlanır.

use crate::db;
use crate::tabs::AppState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::webview::DownloadEvent;
use tauri::{AppHandle, Emitter, Manager, Webview};

static DOWNLOAD_IDS: std::sync::LazyLock<Mutex<HashMap<String, i64>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

fn download_ids() -> std::sync::MutexGuard<'static, HashMap<String, i64>> {
    DOWNLOAD_IDS.lock().unwrap()
}

/// İndirme klasörü: %USERPROFILE%\Downloads\QuantumBrowser (oluşturur).
pub fn downloads_dir() -> PathBuf {
    let base = std::env::var("USERPROFILE")
        .map(|p| PathBuf::from(p).join("Downloads").join("QuantumBrowser"))
        .unwrap_or_else(|_| PathBuf::from("QuantumBrowserDownloads"));
    std::fs::create_dir_all(&base).ok();
    base
}

/// Hedef dosya adını çakışmayacak şekilde benzersizleştir.
fn unique_path(dir: &std::path::Path, file_name: &str) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }
    let stem = candidate
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "dosya".to_string());
    let ext = candidate
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();
    for i in 1..1000 {
        let c = dir.join(format!("{stem} ({i}){ext}"));
        if !c.exists() {
            return c;
        }
    }
    candidate
}

/// Bir `on_download` işleyicisi üret (her sekme için ayrı örnek).
pub fn on_download(
    app: AppHandle,
    profile_id: i64,
) -> impl Fn(Webview, DownloadEvent<'_>) -> bool + Send + Sync + 'static {
    move |_webview, event| {
        match event {
            // İndirme isteği: hedef yolu biz belirleriz
            DownloadEvent::Requested { url, destination } => {
                let dir = downloads_dir();
                let from_url = url
                    .path_segments()
                    .and_then(|mut segments| segments.next_back().map(|s| s.to_string()))
                    .filter(|s| !s.is_empty())
                    .map(|s| percent_decode(&s));
                let file_name = from_url
                    .filter(|s| s.contains('.'))
                    .unwrap_or_else(|| "indirme".to_string());
                let target = unique_path(&dir, &sanitize_file_name(&file_name));
                *destination = target.clone();

                let state = app.state::<AppState>();
                let db = state.db.lock().unwrap();
                if let Ok(id) = db::insert_download(
                    &db,
                    profile_id,
                    url.as_str(),
                    &target
                        .file_name()
                        .map(|f| f.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "indirme".to_string()),
                    &target.to_string_lossy(),
                ) {
                    download_ids().insert(target.to_string_lossy().into_owned(), id);
                }
                let _ = app.emit("downloads-changed", serde_json::json!({}));
                true // indirmeye izin ver
            }
            // İndirme bitti (başarılı veya değil). path Option'dır.
            DownloadEvent::Finished { url, path, success } => {
                let path_str = path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let bytes = path
                    .as_ref()
                    .and_then(|p| std::fs::metadata(p).ok())
                    .map(|m| m.len() as i64)
                    .unwrap_or(0);
                let status = if success { "completed" } else { "interrupted" };
                let app_state = app.state::<AppState>();
                if let Some(id) = download_ids().remove(&path_str) {
                    if let Ok(db) = app_state.db.lock() {
                        let _ = db::update_download(&db, id, bytes, bytes, status, true);
                    }
                } else {
                    // Eşleşme yoksa url üzerinden dene (konum farklılaştırma)
                    if let Ok(db) = app_state.db.lock() {
                        let _ = db::insert_download(
                            &db,
                            profile_id,
                            url.as_str(),
                            "indirme",
                            &path_str,
                        )
                        .map(|id| {
                            let _ = db::update_download(&db, id, bytes, bytes, status, true);
                        });
                    }
                }
                let _ = app.emit("downloads-changed", serde_json::json!({}));
                true
            }
            _ => true,
        }
    }
}

/// Basit yüzde kod çözümü (dosya adındaki %20 vb.)
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            if let Ok(v) = u8::from_str_radix(hex, 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Dosya adındaki yasaklı karakterleri temizle
fn sanitize_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        "indirme".to_string()
    } else {
        trimmed
    }
}
