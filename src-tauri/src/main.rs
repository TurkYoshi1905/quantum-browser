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

            // İlk sekme: NewTab sayfası
            let handle = app.handle().clone();
            tabs::create_tab(&handle, None, false)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            Ok(())
        })
        .on_window_event(|window, event| {
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
            commands::create_group,
            commands::toggle_group,
            commands::delete_group,
            commands::assign_group,
            commands::set_overlay_open,
            commands::set_sidebar_collapsed,
            commands::set_theme,
            commands::set_search_engine,
            commands::set_blocker_enabled,
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
