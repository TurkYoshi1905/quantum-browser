//! QuantumBrowser — frontend ↔ Rust IPC komutları

use crate::db;
use crate::tabs::{self, AppState, Snapshot};
use crate::vault;
use tauri::{AppHandle, Emitter, Manager, State};

// ---------- Anlık görüntü ----------

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

/// Son kapatılan sekmeyi yeniden açar (Ctrl+Shift+T).
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

/// Popover penceresinin tembel (lazy) yaratımı + konumlandırma + gösterme.
/// Startup'ta yaratmak yerine ilk menü açılışında yaratılır — bu, uygulamanın
/// açılır açılmaz çökmesini önler.
/// ⋮ Menüsünü kabuk penceresi içindeki bir çocuk webview olarak gösterir.
/// Eski "top-level transparent popover" yaklaşımı Windows'ta görünmez pencere +
/// odak çalma + tıklama yutma sorunlarına yol açmıştı; çocuk webview sayfalarla
/// aynı z-order motorunu ve aynı girdi yolunu paylaşır.
#[tauri::command]
pub fn open_menu_popup(
    app: AppHandle,
    menu: String,
    x: f64,
    y: f64,
    tab_id: Option<String>,
) -> Result<(), String> {
    tabs::dbg_log(&format!("open_menu_popup: menu={menu} x={x} y={y}"));
    tabs::show_menu(&app, &menu, x, y)?;
    {
        let state = app.state::<AppState>();
        let store = state.tabs.lock().unwrap();
        store.menu_pending = Some((menu.clone(), tab_id.clone()));
        drop(store);
    }
    let _ = app.emit(
        "menu-open",
        serde_json::json!({ "menu": menu, "tabId": tab_id }),
    );
    Ok(())
}

/// Bekleyen menü durumunu getir — çocuk webview'um ilk yüklenme yarışını ortadan kaldır.
#[tauri::command]
pub fn get_pending_menu(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let store = state.tabs.lock().unwrap();
    Ok(match &store.menu_pending {
        Some((menu, tab_id)) => serde_json::json!({
            "menu": menu,
            "tabId": tab_id,
        }),
        None => serde_json::Value::Null,
    })
}

/// Menüyü gizler (kapanma / Escape / dışarı tıklama / öğe seçimi).
#[tauri::command]
pub fn close_menu_popup(app: AppHandle) -> Result<(), String> {
    tabs::hide_menu(&app);
    Ok(())
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

/// İndirme tamamlandı baloncuğunu çocuk webview ile gösterir (sayfa yok edilmez).
#[tauri::command]
pub fn show_toast(app: AppHandle, text: String) -> Result<(), String> {
    tabs::show_toast(&app, &text)
}

#[tauri::command]
pub fn close_toast(app: AppHandle) {
    tabs::hide_toast(&app);
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

#[tauri::command]
pub fn set_auto_update(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    {
        let db = state.db.lock().unwrap();
        db::set_setting(&db, "auto_update", if enabled { "1" } else { "0" })
            .map_err(|e| e.to_string())?;
    }
    tabs::emit_tabs_changed(&app);
    Ok(())
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

// ---------- Geçmiş silme (seçili) ----------

#[tauri::command]
pub fn delete_history_entries(state: State<'_, AppState>, ids: Vec<i64>) -> Result<usize, String> {
    let profile_id = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    let db = state.db.lock().unwrap();
    db::delete_history_entries(&db, profile_id, &ids).map_err(|e| e.to_string())
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
            return Err("profil adı boş olamaz".to_string());
        }
        db::add_profile(&db, clean, &color).map_err(|e| e.to_string())?
    };
    // Yeni profille boş arayüz kalmasın — NewTab aç
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
        return Err("profil adı boş olamaz".to_string());
    }
    let db = state.db.lock().unwrap();
    let found = db::profile_by_id(&db, profile_id).map_err(|e| e.to_string())?;
    if found.is_none() {
        return Err("profil bulunamadı".to_string());
    }
    db::rename_profile(&db, profile_id, clean).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_profile(state: State<'_, AppState>, profile_id: i64) -> Result<(), String> {
    let current = tabs::current_profile_id(&state).ok_or("profil bulunamadı")?;
    if profile_id == current {
        return Err("Aktif profili silemezsin — önce başka bir profile geç.".to_string());
    }
    let db = state.db.lock().unwrap();
    let found = db::profile_by_id(&db, profile_id)
        .map_err(|e| e.to_string())?
        .ok_or("profil bulunamadı")?;
    if found.is_default {
        return Err("Varsayılan profil silinemez.".to_string());
    }
    db::delete_profile(&db, profile_id).map_err(|e| e.to_string())
}

/// Profili değiştirir: mevcut sekmeler kapanır ve yeni oturum başlar.
/// (Çoklu pencere desteği v0.2'ye ertelendi — kararlar.md'ye not düşüldü.)
#[tauri::command]
pub fn switch_profile(app: AppHandle, profile_id: i64) -> Result<(), String> {
    tabs::dbg_log("switch_profile: start");
    // Profil değişimi sırasında close_tab'ın otomatik NewTab yaratmasını bastır
    {
        let state = app.state::<AppState>();
        let store = state.tabs.lock().unwrap();
        store.suppress_auto_newtab = true;
        drop(store);
    }
    let labels: Vec<String> = {
        let state = app.state::<AppState>();
        let store = state.tabs.lock().unwrap();
        store.tabs.iter().map(|t| t.id.clone()).collect()
    };
    tabs::dbg_log(&format!("switch_profile: closing {} tab(s)", labels.len()));
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
        store.suppress_auto_newtab = false;
    }
    // Boş arayüz kalmasın — yeni oturum için NewTab aç (tek açılır, çift olmaz)
    let _ = tabs::create_tab(&app, None, false);
    tabs::dbg_log("switch_profile: done");
    Ok(())
}

