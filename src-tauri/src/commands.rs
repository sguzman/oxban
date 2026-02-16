use std::sync::Arc;

use oxban_core::{
    BoardState, BoardSummary, CreateBoardArgs, CreateCardArgs, CreateColumnArgs, DeleteCardArgs,
    DeleteColumnArgs, GetBoardArgs, MoveCardArgs, RenameColumnArgs, ReorderColumnArgs,
    UpdateCardArgs,
};
use tauri::State;
use tracing::instrument;
use uuid::Uuid;

use crate::{app_config::AppConfig, db::Db};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub cfg: Arc<AppConfig>,
}

fn to_string_error(error: anyhow::Error) -> String {
    error.to_string()
}

#[tauri::command]
#[instrument(skip(state))]
pub async fn get_effective_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok((*state.cfg).clone())
}

#[tauri::command]
#[instrument(skip(state))]
pub async fn list_boards(state: State<'_, AppState>) -> Result<Vec<BoardSummary>, String> {
    state.db.list_boards().await.map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(name = %args.name))]
pub async fn create_board(
    state: State<'_, AppState>,
    args: CreateBoardArgs,
) -> Result<Uuid, String> {
    state
        .db
        .create_board(&state.cfg, args.name)
        .await
        .map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(board_id = %args.board_id))]
pub async fn get_board(
    state: State<'_, AppState>,
    args: GetBoardArgs,
) -> Result<BoardState, String> {
    state
        .db
        .get_board(args.board_id)
        .await
        .map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(board_id = %args.board_id, name = %args.name))]
pub async fn create_column(
    state: State<'_, AppState>,
    args: CreateColumnArgs,
) -> Result<Uuid, String> {
    state
        .db
        .create_column(&state.cfg.ordering, args.board_id, args.name)
        .await
        .map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(column_id = %args.column_id, name = %args.name))]
pub async fn rename_column(
    state: State<'_, AppState>,
    args: RenameColumnArgs,
) -> Result<(), String> {
    state
        .db
        .rename_column(args.column_id, args.name)
        .await
        .map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(column_id = %args.column_id, before = ?args.before_column_id, after = ?args.after_column_id))]
pub async fn reorder_column(
    state: State<'_, AppState>,
    args: ReorderColumnArgs,
) -> Result<(), String> {
    state
        .db
        .reorder_column(
            &state.cfg.ordering,
            args.column_id,
            args.before_column_id,
            args.after_column_id,
        )
        .await
        .map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(board_id = %args.board_id, column_id = %args.column_id))]
pub async fn create_card(state: State<'_, AppState>, args: CreateCardArgs) -> Result<Uuid, String> {
    state
        .db
        .create_card(
            &state.cfg.ordering,
            args.board_id,
            args.column_id,
            args.title,
        )
        .await
        .map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(card_id = %args.card_id))]
pub async fn update_card(state: State<'_, AppState>, args: UpdateCardArgs) -> Result<(), String> {
    state
        .db
        .update_card(
            args.card_id,
            args.title,
            args.description,
            args.tags,
            args.due_date,
            args.priority,
        )
        .await
        .map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(card_id = %args.card_id, to_column_id = %args.to_column_id))]
pub async fn move_card(state: State<'_, AppState>, args: MoveCardArgs) -> Result<(), String> {
    state
        .db
        .move_card(
            &state.cfg.ordering,
            args.card_id,
            args.to_column_id,
            args.before_card_id,
            args.after_card_id,
        )
        .await
        .map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(card_id = %args.card_id))]
pub async fn delete_card(state: State<'_, AppState>, args: DeleteCardArgs) -> Result<(), String> {
    state
        .db
        .delete_card(args.card_id)
        .await
        .map_err(to_string_error)
}

#[tauri::command]
#[instrument(skip(state, args), fields(column_id = %args.column_id))]
pub async fn delete_column(
    state: State<'_, AppState>,
    args: DeleteColumnArgs,
) -> Result<(), String> {
    state
        .db
        .delete_column(args.column_id)
        .await
        .map_err(to_string_error)
}
