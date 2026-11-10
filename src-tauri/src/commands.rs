//! QuantumBrowser — frontend ↔ Rust IPC komutları

use crate::db;
use crate::tabs::{self, AppState, Snapshot};
use tauri::{AppHandle, Manager, State};

// ---------- Anlık görüntü ----------

#[tauri::command]
pub fn get_snapshot(app: AppHandle) -> Snapshot {
    tabs::snapshot(&app)
}

// ---------- Sekmeler ----------

#[tauri::command]
pub fn new_tab(app: AppHandle, url: Option<String>, incognito: Option<bool>) -> Result<(), String> {
    tabs::create_tab(&app, url, incognito.unwrap_or(false)).map(|_| ())
}

#[tauri::command]
pub fn close_tab(app: AppHandle, id: String) -> Result<(), String> {
    tabs::close_tab(&app, &id)
}

#[tauri::command]
pub fn activate_tab(app: AppHandle, id: String) -> Result<(), String> {
    tabs::activate_tab(&app, &id)
}

#[tauri::command]
pub fn duplicate_tab(app: AppHandle, id: String) -> Result<(), String> {
    tabs::duplicate_tab(&app, &id)
}

#[tauri::command]
pub fn move_tab(app: AppHandle, from_id: String, to_id: String) -> Result<(), String> {
    tabs::move_tab(&app, &from_id, &to_id)
}

#[tauri::command]
pub fn toggle_pin(app: AppHandle, id: String) -> Result<(), String> {
    tabs::toggle_pin(&app, &id)
}

#[tauri::command]
pub fn close_others(app: AppHandle, id: String) -> Result<(), String> {
    tabs::close_others(&app, &id)
}

#[tauri::command]
pub fn navigate(app: AppHandle, url: String) -> Result<(), String> {
    tabs::navigate_active(&app, &url)
}