// ---------- Geliştirici paneli ----------

#[tauri::command]
pub fn toggle_devtools(app: AppHandle) -> Result<(), String> {
    let label = tabs::active_label(&app).ok_or("aktif sekme yok")?;
    let Some(wv) = app.get_webview(&label) else {
        return Err("aktif webview bulunamadı".to_string());
    };
    if wv.is_devtools_open() {
        let _ = wv.close_devtools();
    } else {
        let _ = wv.open_devtools();
    }
    Ok(())
}

// ---------- Şifre Kasası (quantum://vault) ----------

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

// ---------- Tanılama ----------

/// Kabuk/iç sayfalardan gelen frontend hatalarını rust.log'a düşür.
#[tauri::command]
pub fn dbg_log_frontend(msg: String) {
    tabs::dbg_log(&format!("[frontend] {msg}"));
}

// ---------- Komut Paleti (Ctrl+K) ----------

#[tauri::command]
pub fn show_palette(app: AppHandle) -> Result<(), String> {
    tabs::show_palette(&app)
}

#[tauri::command]
pub fn hide_palette(app: AppHandle) {
    tabs::hide_palette(&app);
}

#[tauri::command]
pub fn toggle_palette(app: AppHandle) -> Result<(), String> {
    tabs::toggle_palette(&app)
}

// ---------- Kurulumu Onar ----------

/// Setup'ı indirip sessiz (/S) kur: bozuk kurulumu/sayfaları üzerine onar.
#[tauri::command]
pub fn run_repair(app: AppHandle) -> Result<(), String> {
    let temp = std::env::temp_dir();
    let installer = temp.join("QuantumBrowser_repair.exe");
    let script = format!(
        "Invoke-WebRequest -Uri 'https://github.com/TurkYoshi1905/quantum-browser/releases/latest/download/QuantumBrowser_0.0.4_x64-setup.exe' -OutFile '{}'; Start-Process '{}' -ArgumentList '/S'",
        installer.display(),
        installer.display()
    );
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .spawn()
        .map_err(|e| format!("onarım başlatılamadı: {e}"))?;
    tabs::dbg_log("run_repair: setup indirildi, /S başlatıldı — uygulama kapanıyor");
    app.exit(0);
    Ok(())
}

// ---------- Sessiz arka plan güncelleme ----------

#[tauri::command]
pub fn set_auto_update_background(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    {
        let db = state.db.lock().unwrap();
        db::set_setting(
            &db,
            "auto_update_background",
            if enabled { "1" } else { "0" },
        )
        .map_err(|e| e.to_string())?;
    }
    tabs::emit_tabs_changed(&app);
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
