use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardSummary {
    pub id: Uuid,
    pub name: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardState {
    pub board: Board,
    pub columns: Vec<Column>,
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Column {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub pos: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Card {
    pub id: Uuid,
    pub board_id: Uuid,
    pub column_id: Uuid,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: i32,
    pub pos: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetBoardArgs {
    pub board_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateBoardArgs {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateColumnArgs {
    pub board_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenameColumnArgs {
    pub column_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReorderColumnArgs {
    pub column_id: Uuid,
    pub before_column_id: Option<Uuid>,
    pub after_column_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateCardArgs {
    pub board_id: Uuid,
    pub column_id: Uuid,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateCardArgs {
    pub card_id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub due_date: Option<Option<DateTime<Utc>>>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoveCardArgs {
    pub card_id: Uuid,
    pub to_column_id: Uuid,
    pub before_card_id: Option<Uuid>,
    pub after_card_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeleteCardArgs {
    pub card_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeleteColumnArgs {
    pub column_id: Uuid,
}
