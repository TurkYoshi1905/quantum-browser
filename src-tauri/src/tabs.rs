//! QuantumBrowser — sekme motoru & çoklu webview yerleşimi
//! Kabuk penceresi: dikey sekme şeridi + adres çubuğu (React).
//! Her sekme, kabuk penceresinin içinde katman olarak açılan gerçek bir webview'dur.

use crate::blocker;
use crate::db::{self, TabGroup};
use rusqlite::Connection;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{
    webview::{PageLoadEvent, WebviewBuilder},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Webview, WebviewUrl, Window,
};

/// Geçerli profili döndürür (store'da yoksa varsayılan profile düşer).
pub fn current_profile_id(state: &AppState) -> Option<i64> {
    if let Some(id) = state.tabs.lock().unwrap().profile_id {
        return Some(id);
    }
    let db = state.db.lock().unwrap();
    db::default_profile(&db).ok().map(|p| p.id)
}

// ---------- Durum ----------

pub struct AppState {
    pub db: Mutex<Connection>,
    pub tabs: Mutex<TabsStore>,
}

#[derive(Default)]
pub struct TabsStore {
    pub tabs: Vec<TabMeta>,
    pub active: Option<String>,
    pub overlay_open: bool,
    pub sidebar_collapsed: bool,
    pub blocker_enabled: bool,
    pub profile_id: Option<i64>,
    pub profile_store_id: Option<String>,
    pub incognito_store: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct TabMeta {
    pub id: String, // webview etiketi
    pub url: String,
    pub title: String,
    pub group_id: Option<i64>,
    pub pinned: bool,
    pub incognito: bool,
    pub blocked_count: u32,
    pub is_newtab: bool,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub tabs: Vec<TabMeta>,
    pub active_id: Option<String>,
    pub groups: Vec<TabGroup>,
    pub profile_id: Option<i64>,
    pub profile_name: String,
    pub profile_color: String,
    pub profiles: Vec<crate::db::Profile>,
    pub theme: String,
    pub search_engine: String,
    pub blocker_enabled: bool,
    pub sidebar_collapsed: bool,
    pub overlay_open: bool,
}

// ---------- Yerleşim ----------

const TITLEBAR_H: f64 = 40.0;
const SIDEBAR_W: f64 = 248.0;
const SIDEBAR_W_COLLAPSED: f64 = 60.0;

pub fn sidebar_width(collapsed: bool) -> f64 {
    if collapsed {
        SIDEBAR_W_COLLAPSED
    } else {
        SIDEBAR_W
    }
}

fn scale(window: &Window) -> f64 {
    window.scale_factor().unwrap_or(1.0)
}

fn logical_inner(window: &Window) -> (f64, f64) {
    let scale = scale(window);
    match window.inner_size() {
        Ok(size) => (size.width as f64 / scale, size.height as f64 / scale),
        Err(_) => (1440.0, 900.0),
    }
}

pub fn page_rect(window: &Window, collapsed: bool) -> (LogicalPosition<f64>, LogicalSize<f64>) {
    let (w, h) = logical_inner(window);
    let sidebar = sidebar_width(collapsed);
    let pw = (w - sidebar).max(240.0);
    let ph = (h - TITLEBAR_H).max(180.0);
    (
        LogicalPosition::new(sidebar, TITLEBAR_H),
        LogicalSize::new(pw, ph),
    )
}

/// Aktif webview'u sayfa alanına alır, diğerlerini 1x1'e küçültür.
/// (hide/show yerine yalnızca bounds — sık çağrıda kararlı çalışır)
pub fn relayout(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_window("shell")
        .ok_or_else(|| "kabuk penceresi bulunamadı".to_string())?;
    let (active, overlay, collapsed) = {
        let state = app.state::<AppState>();
        let store = state.tabs.lock().unwrap();
        (
            store.active.clone(),
            store.overlay_open,
            store.sidebar_collapsed,
        )
    };
    let page = page_rect(&window, collapsed);
    let zero = (LogicalPosition::new(0.0, 0.0), LogicalSize::new(1.0, 1.0));
    let labels: Vec<String> = {
        let state = app.state::<AppState>();
        let store = state.tabs.lock().unwrap();
        store.tabs.iter().map(|t| t.id.clone()).collect()
    };
    for label in labels {
        if let Some(wv) = app.get_webview(&label) {
            let is_active = !overlay && active.as_deref() == Some(label.as_str());
            if is_active {
                set_webview_rect(&wv, page.0, page.1);
            } else {
                set_webview_rect(&wv, zero.0, zero.1);
            }
        }
    }
    Ok(())
}

fn set_webview_rect(wv: &Webview, pos: LogicalPosition<f64>, size: LogicalSize<f64>) {
    // Önce konum sonra boyut (boyut 0'a düşerse konum önemli değil)
    let _ = wv.set_position(pos);
    let _ = wv.set_size(size);
}

// ---------- Snapshot / olaylar ----------

pub fn emit_tabs_changed(app: &AppHandle) {
    let _ = app.emit("tabs-changed", serde_json::json!({}));
}

pub fn snapshot(app: &AppHandle) -> Snapshot {
    let state = app.state::<AppState>();
    let (tabs, active, overlay, collapsed, blocker_enabled, profile_id) = {
        let store = state.tabs.lock().unwrap();
        (
            store.tabs.clone(),
            store.active.clone(),
            store.overlay_open,
            store.sidebar_collapsed,
            store.blocker_enabled,
            store.profile_id,
        )
    };
    let db = state.db.lock().unwrap();
    let profiles = db::list_profiles(&db).unwrap_or_default();
    let profile = profile_id
        .and_then(|id| profiles.iter().find(|p| p.id == id).cloned())
        .or_else(|| profiles.first().cloned());
    let groups = profile
        .as_ref()
        .and_then(|p| db::list_groups(&db, p.id).ok())
        .unwrap_or_default();
    Snapshot {
        tabs,
        active_id: active,
        groups,
        profile_id: profile.as_ref().map(|p| p.id),
        profile_name: profile.as_ref().map(|p| p.name.clone()).unwrap_or_default(),
        profile_color: profile
            .as_ref()
            .map(|p| p.color.clone())
            .unwrap_or_else(|| "#7c6cf5".to_string()),
        profiles,
        theme: db::get_setting(&db, "theme"),
        search_engine: db::get_setting(&db, "search_engine"),
        blocker_enabled,
        sidebar_collapsed: collapsed,
        overlay_open: overlay,
    }
}

// ---------- URL normalizasyonu ----------

pub fn normalize_url(input: &str, search_engine: &str) -> String {
    let q = input.trim();
    if q.is_empty() {
        return String::new(); // newtab'a dön
    }
    if q.starts_with("quantum://newtab") {
        return String::new();
    }
    if q.starts_with("http://") || q.starts_with("https://") {
        return q.to_string();
    }
    // Scheme yoksa: "youtube.com" gibi alan adları → https, değilse arama
    let looks_like_host = q.contains('.')
        && !q.contains(char::is_whitespace)
        && !q.starts_with('/')
        && q.chars().all(|c| c.is_alphanumeric() || ".-_:".contains(c));
    if looks_like_host {
        return format!("https://{q}");
    }
    let encoded = percent_encode(q);
    let template = match search_engine {
        "google" => "https://www.google.com/search?q=",
        "bing" => "https://www.bing.com/search?q=",
        "brave" => "https://search.brave.com/search?q=",
        "ecosia" => "https://www.ecosia.org/search?q=",
        _ => "https://duckduckgo.com/?q=",
    };
    format!("{template}{encoded}")
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

// ---------- Sekme oluşturma ----------

fn active_profile(app: &AppHandle) -> (i64, String) {
    let state = app.state::<AppState>();
    let store = state.tabs.lock().unwrap();
    if let (Some(id), Some(store_id)) = (store.profile_id, store.profile_store_id.clone()) {
        return (id, store_id);
    }
    drop(store);
    let db = state.db.lock().unwrap();
    let profile = db::default_profile(&db).expect("varsayılan profil");
    (profile.id, profile.store_id)
}

fn incognito_store_id(app: &AppHandle) -> String {
    let state = app.state::<AppState>();
    let mut store = state.tabs.lock().unwrap();
    if store.incognito_store.is_none() {
        store.incognito_store = Some(uuid::Uuid::new_v4().to_string());
    }
    store.incognito_store.clone().unwrap()
}

pub fn create_tab(
    app: &AppHandle,
    url_opt: Option<String>,
    incognito: bool,
) -> Result<String, String> {
    let window = app
        .get_window("shell")
        .ok_or_else(|| "kabuk penceresi bulunamadı".to_string())?;

    let state = app.state::<AppState>();
    let collapsed = state.tabs.lock().unwrap().sidebar_collapsed;
    let (pos, size) = page_rect(&window, collapsed);

    let (profile_id, store_id) = active_profile(app);
    let store_id = if incognito {
        incognito_store_id(app)
    } else {
        store_id
    };

    let search_engine = {
        let db = state.db.lock().unwrap();
        db::get_setting(&db, "search_engine")
    };
    let normalized = url_opt.as_ref().map(|u| normalize_url(u, &search_engine));

    let label = format!("tab-{}", uuid::Uuid::new_v4().simple());
    let is_newtab = normalized.as_deref().unwrap_or("").is_empty();
    let webview_url = if is_newtab {
        WebviewUrl::App("index.html#/newtab".into())
    } else {
        WebviewUrl::External(
            normalized
                .clone()
                .unwrap()
                .parse()
                .map_err(|e| format!("geçersiz URL: {e}"))?,
        )
    };

    let mut builder =
        WebviewBuilder::new(&label, webview_url).initialization_script(blocker::COSMETIC_JS);

    // Profil bazında ayrı çerez-deposu / gizli sekmeler için WebView2'nin
    // kalıcı olmayan (non-persistent) veri deposu
    if incognito {
        builder = builder.incognito(true);
    } else {
        let profiles_dir = app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("profiles");
        std::fs::create_dir_all(&profiles_dir).ok();
        builder = builder.data_directory(profiles_dir.join(&store_id));
    }

    // Sekme başlığı değişikliklerini bağla (kararlı API — COM gerekmez)
    {
        let app_t = app.clone();
        let label_t = label.clone();
        builder = builder.on_document_title_changed(move |_wv, title| {
            let state = app_t.state::<AppState>();
            let mut store = state.tabs.lock().unwrap();
            let Some(tab) = store.tabs.iter_mut().find(|t| t.id == label_t) else {
                return;
            };
            tab.title = title.clone();
            let url = tab.url.clone();
            let incognito = tab.incognito;
            drop(store);
            if !incognito {
                if let Ok(db) = state.db.lock() {
                    let _ = db::update_history_title(&db, &url, &title);
                }
            }
            emit_tabs_changed(&app_t);
        });
    }

    // İndirme yöneticisini bağla (kararlı API)
    builder = builder.on_download(crate::downloads::on_download(app.clone(), profile_id));

    let app_for_load = app.clone();
    let load_profile_id = profile_id;
    builder = builder.on_page_load(move |wv, payload| {
        if !matches!(payload.event(), PageLoadEvent::Finished) {
            return;
        }
        let url = payload.url().to_string();
        if url.starts_with("tauri://") || url.contains("tauri.localhost") {
            return; // kabuk uygulamasının kendi sayfaları
        }
        let label = wv.label().to_string();
        let state = app_for_load.state::<AppState>();
        // Tab URL'sini güncelle
        let domain = blocker::domain_of(&url);
        {
            let mut store = state.tabs.lock().unwrap();
            if let Some(tab) = store.tabs.iter_mut().find(|t| t.id == label) {
                let title_empty = tab.title.is_empty();
                tab.url = url.clone();
                tab.is_newtab = false;
                if title_empty {
                    tab.title = domain.clone();
                }
            }
            let is_incog = store
                .tabs
                .iter()
                .find(|t| t.id == label)
                .map(|t| t.incognito)
                .unwrap_or(false);
            if !is_incog {
                if let Ok(db) = state.db.lock() {
                    let _ = db::record_history(&db, load_profile_id, &url, &domain);
                }
            }
        }
        emit_tabs_changed(&app_for_load);
    });

    let webview = window
        .add_child(builder, pos, size)
        .map_err(|e| format!("webview oluşturulamadı: {e}"))?;

    // Reklam engelleyiciyi bağla (Windows)
    hook_web_resource_requested(&webview, app, label.clone());

    // Store'a ekle ve aktifleştir
    {
        let mut store = state.tabs.lock().unwrap();
        store.tabs.push(TabMeta {
            id: label.clone(),
            url: normalized.unwrap_or_default(),
            title: if is_newtab {
                "Yeni Sekme".to_string()
            } else {
                let url = url_opt.clone().unwrap_or_default();
                blocker::domain_of(&url)
            },
            group_id: None,
            pinned: false,
            incognito,
            blocked_count: 0,
            is_newtab,
        });
    }
    activate_tab(app, &label)?;
    Ok(label)
}

pub fn activate_tab(app: &AppHandle, label: &str) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        let mut store = state.tabs.lock().unwrap();
        if store.tabs.iter().any(|t| t.id == label) {
            store.active = Some(label.to_string());
        } else {
            return Err("sekme bulunamadı".to_string());
        }
    }
    relayout(app)?;
    emit_tabs_changed(app);
    Ok(())
}

