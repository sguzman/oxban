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

            tracing::info!("application setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_effective_config,
            commands::list_boards,
            commands::create_board,
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
