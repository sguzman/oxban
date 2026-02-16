PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS boards (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS columns (
  id TEXT PRIMARY KEY NOT NULL,
  board_id TEXT NOT NULL,
  name TEXT NOT NULL,
  pos INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(board_id) REFERENCES boards(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_columns_board_pos ON columns(board_id, pos);

CREATE TABLE IF NOT EXISTS cards (
  id TEXT PRIMARY KEY NOT NULL,
  board_id TEXT NOT NULL,
  column_id TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  tags_json TEXT NOT NULL,
  due_date TEXT NULL,
  priority INTEGER NOT NULL,
  pos INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(board_id) REFERENCES boards(id) ON DELETE CASCADE,
  FOREIGN KEY(column_id) REFERENCES columns(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_cards_board ON cards(board_id);
CREATE INDEX IF NOT EXISTS idx_cards_column_pos ON cards(column_id, pos);