pub fn close_tab(app: &AppHandle, label: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    let was_active;
    {
        let store = state.tabs.lock().unwrap();
        was_active = store.active.as_deref() == Some(label);
    }
    destroy_webview(app, label);
    let mut store = state.tabs.lock().unwrap();
    let idx = store.tabs.iter().position(|t| t.id == label);
    store.tabs.retain(|t| t.id != label);
    if was_active {
        let next = idx
            .and_then(|i| store.tabs.get(i.min(store.tabs.len().saturating_sub(1))))
            .or_else(|| store.tabs.last())
            .map(|t| t.id.clone());
        store.active = next;
    }
    drop(store);
    relayout(app)?;
    emit_tabs_changed(app);
    Ok(())
}

fn destroy_webview(app: &AppHandle, label: &str) {
    if let Some(wv) = app.get_webview(label) {
        let _ = wv.close();
    }
}

pub fn duplicate_tab(app: &AppHandle, label: &str) -> Result<(), String> {
    let source = {
        let state = app.state::<AppState>();
        let store = state.tabs.lock().unwrap();
        store.tabs.iter().find(|t| t.id == label).cloned()
    };
    let Some(source) = source else {
        return Err("sekme bulunamadı".to_string());
    };
    if source.is_newtab {
        create_tab(app, None, source.incognito)?;
    } else {
        create_tab(app, Some(source.url), source.incognito)?;
    }
    Ok(())
}

