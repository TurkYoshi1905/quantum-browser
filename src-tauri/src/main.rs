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
    Builder::default()
        .setup(|app| {
            // Veritabanını aç ve durumunu kur
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("uygulama veri klasörü alınamadı");
            std::fs::create_dir_all(&data_dir).ok();
            let conn = db::open(&data_dir.join("quantum.db")).expect("veritabanı açılamadı");

            let profile = db::default_profile(&conn).expect("varsayılan profil bulunamadı");
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

            // Popover penceresi (gizli, dekorsuz, her zaman üstte, şeffaf köşe)
            // Ana pencereden çağrılır; menü öğeleri menü yolunu kullanır.
            let _popover = tauri::webview::WebviewWindowBuilder::new(
                &handle,
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
            .build()?;

            // Oturum geri yükleme: önceki çalışmada açık sekmeleri yeniden aç
            let restored = (|| -> Result<usize, Box<dyn std::error::Error>> {
                let state = handle.state::<AppState>();
                let (session_json, restore_enabled) = {
                    let db = state.db.lock().unwrap();
                    (
                        db::load_session(&db).unwrap_or(None),
                        db::get_setting(&db, "session_restore") == "1",
                    )
                };
                if !restore_enabled {
                    return Ok(0);
                }
                let Some(json) = session_json else {
                    return Ok(0);
                };
                let value: serde_json::Value = serde_json::from_str(&json)?;
                let Some(items) = value.get("tabs").and_then(|t| t.as_array()) else {
                    return Ok(0);
                };
                let mut count = 0usize;
                let mut active_label: Option<String> = None;
                for item in items {
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
                    let label = tabs::create_tab(&handle, url_opt, false)?;
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
                if let Some(label) = active_label {
                    let _ = tabs::activate_tab(&handle, &label);
                }
                Ok(count)
            })()
            .unwrap_or(0);

            // Oturum boşsa ya da geri yüklenemediyse yeni bir sekme aç
            if restored == 0 {
                tabs::create_tab(&handle, None, false)
                    .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            }
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
        .expect("QuantumBrowser başlatılamadı");
}
