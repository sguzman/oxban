use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use oxban_core::{Board, BoardState, BoardSummary, Card, Column};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use uuid::Uuid;

use crate::{
    app_config::{AppConfig, OrderingSection},
    positions,
};

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    #[tracing::instrument(skip(cfg), fields(db_path = %db_path.display()))]
    pub async fn new(db_path: PathBuf, cfg: &AppConfig) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let db_url = format!("sqlite://{}", db_path.display());
        tracing::info!(%db_url, "opening sqlite database");

        let connect_options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .foreign_keys(cfg.storage.foreign_keys);

        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(connect_options)
            .await
            .with_context(|| format!("failed to connect to sqlite database at {db_url}"))?;

        let fk = if cfg.storage.foreign_keys {
            "ON"
        } else {
            "OFF"
        };
        sqlx::query(&format!("PRAGMA foreign_keys = {fk};"))
            .execute(&pool)
            .await?;

        let journal_mode = normalize_journal_mode(&cfg.storage.journal_mode);
        sqlx::query(&format!("PRAGMA journal_mode = {journal_mode};"))
            .execute(&pool)
            .await?;

        let synchronous = normalize_synchronous(&cfg.storage.synchronous);
        sqlx::query(&format!("PRAGMA synchronous = {synchronous};"))
            .execute(&pool)
            .await?;

        tracing::info!(
            journal_mode,
            synchronous,
            foreign_keys = cfg.storage.foreign_keys,
            "applied sqlite pragmas"
        );

        sqlx::migrate!("./migrations").run(&pool).await?;
        tracing::info!("database migrations complete");

        Ok(Self { pool })
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_boards(&self) -> anyhow::Result<Vec<BoardSummary>> {
        let rows = sqlx::query("SELECT id, name, updated_at FROM boards ORDER BY updated_at DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut boards = Vec::with_capacity(rows.len());
        for row in rows {
            boards.push(BoardSummary {
                id: parse_uuid(row.try_get::<String, _>("id")?)?,
                name: row.try_get::<String, _>("name")?,
                updated_at: parse_utc(&row.try_get::<String, _>("updated_at")?)?,
            });
        }

        tracing::debug!(count = boards.len(), "listed boards");
        Ok(boards)
    }

    #[tracing::instrument(skip(self), fields(board_id = %board_id))]
    pub async fn get_board(&self, board_id: Uuid) -> anyhow::Result<BoardState> {
        let board_row =
            sqlx::query("SELECT id, name, created_at, updated_at FROM boards WHERE id = ?")
                .bind(board_id.to_string())
                .fetch_optional(&self.pool)
                .await?;

        let board_row = match board_row {
            Some(row) => row,
            None => bail!("board {board_id} was not found"),
        };

        let board = Board {
            id: parse_uuid(board_row.try_get::<String, _>("id")?)?,
            name: board_row.try_get::<String, _>("name")?,
            created_at: parse_utc(&board_row.try_get::<String, _>("created_at")?)?,
            updated_at: parse_utc(&board_row.try_get::<String, _>("updated_at")?)?,
        };

        let column_rows = sqlx::query(
            "SELECT id, board_id, name, pos, created_at, updated_at \
             FROM columns WHERE board_id = ? ORDER BY pos ASC",
        )
        .bind(board_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut columns = Vec::with_capacity(column_rows.len());
        for row in column_rows {
            columns.push(Column {
                id: parse_uuid(row.try_get::<String, _>("id")?)?,
                board_id: parse_uuid(row.try_get::<String, _>("board_id")?)?,
                name: row.try_get::<String, _>("name")?,
                pos: row.try_get::<i64, _>("pos")?,
                created_at: parse_utc(&row.try_get::<String, _>("created_at")?)?,
                updated_at: parse_utc(&row.try_get::<String, _>("updated_at")?)?,
            });
        }

        let card_rows = sqlx::query(
            "SELECT id, board_id, column_id, title, description, tags_json, due_date, priority, pos, created_at, updated_at \
             FROM cards WHERE board_id = ? ORDER BY column_id ASC, pos ASC",
        )
        .bind(board_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut cards = Vec::with_capacity(card_rows.len());
        for row in card_rows {
            let due_date = row
                .try_get::<Option<String>, _>("due_date")?
                .as_deref()
                .map(parse_utc)
                .transpose()?;

            let tags = serde_json::from_str::<Vec<String>>(&row.try_get::<String, _>("tags_json")?)
                .unwrap_or_default();

            cards.push(Card {
                id: parse_uuid(row.try_get::<String, _>("id")?)?,
                board_id: parse_uuid(row.try_get::<String, _>("board_id")?)?,
                column_id: parse_uuid(row.try_get::<String, _>("column_id")?)?,
                title: row.try_get::<String, _>("title")?,
                description: row.try_get::<String, _>("description")?,
                tags,
                due_date,
                priority: row.try_get::<i32, _>("priority")?,
                pos: row.try_get::<i64, _>("pos")?,
                created_at: parse_utc(&row.try_get::<String, _>("created_at")?)?,
                updated_at: parse_utc(&row.try_get::<String, _>("updated_at")?)?,
            });
        }

        tracing::debug!(
            column_count = columns.len(),
            card_count = cards.len(),
            "loaded board snapshot"
        );

        Ok(BoardState {
            board,
            columns,
            cards,
        })
    }

    #[tracing::instrument(skip(self, cfg), fields(name = %name))]
    pub async fn create_board(&self, cfg: &AppConfig, name: String) -> anyhow::Result<Uuid> {
        let board_id = Uuid::new_v4();
        let now = now_rfc3339();

        let mut tx = self.pool.begin().await?;

        sqlx::query("INSERT INTO boards (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(board_id.to_string())
            .bind(name)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;

        if cfg.app.seed_default_columns {
            let mut pos = 0_i64;
            for column_name in &cfg.app.default_columns {
                sqlx::query(
                    "INSERT INTO columns (id, board_id, name, pos, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(Uuid::new_v4().to_string())
                .bind(board_id.to_string())
                .bind(column_name)
                .bind(pos)
                .bind(&now)
                .bind(&now)
                .execute(&mut *tx)
                .await?;

                pos = pos.saturating_add(cfg.ordering.step);
            }
        }

        tx.commit().await?;

        tracing::info!(%board_id, "created board");
        Ok(board_id)
    }

    #[tracing::instrument(skip(self), fields(board_id = %board_id))]
    pub async fn delete_board(&self, board_id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM boards WHERE id = ?")
            .bind(board_id.to_string())
            .execute(&self.pool)
            .await?;

        tracing::info!(board_id = %board_id, "deleted board");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(board_id = %board_id, name = %name))]
    pub async fn rename_board(&self, board_id: Uuid, name: String) -> anyhow::Result<()> {
        let rows = sqlx::query("UPDATE boards SET name = ?, updated_at = ? WHERE id = ?")
            .bind(name)
            .bind(now_rfc3339())
            .bind(board_id.to_string())
            .execute(&self.pool)
            .await?;

        if rows.rows_affected() == 0 {
            bail!("board {board_id} was not found");
        }

        tracing::info!(board_id = %board_id, "renamed board");
        Ok(())
    }

    #[tracing::instrument(skip(self, ordering), fields(board_id = %board_id, name = %name))]
    pub async fn create_column(
        &self,
        ordering: &OrderingSection,
        board_id: Uuid,
        name: String,
    ) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let now = now_rfc3339();

        let last_pos = self.last_column_position(board_id).await?;
        let pos = positions::between(ordering, last_pos, None);

        sqlx::query(
            "INSERT INTO columns (id, board_id, name, pos, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(board_id.to_string())
        .bind(name)
        .bind(pos)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.touch_board(board_id).await?;

        tracing::info!(%id, board_id = %board_id, "created column");
        Ok(id)
    }

    #[tracing::instrument(skip(self), fields(column_id = %column_id, name = %name))]
    pub async fn rename_column(&self, column_id: Uuid, name: String) -> anyhow::Result<()> {
        let board_id = self.board_id_for_column(column_id).await?;

        sqlx::query("UPDATE columns SET name = ?, updated_at = ? WHERE id = ?")
            .bind(name)
            .bind(now_rfc3339())
            .bind(column_id.to_string())
            .execute(&self.pool)
            .await?;

        self.touch_board(board_id).await?;

        tracing::info!(column_id = %column_id, board_id = %board_id, "renamed column");
        Ok(())
    }

    #[tracing::instrument(skip(self, ordering), fields(column_id = %column_id, before = ?before_column_id, after = ?after_column_id))]
    pub async fn reorder_column(
        &self,
        ordering: &OrderingSection,
        column_id: Uuid,
        before_column_id: Option<Uuid>,
        after_column_id: Option<Uuid>,
    ) -> anyhow::Result<()> {
        let board_id = self.board_id_for_column(column_id).await?;

        let mut left = match after_column_id {
            Some(after_id) => Some(self.position_for_column(after_id).await?),
            None => None,
        };
        let mut right = match before_column_id {
            Some(before_id) => Some(self.position_for_column(before_id).await?),
            None => None,
        };

        if left.is_none() && right.is_none() {
            left = self.last_column_position(board_id).await?;
        }

        if let (Some(l), Some(r)) = (left, right) {
            if !positions::gap_ok(ordering, l, r) {
                tracing::warn!(board_id = %board_id, "column positions are dense; renormalizing");
                self.renormalize_columns(ordering, board_id).await?;
                left = match after_column_id {
                    Some(after_id) => Some(self.position_for_column(after_id).await?),
                    None => None,
                };
                right = match before_column_id {
                    Some(before_id) => Some(self.position_for_column(before_id).await?),
                    None => None,
                };
                if left.is_none() && right.is_none() {
                    left = self.last_column_position(board_id).await?;
                }
            }
        }

        let new_pos = positions::between(ordering, left, right);

        sqlx::query("UPDATE columns SET pos = ?, updated_at = ? WHERE id = ?")
            .bind(new_pos)
            .bind(now_rfc3339())
            .bind(column_id.to_string())
            .execute(&self.pool)
            .await?;

        self.touch_board(board_id).await?;

        tracing::info!(column_id = %column_id, board_id = %board_id, pos = new_pos, "reordered column");
        Ok(())
    }

    #[tracing::instrument(skip(self, ordering), fields(board_id = %board_id, column_id = %column_id, title = %title))]
    pub async fn create_card(
        &self,
        ordering: &OrderingSection,
        board_id: Uuid,
        column_id: Uuid,
        title: String,
    ) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let now = now_rfc3339();

        let last_pos = self.last_card_position(column_id).await?;
        let pos = positions::between(ordering, last_pos, None);

        sqlx::query(
            "INSERT INTO cards (id, board_id, column_id, title, description, tags_json, due_date, priority, pos, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id.to_string())
        .bind(board_id.to_string())
        .bind(column_id.to_string())
        .bind(title)
        .bind("")
        .bind("[]")
        .bind(Option::<String>::None)
        .bind(0_i32)
        .bind(pos)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.touch_board(board_id).await?;

        tracing::info!(%id, board_id = %board_id, column_id = %column_id, "created card");
        Ok(id)
    }

    #[tracing::instrument(skip(self, title, description, tags, due_date), fields(card_id = %card_id))]
    pub async fn update_card(
        &self,
        card_id: Uuid,
        title: Option<String>,
        description: Option<String>,
        tags: Option<Vec<String>>,
        due_date: Option<Option<DateTime<Utc>>>,
        priority: Option<i32>,
    ) -> anyhow::Result<()> {
        let existing = sqlx::query(
            "SELECT board_id, title, description, tags_json, due_date, priority FROM cards WHERE id = ?",
        )
        .bind(card_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        let existing = match existing {
            Some(row) => row,
            None => bail!("card {card_id} was not found"),
        };

        let board_id = parse_uuid(existing.try_get::<String, _>("board_id")?)?;
        let new_title = title.unwrap_or(existing.try_get::<String, _>("title")?);
        let new_description = description.unwrap_or(existing.try_get::<String, _>("description")?);

        let new_tags_json = match tags {
            Some(values) => serde_json::to_string(&values)?,
            None => existing.try_get::<String, _>("tags_json")?,
        };

        let new_due_date = match due_date {
            Some(Some(dt)) => Some(dt.to_rfc3339()),
            Some(None) => None,
            None => existing.try_get::<Option<String>, _>("due_date")?,
        };

        let new_priority = priority.unwrap_or(existing.try_get::<i32, _>("priority")?);

        sqlx::query(
            "UPDATE cards SET title = ?, description = ?, tags_json = ?, due_date = ?, priority = ?, updated_at = ? WHERE id = ?",
        )
        .bind(new_title)
        .bind(new_description)
        .bind(new_tags_json)
        .bind(new_due_date)
        .bind(new_priority)
        .bind(now_rfc3339())
        .bind(card_id.to_string())
        .execute(&self.pool)
        .await?;

        self.touch_board(board_id).await?;

        tracing::info!(card_id = %card_id, board_id = %board_id, "updated card");
        Ok(())
    }

    #[tracing::instrument(skip(self, ordering), fields(card_id = %card_id, to_column_id = %to_column_id, before = ?before_card_id, after = ?after_card_id))]
    pub async fn move_card(
        &self,
        ordering: &OrderingSection,
        card_id: Uuid,
        to_column_id: Uuid,
        before_card_id: Option<Uuid>,
        after_card_id: Option<Uuid>,
    ) -> anyhow::Result<()> {
        let board_id = self.board_id_for_card(card_id).await?;

        let mut left = match after_card_id {
            Some(after_id) => Some(self.position_for_card(after_id).await?),
            None => None,
        };
        let mut right = match before_card_id {
            Some(before_id) => Some(self.position_for_card(before_id).await?),
            None => None,
        };

        if left.is_none() && right.is_none() {
            left = self.last_card_position(to_column_id).await?;
        }

        if let (Some(l), Some(r)) = (left, right) {
            if !positions::gap_ok(ordering, l, r) {
                tracing::warn!(column_id = %to_column_id, "card positions are dense; renormalizing");
                self.renormalize_cards(ordering, to_column_id).await?;

                left = match after_card_id {
                    Some(after_id) => Some(self.position_for_card(after_id).await?),
                    None => None,
                };
                right = match before_card_id {
                    Some(before_id) => Some(self.position_for_card(before_id).await?),
                    None => None,
                };
                if left.is_none() && right.is_none() {
                    left = self.last_card_position(to_column_id).await?;
                }
            }
        }

        let new_pos = positions::between(ordering, left, right);

        sqlx::query("UPDATE cards SET column_id = ?, pos = ?, updated_at = ? WHERE id = ?")
            .bind(to_column_id.to_string())
            .bind(new_pos)
            .bind(now_rfc3339())
            .bind(card_id.to_string())
            .execute(&self.pool)
            .await?;

        self.touch_board(board_id).await?;

        tracing::info!(card_id = %card_id, to_column_id = %to_column_id, board_id = %board_id, pos = new_pos, "moved card");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(card_id = %card_id))]
    pub async fn delete_card(&self, card_id: Uuid) -> anyhow::Result<()> {
        let board_id = self.board_id_for_card(card_id).await?;

        sqlx::query("DELETE FROM cards WHERE id = ?")
            .bind(card_id.to_string())
            .execute(&self.pool)
            .await?;

        self.touch_board(board_id).await?;

        tracing::info!(card_id = %card_id, board_id = %board_id, "deleted card");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(column_id = %column_id))]
    pub async fn delete_column(&self, column_id: Uuid) -> anyhow::Result<()> {
        let board_id = self.board_id_for_column(column_id).await?;

        sqlx::query("DELETE FROM columns WHERE id = ?")
            .bind(column_id.to_string())
            .execute(&self.pool)
            .await?;

        self.touch_board(board_id).await?;

        tracing::info!(column_id = %column_id, board_id = %board_id, "deleted column");
        Ok(())
    }

    async fn renormalize_columns(
        &self,
        ordering: &OrderingSection,
        board_id: Uuid,
    ) -> anyhow::Result<()> {
        let rows = sqlx::query("SELECT id FROM columns WHERE board_id = ? ORDER BY pos ASC")
            .bind(board_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut ids = Vec::with_capacity(rows.len());
        for row in rows {
            ids.push(parse_uuid(row.try_get::<String, _>("id")?)?);
        }

        let updates = positions::renormalize(ordering, &ids);
        let timestamp = now_rfc3339();

        let mut tx = self.pool.begin().await?;
        for (id, pos) in updates {
            sqlx::query("UPDATE columns SET pos = ?, updated_at = ? WHERE id = ?")
                .bind(pos)
                .bind(&timestamp)
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;

        Ok(())
    }

    async fn renormalize_cards(
        &self,
        ordering: &OrderingSection,
        column_id: Uuid,
    ) -> anyhow::Result<()> {
        let rows = sqlx::query("SELECT id FROM cards WHERE column_id = ? ORDER BY pos ASC")
            .bind(column_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut ids = Vec::with_capacity(rows.len());
        for row in rows {
            ids.push(parse_uuid(row.try_get::<String, _>("id")?)?);
        }

        let updates = positions::renormalize(ordering, &ids);
        let timestamp = now_rfc3339();

        let mut tx = self.pool.begin().await?;
        for (id, pos) in updates {
            sqlx::query("UPDATE cards SET pos = ?, updated_at = ? WHERE id = ?")
                .bind(pos)
                .bind(&timestamp)
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;

        Ok(())
    }

    async fn last_column_position(&self, board_id: Uuid) -> anyhow::Result<Option<i64>> {
        let row =
            sqlx::query("SELECT pos FROM columns WHERE board_id = ? ORDER BY pos DESC LIMIT 1")
                .bind(board_id.to_string())
                .fetch_optional(&self.pool)
                .await?;

        row.map(|r| r.try_get::<i64, _>("pos"))
            .transpose()
            .map_err(Into::into)
    }

    async fn last_card_position(&self, column_id: Uuid) -> anyhow::Result<Option<i64>> {
        let row =
            sqlx::query("SELECT pos FROM cards WHERE column_id = ? ORDER BY pos DESC LIMIT 1")
                .bind(column_id.to_string())
                .fetch_optional(&self.pool)
                .await?;

        row.map(|r| r.try_get::<i64, _>("pos"))
            .transpose()
            .map_err(Into::into)
    }

    async fn position_for_column(&self, column_id: Uuid) -> anyhow::Result<i64> {
        let row = sqlx::query("SELECT pos FROM columns WHERE id = ?")
            .bind(column_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => Ok(r.try_get::<i64, _>("pos")?),
            None => bail!("column {column_id} was not found"),
        }
    }

    async fn position_for_card(&self, card_id: Uuid) -> anyhow::Result<i64> {
        let row = sqlx::query("SELECT pos FROM cards WHERE id = ?")
            .bind(card_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => Ok(r.try_get::<i64, _>("pos")?),
            None => bail!("card {card_id} was not found"),
        }
    }

    async fn board_id_for_column(&self, column_id: Uuid) -> anyhow::Result<Uuid> {
        let row = sqlx::query("SELECT board_id FROM columns WHERE id = ?")
            .bind(column_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => parse_uuid(r.try_get::<String, _>("board_id")?),
            None => bail!("column {column_id} was not found"),
        }
    }

    async fn board_id_for_card(&self, card_id: Uuid) -> anyhow::Result<Uuid> {
        let row = sqlx::query("SELECT board_id FROM cards WHERE id = ?")
            .bind(card_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => parse_uuid(r.try_get::<String, _>("board_id")?),
            None => bail!("card {card_id} was not found"),
        }
    }

    async fn touch_board(&self, board_id: Uuid) -> anyhow::Result<()> {
        sqlx::query("UPDATE boards SET updated_at = ? WHERE id = ?")
            .bind(now_rfc3339())
            .bind(board_id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn parse_utc(input: &str) -> anyhow::Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(input)?.with_timezone(&Utc))
}

fn parse_uuid(input: String) -> anyhow::Result<Uuid> {
    Ok(Uuid::parse_str(&input)?)
}

fn normalize_journal_mode(input: &str) -> &'static str {
    match input.to_ascii_uppercase().as_str() {
        "DELETE" => "DELETE",
        "TRUNCATE" => "TRUNCATE",
        "PERSIST" => "PERSIST",
        "MEMORY" => "MEMORY",
        "WAL" => "WAL",
        "OFF" => "OFF",
        _ => "WAL",
    }
}

fn normalize_synchronous(input: &str) -> &'static str {
    match input.to_ascii_uppercase().as_str() {
        "OFF" => "OFF",
        "NORMAL" => "NORMAL",
        "FULL" => "FULL",
        "EXTRA" => "EXTRA",
        _ => "NORMAL",
    }
}