#[tauri::command]
pub fn go_back(app: AppHandle) {
    #[cfg(windows)]
    {
        if let Some(label) = tabs::active_label(&app) {
            if let Some(wv) = app.get_webview(&label) {
                let _ = wv.with_webview(|platform| unsafe {
                    let _ = platform.controller().CoreWebView2().map(|core| {
                        let _ = core.GoBack();
                    });
                });
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = app;
    }
}

#[tauri::command]
pub fn go_forward(app: AppHandle) {
    #[cfg(windows)]
    {
        if let Some(label) = tabs::active_label(&app) {
            if let Some(wv) = app.get_webview(&label) {
                let _ = wv.with_webview(|platform| unsafe {
                    let _ = platform.controller().CoreWebView2().map(|core| {
                        let _ = core.GoForward();
                    });
                });
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = app;
    }
}

#[tauri::command]
pub fn reload(app: AppHandle) {
    #[cfg(windows)]
    {
        if let Some(label) = tabs::active_label(&app) {
            if let Some(wv) = app.get_webview(&label) {
                let _ = wv.with_webview(|platform| unsafe {
                    let _ = platform.controller().CoreWebView2().map(|core| {
                        let _ = core.Reload();
                    });
                });
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = app;
    }
}

#[tauri::command]
pub fn new_incognito_tab(app: AppHandle) -> Result<(), String> {
    tabs::create_tab(&app, None, true).map(|_| ())
}

// ---------- Sekme grupları ----------

#[tauri::command]
pub fn create_group(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<i64, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    db::create_group(&db, profile_id, &name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_group(state: State<'_, AppState>, group_id: i64) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db::toggle_group_collapsed(&db, group_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_group(state: State<'_, AppState>, group_id: i64) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db::delete_group(&db, group_id).map_err(|e| e.to_string())?;
    // Sekmelerde grubu temizle
    let mut store = state.tabs.lock().unwrap();
    for tab in store.tabs.iter_mut() {
        if tab.group_id == Some(group_id) {
            tab.group_id = None;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn assign_group(
    state: State<'_, AppState>,
    tab_id: String,
    group_id: Option<i64>,
) -> Result<(), String> {
    let mut store = state.tabs.lock().unwrap();
    if let Some(tab) = store.tabs.iter_mut().find(|t| t.id == tab_id) {
        tab.group_id = group_id;
        Ok(())
    } else {
        Err("sekme bulunamadı".to_string())
    }
}

// ---------- Kabuk durumu ----------

#[tauri::command]
pub fn set_overlay_open(app: AppHandle, open: bool) {
    tabs::set_overlay_open(&app, open);
}

#[tauri::command]
pub fn set_sidebar_collapsed(app: AppHandle, collapsed: bool) {
    tabs::set_sidebar_collapsed(&app, collapsed);
}

// ---------- Tema & ayarlar ----------

#[tauri::command]
pub fn set_theme(state: State<'_, AppState>, theme: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db::set_setting(&db, "theme", &theme).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_search_engine(state: State<'_, AppState>, engine: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db::set_setting(&db, "search_engine", &engine).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_blocker_enabled(app: AppHandle, enabled: bool) {
    tabs::set_blocker_enabled(&app, enabled);
}

// ---------- Yer imleri ----------

#[tauri::command]
pub fn list_bookmarks(
    state: State<'_, AppState>,
    query: Option<String>,
) -> Result<Vec<db::Bookmark>, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    db::list_bookmarks(&db, profile_id, query.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_bookmark(
    state: State<'_, AppState>,
    url: String,
    title: String,
) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    if db::is_bookmarked(&db, profile_id, &url).map_err(|e| e.to_string())? {
        db::remove_bookmark(&db, profile_id, &url).map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        db::add_bookmark(&db, profile_id, &url, &title).map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub fn is_bookmarked(state: State<'_, AppState>, url: String) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    db::is_bookmarked(&db, profile_id, &url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_bookmark(state: State<'_, AppState>, url: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    db::remove_bookmark(&db, profile_id, &url).map_err(|e| e.to_string())
}

// ---------- Geçmiş ----------

#[tauri::command]
pub fn list_history(
    state: State<'_, AppState>,
    query: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<db::HistoryEntry>, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    db::list_history(&db, profile_id, query.as_deref(), limit.unwrap_or(200))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    db::clear_history(&db, profile_id).map_err(|e| e.to_string())
}

// ---------- İndirmeler ----------

#[tauri::command]
pub fn list_downloads(state: State<'_, AppState>) -> Result<Vec<db::DownloadEntry>, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    db::list_downloads(&db, profile_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_finished_downloads(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    db::clear_finished_downloads(&db, profile_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_downloads_folder() -> Result<(), String> {
    let dir = crate::downloads::downloads_dir();
    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| format!("klasör açılamadı: {e}"))?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}

// ---------- Profiller ----------

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> Result<Vec<db::Profile>, String> {
    let db = state.db.lock().unwrap();
    db::list_profiles(&db).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_profile(state: State<'_, AppState>, name: String, color: String) -> Result<i64, String> {
    let db = state.db.lock().unwrap();
    let clean = name.trim();
    if clean.is_empty() {
        return Err("profil adı boş olamaz".to_string());
    }
    db::add_profile(&db, clean, &color).map_err(|e| e.to_string())
}

/// Profili değiştirir: mevcut sekmeler kapanır ve yeni oturum başlar.
/// (Çoklu pencere desteği v0.2'ye ertelendi — kararlar.md'ye not düşüldü.)
#[tauri::command]
pub fn switch_profile(app: AppHandle, profile_id: i64) -> Result<(), String> {
    let labels: Vec<String> = {
        let state = app.state::<AppState>();
        let store = state.tabs.lock().unwrap();
        store.tabs.iter().map(|t| t.id.clone()).collect()
    };
    for label in labels {
        let _ = tabs::close_tab(&app, &label);
    }
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().unwrap();
        let profile = db::profile_by_id(&db, profile_id)
            .map_err(|e| e.to_string())?
            .ok_or("profil bulunamadı")?;
        let mut store = state.tabs.lock().unwrap();
        store.profile_id = Some(profile.id);
        store.profile_store_id = Some(profile.store_id);
    }
    Ok(())
}

// ---------- Veri temizleme (gizlilik) ----------

#[tauri::command]
pub fn clear_browsing_data(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    db::clear_history(&db, profile_id).map_err(|e| e.to_string())?;
    // Yer imlerini koru; yalnızca geçmiş + indirme listesi temizlenir.
    db::clear_finished_downloads(&db, profile_id).map_err(|e| e.to_string())?;
    Ok(())
}
