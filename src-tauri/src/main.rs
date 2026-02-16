mod app_config;
mod commands;
mod db;
mod logging;
mod positions;

use std::sync::Arc;

use anyhow::Context;
use tauri::Manager;

use app_config::{default_config_text, load_or_init_config, log_file_path};
use commands::AppState;
use db::Db;

fn main() {
    configure_wayland_runtime_defaults();

    tauri::Builder::default()
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .context("failed to locate app config directory")?;

            let config = load_or_init_config(&config_dir, "oxban.toml", default_config_text())?;

            let log_path = if config.logging.write_to_file {
                Some(log_file_path(&config_dir, &config.logging.log_dirname))
            } else {
                None
            };

            logging::init_logging(&config.logging.level, log_path)?;
            tracing::info!(config_dir = %config_dir.display(), "loaded application configuration");

            let db_path = config_dir.join(&config.storage.sqlite_filename);
            let db = tauri::async_runtime::block_on(Db::new(db_path, &config))?;

            let state = AppState {
                db,
                cfg: Arc::new(config),
            };

            app.manage(state);
            install_signal_handlers(app.handle().clone());

            tracing::info!("application setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_effective_config,
            commands::list_boards,
            commands::create_board,
            commands::delete_board,
            commands::rename_board,
            commands::get_board,
            commands::create_column,
            commands::rename_column,
            commands::reorder_column,
            commands::create_card,
            commands::update_card,
            commands::move_card,
            commands::delete_card,
            commands::delete_column,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn install_signal_handlers(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        wait_for_shutdown_signal().await;
        tracing::warn!("received shutdown signal; exiting application");
        app_handle.exit(0);
    });
}

#[cfg(unix)]
async fn wait_for_shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigint = match signal(SignalKind::interrupt()) {
        Ok(stream) => stream,
        Err(error) => {
            tracing::error!(%error, "failed to register SIGINT handler; falling back to ctrl_c");
            let _ = tokio::signal::ctrl_c().await;
            return;
        }
    };

    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(stream) => stream,
        Err(error) => {
            tracing::error!(%error, "failed to register SIGTERM handler; falling back to ctrl_c");
            let _ = tokio::signal::ctrl_c().await;
            return;
        }
    };

    tokio::select! {
        _ = sigint.recv() => {}
        _ = sigterm.recv() => {}
    }
}

#[cfg(not(unix))]
async fn wait_for_shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed waiting for ctrl_c signal");
    }
}

#[cfg(target_os = "linux")]
fn configure_wayland_runtime_defaults() {
    let is_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|session| session.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false);

    if !is_wayland {
        return;
    }

    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        // Safety: this runs at process startup before worker threads are spawned.
        unsafe {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    if std::env::var_os("WEBKIT_DISABLE_COMPOSITING_MODE").is_none() {
        // Safety: this runs at process startup before worker threads are spawned.
        unsafe {
            std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn configure_wayland_runtime_defaults() {}