pub fn move_tab(app: &AppHandle, from_id: &str, to_id: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut store = state.tabs.lock().unwrap();
    let from = store
        .tabs
        .iter()
        .position(|t| t.id == from_id)
        .ok_or("kaynak sekme bulunamadı")?;
    if store.tabs.iter().all(|t| t.id != to_id) {
        return Err("hedef sekme bulunamadı".to_string());
    }
    let tab = store.tabs.remove(from);
    let to_idx = store.tabs.iter().position(|t| t.id == to_id).unwrap();
    store.tabs.insert(to_idx, tab);
    drop(store);
    emit_tabs_changed(app);
    Ok(())
}

pub fn toggle_pin(app: &AppHandle, label: &str) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut store = state.tabs.lock().unwrap();
    if let Some(tab) = store.tabs.iter_mut().find(|t| t.id == label) {
        tab.pinned = !tab.pinned;
    } else {
        return Err("sekme bulunamadı".to_string());
    }
    drop(store);
    emit_tabs_changed(app);
    Ok(())
}

pub fn close_others(app: &AppHandle, keep: &str) -> Result<(), String> {
    let labels: Vec<String> = {
        let state = app.state::<AppState>();
        let store = state.tabs.lock().unwrap();
        store
            .tabs
            .iter()
            .filter(|t| t.id != keep)
            .map(|t| t.id.clone())
            .collect()
    };
    for label in labels {
        let _ = close_tab(app, &label);
    }
    Ok(())
}

