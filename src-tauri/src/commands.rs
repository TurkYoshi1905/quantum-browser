//! QuantumBrowser â€” frontend â†” Rust IPC komutlarÄ±

use crate::db;
use crate::tabs::{self, AppState, Snapshot};
use crate::vault;
use tauri::{AppHandle, Emitter, Manager, State};

// ---------- AnlÄ±k gÃ¶rÃ¼ntÃ¼ ----------

#[tauri::command]
pub fn get_snapshot(app: AppHandle) -> Snapshot {
    tabs::snapshot(&app)
}

// ---------- Sekmeler ----------

#[tauri::command]
pub async fn new_tab(
    app: AppHandle,
    url: Option<String>,
    incognito: Option<bool>,
) -> Result<(), String> {
    tabs::create_tab(&app, url, incognito.unwrap_or(false)).map(|_| ())
}

#[tauri::command]
pub async fn close_tab(app: AppHandle, id: String) -> Result<(), String> {
    tabs::close_tab(&app, &id)
}

#[tauri::command]
pub fn activate_tab(app: AppHandle, id: String) -> Result<(), String> {
    tabs::activate_tab(&app, &id)
}

#[tauri::command]
pub async fn duplicate_tab(app: AppHandle, id: String) -> Result<(), String> {
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
pub async fn close_others(app: AppHandle, id: String) -> Result<(), String> {
    tabs::close_others(&app, &id)
}

#[tauri::command]
pub async fn navigate(app: AppHandle, url: String) -> Result<(), String> {
    tabs::navigate_active(&app, &url)
}

#[tauri::command]
pub async fn go_back(app: AppHandle) {
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
pub async fn go_forward(app: AppHandle) {
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
pub async fn reload(app: AppHandle) {
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
pub async fn new_incognito_tab(app: AppHandle) -> Result<(), String> {
    tabs::create_tab(&app, None, true).map(|_| ())
}

/// Son kapatÄ±lan sekmeyi yeniden aÃ§ar (Ctrl+Shift+T).
#[tauri::command]
pub async fn reopen_closed_tab(app: AppHandle) -> Result<(), String> {
    let entry = {
        let state = app.state::<AppState>();
        let mut store = state.tabs.lock().unwrap();
        store.last_closed.pop()
    };
    let Some(entry) = entry else {
        return Ok(());
    };
    if let Some(kind) = entry.kind {
        let _ = tabs::create_tab(&app, Some(format!("quantum://{kind}")), false)?;
    } else if !entry.url.is_empty() {
        let label = tabs::create_tab(&app, Some(entry.url), entry.incognito)?;
        if entry.pinned {
            let _ = tabs::toggle_pin(&app, &label);
        }
    }
    Ok(())
}

/// Popover penceresinin tembel (lazy) yaratÄ±mÄ± + konumlandÄ±rma + gÃ¶sterme.
/// Startup'ta yaratmak yerine ilk menÃ¼ aÃ§Ä±lÄ±ÅŸÄ±nda yaratÄ±lÄ±r â€” bu, uygulamanÄ±n
/// aÃ§Ä±lÄ±r aÃ§Ä±lmaz Ã§Ã¶kmesini Ã¶nler.
fn ensure_popover(app: &AppHandle) -> Option<tauri::webview::WebviewWindow<tauri::Wry>> {
    if let Some(win) = app.get_webview_window("popover") {
        return Some(win);
    }
    let win = tauri::webview::WebviewWindowBuilder::new(
        app,
        "popover",
        tauri::WebviewUrl::App("index.html#/popover".into()),
    )
    .title("QuantumBrowser Menu")
    .decorations(false)
    .inner_size(248.0, 420.0)
    .always_on_top(true)
    .transparent(true)
    .resizable(false)
    .visible(false)
    .skip_taskbar(true)
    .focused(false)
    .build()
    .ok()?;
    Some(win)
}

/// Popover penceresini (dekorsuz, ÅŸeffaf, en Ã¼stte) konumlandÄ±rÄ±p gÃ¶sterir.
/// Kabuktan Ã§aÄŸrÄ±lÄ±r; popover sayfasÄ± "menu-open" olayÄ± ile menÃ¼ tipini okur.
#[tauri::command]
pub fn open_menu_popup(
    app: AppHandle,
    menu: String,
    x: f64,
    y: f64,
    tab_id: Option<String>,
) -> Result<(), String> {
    let (w, h) = match menu.as_str() {
        "main" => (260.0_f64, 340.0_f64),
        "tab" => (260.0_f64, 440.0_f64),
        _ => (260.0_f64, 340.0_f64),
    };
    tabs::dbg_log(&format!(
        "open_menu_popup: menu={menu} x={x} y={y} w={w} h={h}"
    ));
    let Some(win) = ensure_popover(&app) else {
        tabs::dbg_log("open_menu_popup: popover penceresi yaratılamadı");
        return Err("popover penceresi yaratÄ±lamadÄ±".to_string());
    };
    let _ = win.set_size(tauri::LogicalSize::<f64>::new(w, h));
    let _ = win.set_position(tauri::LogicalPosition::<f64>::new(x, y));
    let _ = win.show();
    let _ = win.set_focus();
    let _ = app.emit(
        "menu-open",
        serde_json::json!({ "menu": menu, "tabId": tab_id }),
    );
    Ok(())
}

/// Popover penceresini gizler (kapanma / Escape / dÄ±ÅŸarÄ± tÄ±klama / Ã¶ÄŸe seÃ§imi).
#[tauri::command]
pub fn close_menu_popup(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("popover") {
        let _ = win.hide();
    }
    Ok(())
}

// ---------- Sekme gruplarÄ± ----------

#[tauri::command]
pub fn create_group(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<i64, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
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
        Err("sekme bulunamadÄ±".to_string())
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
pub fn set_theme(app: AppHandle, state: State<'_, AppState>, theme: String) -> Result<(), String> {
    {
        let db = state.db.lock().unwrap();
        db::set_setting(&db, "theme", &theme).map_err(|e| e.to_string())?;
    }
    tabs::emit_tabs_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn set_accent(
    app: AppHandle,
    state: State<'_, AppState>,
    accent: String,
) -> Result<(), String> {
    {
        let db = state.db.lock().unwrap();
        db::set_setting(&db, "accent", &accent).map_err(|e| e.to_string())?;
    }
    tabs::emit_tabs_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn set_block_insecure(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    {
        let db = state.db.lock().unwrap();
        db::set_setting(&db, "block_insecure", if enabled { "1" } else { "0" })
            .map_err(|e| e.to_string())?;
    }
    tabs::emit_tabs_changed(&app);
    Ok(())
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

#[tauri::command]
pub fn set_session_restore(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db::set_setting(&db, "session_restore", if enabled { "1" } else { "0" })
        .map_err(|e| e.to_string())
}

// ---------- Yer imleri ----------

#[tauri::command]
pub fn list_bookmarks(
    state: State<'_, AppState>,
    query: Option<String>,
) -> Result<Vec<db::Bookmark>, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
    db::list_bookmarks(&db, profile_id, query.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_bookmark(
    state: State<'_, AppState>,
    url: String,
    title: String,
) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
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
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
    db::is_bookmarked(&db, profile_id, &url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_bookmark(state: State<'_, AppState>, url: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
    db::remove_bookmark(&db, profile_id, &url).map_err(|e| e.to_string())
}

// ---------- GeÃ§miÅŸ ----------

#[tauri::command]
pub fn list_history(
    state: State<'_, AppState>,
    query: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<db::HistoryEntry>, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
    db::list_history(&db, profile_id, query.as_deref(), limit.unwrap_or(200))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
    db::clear_history(&db, profile_id).map_err(|e| e.to_string())
}

// ---------- Ä°ndirmeler ----------

#[tauri::command]
pub fn list_downloads(state: State<'_, AppState>) -> Result<Vec<db::DownloadEntry>, String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
    db::list_downloads(&db, profile_id).map_err(|e| e.to_string())
}

// ---------- GeÃ§miÅŸ silme (seÃ§ili) ----------

#[tauri::command]
pub fn delete_history_entries(state: State<'_, AppState>, ids: Vec<i64>) -> Result<usize, String> {
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
    let db = state.db.lock().unwrap();
    db::delete_history_entries(&db, profile_id, &ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_finished_downloads(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
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
            .map_err(|e| format!("klasÃ¶r aÃ§Ä±lamadÄ±: {e}"))?;
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
pub fn add_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<i64, String> {
    let id = {
        let db = state.db.lock().unwrap();
        let clean = name.trim();
        if clean.is_empty() {
            return Err("profil adÄ± boÅŸ olamaz".to_string());
        }
        db::add_profile(&db, clean, &color).map_err(|e| e.to_string())?
    };
    // Yeni profille boÅŸ arayÃ¼z kalmasÄ±n â€” NewTab aÃ§
    let _ = tabs::create_tab(&app, None, false);
    Ok(id)
}

#[tauri::command]
pub fn rename_profile(
    state: State<'_, AppState>,
    profile_id: i64,
    name: String,
) -> Result<(), String> {
    let clean = name.trim();
    if clean.is_empty() {
        return Err("profil adÄ± boÅŸ olamaz".to_string());
    }
    let db = state.db.lock().unwrap();
    let found = db::profile_by_id(&db, profile_id).map_err(|e| e.to_string())?;
    if found.is_none() {
        return Err("profil bulunamadÄ±".to_string());
    }
    db::rename_profile(&db, profile_id, clean).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_profile(state: State<'_, AppState>, profile_id: i64) -> Result<(), String> {
    let current = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
    if profile_id == current {
        return Err("Aktif profili silemezsin â€” Ã¶nce baÅŸka bir profile geÃ§.".to_string());
    }
    let db = state.db.lock().unwrap();
    let found = db::profile_by_id(&db, profile_id)
        .map_err(|e| e.to_string())?
        .ok_or("profil bulunamadÄ±")?;
    if found.is_default {
        return Err("VarsayÄ±lan profil silinemez.".to_string());
    }
    db::delete_profile(&db, profile_id).map_err(|e| e.to_string())
}

/// Profili deÄŸiÅŸtirir: mevcut sekmeler kapanÄ±r ve yeni oturum baÅŸlar.
/// (Ã‡oklu pencere desteÄŸi v0.2'ye ertelendi â€” kararlar.md'ye not dÃ¼ÅŸÃ¼ldÃ¼.)
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
            .ok_or("profil bulunamadÄ±")?;
        let mut store = state.tabs.lock().unwrap();
        store.profile_id = Some(profile.id);
        store.profile_store_id = Some(profile.store_id);
    }
    // BoÅŸ arayÃ¼z kalmasÄ±n â€” yeni oturum iÃ§in NewTab aÃ§
    let _ = tabs::create_tab(&app, None, false);
    Ok(())
}

// ---------- GeliÅŸtirici paneli ----------

#[tauri::command]
pub fn toggle_devtools(app: AppHandle) -> Result<(), String> {
    let label = tabs::active_label(&app).ok_or("aktif sekme yok")?;
    let Some(wv) = app.get_webview(&label) else {
        return Err("aktif webview bulunamadÄ±".to_string());
    };
    if wv.is_devtools_open() {
        let _ = wv.close_devtools();
    } else {
        let _ = wv.open_devtools();
    }
    Ok(())
}

// ---------- Åifre KasasÄ± (quantum://vault) ----------

#[tauri::command]
pub fn vault_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let conn = state.vault_conn.lock().unwrap();
    let s = vault::status(&conn).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "hasMasterPassword": s.has_master_password,
        "unlocked": vault::unlocked(state.inner()),
    }))
}

#[tauri::command]
pub fn vault_set_master_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<(), String> {
    let conn = state.vault_conn.lock().unwrap();
    vault::set_master_password(&conn, &password)
}

#[tauri::command]
pub fn vault_unlock(state: State<'_, AppState>, password: String) -> Result<bool, String> {
    let conn = state.vault_conn.lock().unwrap();
    vault::unlock(&conn, state.inner(), &password)
}

#[tauri::command]
pub fn vault_lock(state: State<'_, AppState>) {
    vault::lock(state.inner());
}

#[tauri::command]
pub fn vault_list(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let conn = state.vault_conn.lock().unwrap();
    let items = vault::list(&conn, state.inner())?;
    Ok(items
        .into_iter()
        .map(|i| {
            serde_json::json!({
                "id": i.id,
                "host": i.host,
                "username": i.username,
                "created_at": i.created_at,
                "updated_at": i.updated_at,
            })
        })
        .collect())
}

#[tauri::command]
pub fn vault_find_for_host(
    state: State<'_, AppState>,
    host: String,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = state.vault_conn.lock().unwrap();
    let items = vault::find_for_host(&conn, state.inner(), &host)?;
    Ok(items
        .into_iter()
        .map(|i| serde_json::json!({ "id": i.id, "host": i.host, "username": i.username }))
        .collect())
}

#[tauri::command]
pub fn vault_add(
    state: State<'_, AppState>,
    host: String,
    username: String,
    password: String,
    note: Option<String>,
) -> Result<(), String> {
    let conn = state.vault_conn.lock().unwrap();
    vault::add(
        &conn,
        state.inner(),
        &host,
        &username,
        &password,
        note.as_deref().unwrap_or(""),
    )
}

#[tauri::command]
pub fn vault_update(
    state: State<'_, AppState>,
    id: i64,
    host: String,
    username: String,
    password: String,
    note: Option<String>,
) -> Result<(), String> {
    let conn = state.vault_conn.lock().unwrap();
    vault::update(
        &conn,
        state.inner(),
        id,
        &host,
        &username,
        &password,
        note.as_deref().unwrap_or(""),
    )
}

#[tauri::command]
pub fn vault_get_password(state: State<'_, AppState>, id: i64) -> Result<String, String> {
    let conn = state.vault_conn.lock().unwrap();
    vault::get_password(&conn, state.inner(), id)
}

#[tauri::command]
pub fn vault_delete(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.vault_conn.lock().unwrap();
    vault::delete(&conn, state.inner(), id)
}

#[tauri::command]
pub fn vault_generate_password() -> Result<String, String> {
    Ok(vault::generate_password(18))
}

#[tauri::command]
pub fn vault_export(state: State<'_, AppState>) -> Result<String, String> {
    let conn = state.vault_conn.lock().unwrap();
    vault::export(&conn, state.inner())
}

#[tauri::command]
pub fn vault_import(state: State<'_, AppState>, contents: String) -> Result<usize, String> {
    let conn = state.vault_conn.lock().unwrap();
    vault::import(&conn, state.inner(), &contents)
}

// ---------- Veri temizleme (gizlilik) ----------

#[tauri::command]
pub fn clear_browsing_data(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadÄ±")?;
    db::clear_history(&db, profile_id).map_err(|e| e.to_string())?;
    // Yer imlerini koru; yalnÄ±zca geÃ§miÅŸ + indirme listesi temizlenir.
    db::clear_finished_downloads(&db, profile_id).map_err(|e| e.to_string())?;
    Ok(())
}
