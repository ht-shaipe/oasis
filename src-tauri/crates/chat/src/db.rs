use rusqlite::{params, Connection, Result};
use std::path::Path;

use crate::models::{
    Conversation, ConversationDetail, ConversationListItem, Message, NewConversation, NewMessage,
    UpdateMessageContent,
};

pub fn init_db(app_data_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(app_data_dir).ok();
    let db_path = app_data_dir.join("chat.db");
    let conn = Connection::open(db_path)?;

    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS conversations (
            id          TEXT PRIMARY KEY,
            title       TEXT NOT NULL DEFAULT '新对话',
            model_id    TEXT NOT NULL DEFAULT '',
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id                  TEXT PRIMARY KEY,
            conversation_id     TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
            role                TEXT NOT NULL,
            content             TEXT NOT NULL DEFAULT '',
            reasoning_content   TEXT,
            token_usage         TEXT,
            created_at          TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);
        CREATE INDEX IF NOT EXISTS idx_conversations_updated ON conversations(updated_at DESC);
        ",
    )?;

    Ok(conn)
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| format!("{}", secs))
}

pub fn list_conversations(conn: &Connection) -> Result<Vec<ConversationListItem>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.title, c.model_id, c.created_at, c.updated_at,
                (SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id) as msg_count
         FROM conversations c
         ORDER BY c.updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ConversationListItem {
            id: row.get(0)?,
            title: row.get(1)?,
            model_id: row.get(2)?,
            message_count: row.get(5)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn get_conversation(conn: &Connection, id: &str) -> Result<ConversationDetail> {
    let conv: Conversation = conn.query_row(
        "SELECT id, title, model_id, created_at, updated_at FROM conversations WHERE id = ?1",
        params![id],
        |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                model_id: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )?;

    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, reasoning_content, token_usage, created_at
         FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map(params![id], |row| {
        Ok(Message {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            reasoning_content: row.get(4)?,
            token_usage: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;
    let messages: Vec<Message> = rows.filter_map(|r| r.ok()).collect();

    Ok(ConversationDetail {
        conversation: conv,
        messages,
    })
}

pub fn create_conversation(conn: &Connection, input: &NewConversation) -> Result<Conversation> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO conversations (id, title, model_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![input.id, input.title, input.model_id, now],
    )?;
    Ok(Conversation {
        id: input.id.clone(),
        title: input.title.clone(),
        model_id: input.model_id.clone(),
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn save_message(conn: &Connection, msg: &NewMessage) -> Result<Message> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, reasoning_content, token_usage, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            msg.id,
            msg.conversation_id,
            msg.role,
            msg.content,
            msg.reasoning_content,
            msg.token_usage,
            now,
        ],
    )?;
    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![now, msg.conversation_id],
    )?;
    Ok(Message {
        id: msg.id.clone(),
        conversation_id: msg.conversation_id.clone(),
        role: msg.role.clone(),
        content: msg.content.clone(),
        reasoning_content: msg.reasoning_content.clone(),
        token_usage: msg.token_usage.clone(),
        created_at: now,
    })
}

pub fn save_messages(conn: &Connection, msgs: &[NewMessage]) -> Result<Vec<Message>> {
    let mut result = Vec::new();
    let now = now_iso();
    for msg in msgs {
        conn.execute(
            "INSERT INTO messages (id, conversation_id, role, content, reasoning_content, token_usage, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                msg.id,
                msg.conversation_id,
                msg.role,
                msg.content,
                msg.reasoning_content,
                msg.token_usage,
                now,
            ],
        )?;
        result.push(Message {
            id: msg.id.clone(),
            conversation_id: msg.conversation_id.clone(),
            role: msg.role.clone(),
            content: msg.content.clone(),
            reasoning_content: msg.reasoning_content.clone(),
            token_usage: msg.token_usage.clone(),
            created_at: now.clone(),
        });
    }
    if let Some(first) = msgs.first() {
        conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![now, first.conversation_id],
        )?;
    }
    Ok(result)
}

pub fn update_conversation_title(conn: &Connection, id: &str, title: &str) -> Result<()> {
    let now = now_iso();
    conn.execute(
        "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )?;
    Ok(())
}

pub fn delete_conversation(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn update_message_content(conn: &Connection, input: &UpdateMessageContent) -> Result<()> {
    conn.execute(
        "UPDATE messages SET content = ?1, reasoning_content = ?2 WHERE id = ?3",
        params![input.content, input.reasoning_content, input.id],
    )?;
    Ok(())
}