pub fn set_overlay_open(app: &AppHandle, open: bool) {
    {
        let state = app.state::<AppState>();
        let mut store = state.tabs.lock().unwrap();
        store.overlay_open = open;
    }
    let _ = relayout(app);
    emit_tabs_changed(app);
}

pub fn set_sidebar_collapsed(app: &AppHandle, collapsed: bool) {
    {
        let state = app.state::<AppState>();
        let mut store = state.tabs.lock().unwrap();
        store.sidebar_collapsed = collapsed;
    }
    {
        let state = app.state::<AppState>();
        let db = state.db.lock().unwrap();
        let _ = db::set_setting(&db, "sidebar_collapsed", if collapsed { "1" } else { "0" });
    }
    let _ = relayout(app);
    emit_tabs_changed(app);
}

pub fn set_blocker_enabled(app: &AppHandle, enabled: bool) {
    {
        let state = app.state::<AppState>();
        let mut store = state.tabs.lock().unwrap();
        store.blocker_enabled = enabled;
    }
    let state = app.state::<AppState>();
    let db = state.db.lock().unwrap();
    let _ = db::set_setting(&db, "blocker_enabled", if enabled { "1" } else { "0" });
}

pub fn navigate_active(app: &AppHandle, url_input: &str) -> Result<(), String> {
    let search_engine = {
        let state = app.state::<AppState>();
        let db = state.db.lock().unwrap();
        db::get_setting(&db, "search_engine")
    };
    let normalized = normalize_url(url_input, &search_engine);
    let url = if normalized.is_empty() {
        return Ok(());
    } else {
        normalized
    };
    let active = {
        let state = app.state::<AppState>();
        let store = state.tabs.lock().unwrap();
        store.active.clone()
    };
    let Some(label) = active else {
        return Err("aktif sekme yok".to_string());
    };
    {
        let state = app.state::<AppState>();
        let mut store = state.tabs.lock().unwrap();
        if let Some(tab) = store.tabs.iter_mut().find(|t| t.id == label) {
            tab.url = url.clone();
            tab.is_newtab = false;
        }
    }
    if let Some(wv) = app.get_webview(&label) {
        let _ = wv.navigate(url.parse().map_err(|e| format!("geçersiz URL: {e}"))?);
    }
    emit_tabs_changed(app);
    Ok(())
}

