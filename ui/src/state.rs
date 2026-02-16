use oxban_core::{Card, Column};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    None,
    CardDetail { card_id: Uuid },
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiState {
    pub search: String,
    pub modal: Modal,
    pub dragging_card: Option<Uuid>,
    pub drag_over_column: Option<Uuid>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            search: String::new(),
            modal: Modal::None,
            dragging_card: None,
            drag_over_column: None,
        }
    }
}

pub fn columns_sorted(columns: &[Column]) -> Vec<Column> {
    let mut out = columns.to_vec();
    out.sort_by_key(|column| column.pos);
    out
}

pub fn cards_for_column(cards: &[Card], column_id: Uuid) -> Vec<Card> {
    let mut out: Vec<Card> = cards
        .iter()
        .filter(|card| card.column_id == column_id)
        .cloned()
        .collect();

    out.sort_by_key(|card| card.pos);
    out
}

pub fn card_matches_search(card: &Card, search: &str) -> bool {
    let search = search.trim().to_lowercase();
    if search.is_empty() {
        return true;
    }

    card.title.to_lowercase().contains(&search)
        || card.description.to_lowercase().contains(&search)
        || card
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(&search))
}

pub fn parse_uuid(raw: &str) -> Option<Uuid> {
    Uuid::parse_str(raw).ok()
}
