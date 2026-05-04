#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_path = {
        let data_dir = dirs::data_local_dir()
            .map(|d| d.join("牛马信仰"))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        std::fs::create_dir_all(&data_dir).ok();
        let user_db = data_dir.join("niuma_faith.db");
        if user_db.exists() {
            user_db
        } else {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            let exe_db = exe_dir.join("niuma_faith.db");
            if exe_db.exists() { exe_db } else { user_db }
        }
    };

    let db = Arc::new(
        niuma_faith_lib::data::sqlite::SqliteDb::open(db_path.to_str().unwrap())
            .expect("Failed to open database")
    );
    let app_state = Arc::new(niuma_faith_lib::tauri::state::AppState::new(db));

    let _ = app_state.faith.get_or_create_user();

    let port: u16 = 23456;
    let token = uuid::Uuid::new_v4().to_string().replace("-", "");

    {
        let data_dir = dirs::data_local_dir()
            .map(|d| d.join("牛马信仰"))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        std::fs::create_dir_all(&data_dir).ok();
        std::fs::write(data_dir.join("http_port.txt"), port.to_string()).ok();
        std::fs::write(data_dir.join("http_token.txt"), &token).ok();
    }

    let server_state = app_state.clone();
    std::thread::spawn(move || {
        let server = niuma_faith_lib::local_server::LocalHttpServer::new(server_state, port, token);
        server.run();
    });

    tauri::Builder::default()
        .manage(app_state.clone())
        .setup(move |app| {
            use tauri::tray::TrayIconBuilder;
            use tauri::menu::{Menu, MenuItem};

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&Menu::with_items(app, &[
                    &MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?,
                    &MenuItem::with_id(app, "floating", "打开悬浮窗", true, None::<&str>)?,
                    &MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?,
                ])?)
                .on_menu_event(|app, event| {
                    use tauri::Manager;
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show(); let _ = w.set_focus();
                            }
                        }
                        "floating" => {
                            if let Some(w) = app.get_webview_window("floating") {
                                let _ = w.show();
                            } else {
                                let _ = tauri::WebviewWindowBuilder::new(
                                    app, "floating", tauri::WebviewUrl::App("/floating".into()),
                                )
                                .inner_size(80.0, 80.0)
                                .always_on_top(true)
                                .decorations(false)
                                .skip_taskbar(true)
                                .transparent(true)
                                .shadow(false)
                                .resizable(false)
                                .build();
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            niuma_faith_lib::tauri::commands::get_status,
            niuma_faith_lib::tauri::commands::check_in,
            niuma_faith_lib::tauri::commands::get_today_record,
            niuma_faith_lib::tauri::commands::get_or_create_user,
            niuma_faith_lib::tauri::commands::is_process_running,
            niuma_faith_lib::tauri::commands::list_processes,
            niuma_faith_lib::tauri::commands::create_task,
            niuma_faith_lib::tauri::commands::get_tasks_by_date,
            niuma_faith_lib::tauri::commands::get_tasks,
            niuma_faith_lib::tauri::commands::get_task,
            niuma_faith_lib::tauri::commands::update_task,
            niuma_faith_lib::tauri::commands::complete_task,
            niuma_faith_lib::tauri::commands::abandon_task,
            niuma_faith_lib::tauri::commands::delete_task,
            niuma_faith_lib::tauri::commands::start_task,
            niuma_faith_lib::tauri::commands::pause_task,
            niuma_faith_lib::tauri::commands::resume_task,
            niuma_faith_lib::tauri::commands::end_task,
            niuma_faith_lib::tauri::commands::set_task_recurrence,
            niuma_faith_lib::tauri::commands::get_daily_stats,
            niuma_faith_lib::tauri::commands::get_project_task,
            niuma_faith_lib::tauri::commands::get_project_tasks,
            niuma_faith_lib::tauri::commands::open_floating_widget,
            niuma_faith_lib::tauri::commands::close_floating_widget,
            niuma_faith_lib::tauri::commands::show_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