pub fn active_label(app: &AppHandle) -> Option<String> {
    let state = app.state::<AppState>();
    let store = state.tabs.lock().unwrap();
    store.active.clone()
}

pub fn active_url(app: &AppHandle) -> String {
    let state = app.state::<AppState>();
    let store = state.tabs.lock().unwrap();
    store
        .active
        .as_deref()
        .and_then(|label| {
            store
                .tabs
                .iter()
                .find(|t| t.id == label)
                .map(|t| t.url.clone())
        })
        .unwrap_or_default()
}

pub fn is_active_incognito(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();
    let store = state.tabs.lock().unwrap();
    store
        .active
        .as_deref()
        .and_then(|label| {
            store
                .tabs
                .iter()
                .find(|t| t.id == label)
                .map(|t| t.incognito)
        })
        .unwrap_or(false)
}

// ---------- Windows'a özgü kancalar ----------

/// PWSTR'ı String'e çevirir. WebView2 PWSTR çıktılarını CoTaskMemAlloc ile ayırır;
/// v0.1'de bu küçük tamponlar bilinçli olarak serbest bırakılmaz (sızıntı < 1 MB).
#[cfg(windows)]
fn take_pwstr(f: impl FnOnce(*mut windows::core::PWSTR) -> windows::core::Result<()>) -> String {
    let mut p = windows::core::PWSTR::null();
    if f(&mut p as *mut _).is_err() {
        return String::new();
    }
    unsafe {
        if p.is_null() {
            return String::new();
        }
        let mut len = 0usize;
        while *p.0.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(p.0, len))
    }
}

