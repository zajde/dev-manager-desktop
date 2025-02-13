#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

extern crate core;

use log::LevelFilter;
use tauri::Manager;

use native_dialog::{MessageDialog, MessageType};

use crate::device_manager::DeviceManager;
use crate::session_manager::SessionManager;
use crate::shell_manager::ShellManager;
use crate::spawn_manager::SpawnManager;

mod conn_pool;
mod device_manager;
mod error;
mod event_channel;
mod plugins;
mod remote_files;
mod session_manager;
mod shell_manager;
mod spawn_manager;

fn main() {
    env_logger::builder().filter_level(LevelFilter::Debug).init();
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(wnd) = app.get_window("main") {
                wnd.unminimize().unwrap_or(());
                wnd.set_focus().unwrap_or(());
            }
        }))
        .plugin(plugins::device::plugin("device-manager"))
        .plugin(plugins::cmd::plugin("remote-command"))
        .plugin(plugins::shell::plugin("remote-shell"))
        .plugin(plugins::file::plugin("remote-file"))
        .plugin(plugins::devmode::plugin("dev-mode"))
        .plugin(plugins::local_file::plugin("local-file"))
        .manage(DeviceManager::default())
        .manage(SessionManager::default())
        .manage(SpawnManager::default())
        .manage(ShellManager::default())
        .on_page_load(|wnd, _payload| {
            let spawns = wnd.state::<SpawnManager>();
            spawns.clear();
        })
        .run(tauri::generate_context!());
    if let Err(e) = result {
        #[cfg(windows)]
        if let tauri::Error::Runtime(ref e) = e {
            if format!("{:?}", e).starts_with("CreateWebview(") {
                MessageDialog::new()
                    .set_type(MessageType::Error)
                    .set_title("webOS Dev Manager")
                    .set_text(&format!("Unexpected error occurred: {:?}\nThis may be due to broken installation of WebView2 Runtime. You may need to reinstall WebView2 Runtime as administrator.", e))
                    .show_alert()
                    .expect("Unexpected error occurred while processing unexpected error :(");
                return;
            }
        }
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("webOS Dev Manager")
            .set_text(&format!("Unexpected error occurred: {:?}", e))
            .show_alert()
            .expect("Unexpected error occurred while processing unexpected error :(");
    }
}
