#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod blocker;
mod commands;
mod db;
mod downloads;
mod tabs;

use std::sync::Mutex;
use tabs::AppState;
use tauri::{Builder, Manager, WindowEvent};

fn main() {
    tabs::dbg_log("QuantumBrowser başlatılıyor…");
    Builder::default()
        .setup(|app| {
            // ---------- Veritabanı ----------
            let data_dir = match app.path().app_data_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    tabs::dbg_log(&format!("HATA: app_data_dir alınamadı: {e}"));
                    std::env::temp_dir().join("QuantumBrowser")
                }
            };
            let _ = std::fs::create_dir_all(&data_dir);
            let conn = match db::open(&data_dir.join("quantum.db")) {
                Ok(c) => c,
                Err(e) => {
                    tabs::dbg_log(&format!("HATA: veritabanı açılamadı: {e}"));
                    return Err(format!("Veritabanı açılamadı: {e}").into());
                }
            };
            tabs::dbg_log("Veritabanı açıldı");

            let profile = match db::default_profile(&conn) {
                Ok(p) => p,
                Err(e) => {
                    tabs::dbg_log(&format!("HATA: varsayılan profil okunamadı: {e}"));
                    db::Profile {
                        id: 0,
                        name: "Kişisel".to_string(),
                        color: "#7c6cf5".to_string(),
                        store_id: "fallback".to_string(),
                        is_default: true,
                    }
                }
            };

            let sidebar_collapsed = db::get_setting(&conn, "sidebar_collapsed") == "1";
            let blocker_enabled = db::get_setting(&conn, "blocker_enabled") == "1";

            let mut store = tabs::TabsStore::default();
            store.profile_id = Some(profile.id);
            store.profile_store_id = Some(profile.store_id);
            store.sidebar_collapsed = sidebar_collapsed;
            store.blocker_enabled = blocker_enabled;

            app.manage(AppState {
                db: Mutex::new(conn),
                tabs: Mutex::new(store),
            });

            let handle = app.handle().clone();
            tabs::dbg_log("State kuruldu");

            // ---------- Oturum geri yükleme (hata toleranslı) ----------
            let restored = (|| -> usize {
                let state = handle.state::<AppState>();
                let (session_json, restore_enabled) = {
                    let db = match state.db.lock() {
                        Ok(g) => g,
                        Err(_) => return 0,
                    };
                    (
                        db::load_session(&db).unwrap_or(None),
                        db::get_setting(&db, "session_restore") == "1",
                    )
                };
                if !restore_enabled {
                    return 0;
                }
                let Some(json) = session_json else {
                    return 0;
                };
                let value: serde_json::Value = match serde_json::from_str(&json) {
                    Ok(v) => v,
                    Err(e) => {
                        tabs::dbg_log(&format!("Oturum JSON'u bozuk, yok sayılıyor: {e}"));
                        return 0;
                    }
                };
                let Some(items) = value.get("tabs").and_then(|t| t.as_array()) else {
                    return 0;
                };
                tabs::dbg_log(&format!("Oturum bulundu: {} sekme", items.len()));

                let mut count = 0usize;
                let mut active_label: Option<String> = None;
                // En fazla 10 sekmeyi geri yükle — aşırı yüklemeyi önle
                for item in items.iter().take(10) {
                    let url = item
                        .get("url")
                        .and_then(|u| u.as_str())
                        .unwrap_or("")
                        .to_string();
                    let is_new = item.get("newtab").and_then(|n| n.as_bool()).unwrap_or(true);
                    let pinned = item
                        .get("pinned")
                        .and_then(|p| p.as_bool())
                        .unwrap_or(false);
                    let group_id = item.get("group_id").and_then(|g| g.as_i64());
                    let url_opt = if is_new { None } else { Some(url) };
                    match tabs::create_tab(&handle, url_opt, false) {
                        Ok(label) => {
                            if pinned {
                                let _ = tabs::toggle_pin(&handle, &label);
                            }
                            tabs::apply_group(&handle, &label, group_id);
                            if item
                                .get("active")
                                .and_then(|a| a.as_bool())
                                .unwrap_or(false)
                            {
                                active_label = Some(label);
                            }
                            count += 1;
                        }
                        Err(e) => {
                            tabs::dbg_log(&format!("Oturum sekmesi açılamadı: {e}"));
                        }
                    }
                }
                if let Some(label) = active_label {
                    let _ = tabs::activate_tab(&handle, &label);
                }
                count
            })();
            tabs::dbg_log(&format!("Oturum geri yüklendi: {restored} sekme"));

            // Oturum yoksa yeni NewTab aç — ama başarısız olsa da uygulama çalışır
            if restored == 0 {
                match tabs::create_tab(&handle, None, false) {
                    Ok(_) => tabs::dbg_log("Başlangıç sekmesi açıldı"),
                    Err(e) => {
                        tabs::dbg_log(&format!("HATA: başlangıç sekmesi açılamadı: {e}"));
                        return Err(format!("Sekme oluşturulamadı: {e}").into());
                    }
                }
            }
            tabs::dbg_log("Startup tamamlandı");
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "popover" {
                // Popover penceresi için kaçak kapanmayı engelle: kapansa bile gizle
                let _ = window.hide();
                return;
            }
            if let WindowEvent::Resized(_) = event {
                if window.label() == "shell" {
                    let handle = window.app_handle().clone();
                    let _ = tabs::relayout(&handle);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::new_tab,
            commands::close_tab,
            commands::activate_tab,
            commands::duplicate_tab,
            commands::move_tab,
            commands::toggle_pin,
            commands::close_others,
            commands::navigate,
            commands::go_back,
            commands::go_forward,
            commands::reload,
            commands::new_incognito_tab,
            commands::reopen_closed_tab,
            commands::create_group,
            commands::toggle_group,
            commands::delete_group,
            commands::assign_group,
            commands::set_overlay_open,
            commands::set_sidebar_collapsed,
            commands::set_theme,
            commands::set_search_engine,
            commands::set_blocker_enabled,
            commands::set_session_restore,
            commands::open_menu_popup,
            commands::close_menu_popup,
            commands::list_bookmarks,
            commands::toggle_bookmark,
            commands::is_bookmarked,
            commands::remove_bookmark,
            commands::list_history,
            commands::delete_history_entries,
            commands::rename_profile,
            commands::delete_profile,
            commands::toggle_devtools,
            commands::clear_history,
            commands::list_downloads,
            commands::clear_finished_downloads,
            commands::open_downloads_folder,
            commands::list_profiles,
            commands::add_profile,
            commands::switch_profile,
            commands::clear_browsing_data,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            tabs::dbg_log(&format!("FATAL: QuantumBrowser çalıştırılamadı: {e}"));
            eprintln!("QuantumBrowser başlatılamadı: {e}");
            std::process::exit(1);
        });
}