static BLOCKER_LAST_EMIT: std::sync::LazyLock<Mutex<HashMap<String, std::time::Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(windows)]
fn hook_web_resource_requested(webview: &Webview, app: &AppHandle, label: String) {
    use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL;
    use webview2_com::WebResourceRequestedEventHandler;
    use windows::core::{Interface, HSTRING, PCWSTR};
    use windows::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal;
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

    let app_for_block = app.clone();
    let label_for_block = label.clone();
    let _ = webview.with_webview(move |platform| {
        let Ok(core) = (unsafe { platform.controller().CoreWebView2() }) else {
            return;
        };
        let filter = HSTRING::from("*");
        unsafe {
            let _ = core.AddWebResourceRequestedFilter(
                PCWSTR(filter.as_ptr()),
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            );
        }
        let environment = platform.environment();
        let handler = WebResourceRequestedEventHandler::create(Box::new(move |_sender, args| {
            let Some(args) = args else { return Ok(()) };
            let enabled = app_for_block
                .state::<AppState>()
                .tabs
                .lock()
                .unwrap()
                .blocker_enabled;
            if !enabled {
                return Ok(());
            }
            // İstek URL'sini oku (out-param PWSTR)
            let url = {
                let mut result = String::new();
                if let Ok(req) = unsafe { args.Request() } {
                    result = take_pwstr(|p| unsafe { req.Uri(p) });
                }
                result
            };
            if !blocker::is_blocked(&url) {
                return Ok(());
            }
            // Engellenen sayacı artır + olayı yayın (en fazla 2.5 kez/sn)
            {
                let state = app_for_block.state::<AppState>();
                let count = {
                    let mut store = state.tabs.lock().unwrap();
                    store
                        .tabs
                        .iter_mut()
                        .find(|t| t.id == label_for_block)
                        .map(|tab| {
                            tab.blocked_count += 1;
                            tab.blocked_count
                        })
                };
                if let Some(count) = count {
                    let mut last = BLOCKER_LAST_EMIT.lock().unwrap();
                    let now = std::time::Instant::now();
                    let should = last
                        .get(&label_for_block)
                        .map(|t| now.duration_since(*t) >= std::time::Duration::from_millis(400))
                        .unwrap_or(true);
                    if should {
                        last.insert(label_for_block.clone(), now);
                        let _ = app_for_block.emit(
                            "blocker-count",
                            serde_json::json!({ "id": label_for_block, "count": count }),
                        );
                    }
                }
            }
            // Boş bir 403 yanıtı ile engelle
            unsafe {
                let body: &[u8] = b"";
                let alloc = GlobalAlloc(GMEM_MOVEABLE, body.len().max(1));
                if let Ok(hglobal) = alloc {
                    let ptr = GlobalLock(hglobal);
                    if !ptr.is_null() {
                        std::ptr::copy_nonoverlapping(body.as_ptr(), ptr as *mut u8, body.len());
                        let _ = GlobalUnlock(hglobal);
                    }
                    if let Ok(stream) = CreateStreamOnHGlobal(hglobal, true) {
                        let reason = HSTRING::from("Blocked by QuantumBrowser");
                        let headers = HSTRING::from("Content-Type: text/plain");
                        if let Ok(response) = environment.CreateWebResourceResponse(
                            &stream,
                            403,
                            PCWSTR(reason.as_ptr()),
                            PCWSTR(headers.as_ptr()),
                        ) {
                            let _ = args.SetResponse(&response);
                        }
                    }
                }
            }
            Ok(())
        }));
        let mut token: i64 = 0;
        unsafe {
            let _ = core.add_WebResourceRequested(&handler, &mut token);
        }
        let _ = Interface::as_raw(&core); // sessiz bırakma
    });
}

#[cfg(not(windows))]
fn hook_web_resource_requested(_webview: &Webview, _app: &AppHandle, _label: String) {}
