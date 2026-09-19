mod commands;
mod device;
mod settings;
mod tray;

use tauri::{Emitter, Manager};

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let (cmd_tx, cmd_rx) = device::command_channel();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_notification::init())
        // Tắt plugin updater:
        // .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(cmd_tx)
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            if let Some(window) = app.get_webview_window("main") {
                // Lưu ý: đoạn này mặc định ẩn cửa sổ và chỉ hiện ở Tray dưới góc máy
                let _ = window.show(); // Sửa từ window.hide() thành window.show() để app hiện ngay cửa sổ khi mở
            }
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(device::run_loop(handle, cmd_rx));

            // Tắt luồng ngầm tự check update sau 10s:
            /*
            let handle2 = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                if let Some(version) = commands::check_update_silent(&handle2).await {
                    let _ = handle2.emit("update-available", version);
                }
            });
            */

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_anc_mode,
            commands::set_eq_preset,
            commands::find_earbud,
            commands::get_settings,
            commands::set_settings,
            commands::get_supported_anc_modes,
            // commands::check_for_update,
            // commands::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}