//! Conversations and their messages, stored locally (PRD §15.2).

use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;

use super::sqlite::Db;
use crate::error::{AppError, Result};

const MAX_TITLE_CHARS: usize = 38;
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub persona: String,
    pub grant_id: Option<String>,
    /// Path of the attached folder, if its grant still exists.
    pub folder: Option<String>,
    pub instructions: String,
    pub message_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: i64,
    pub role: String,
    pub text: String,
    pub task_id: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

impl Role {
    fn as_str(self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

const SELECT: &str = "SELECT c.id, c.workspace_id, c.title, c.persona, c.grant_id, g.path, c.instructions,
        (SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id), c.created_at, c.updated_at
     FROM conversations c LEFT JOIN permission_grants g ON g.id = c.grant_id";

pub fn create(db: &Db, project_id: &str, persona: &str) -> Result<Conversation> {
    let id = uuid::Uuid::new_v4().to_string();
    db.with(|c| {
        c.execute(
            "INSERT INTO conversations (id, workspace_id, title, persona) VALUES (?1, ?2, 'A fresh start', ?3)",
            params![id, project_id, persona],
        )
    })?;
    get(db, &id)
}

/// A project's conversations, newest activity first.
pub fn list(db: &Db, project_id: &str) -> Result<Vec<Conversation>> {
    db.with(|c| {
        c.prepare(&format!("{SELECT} WHERE c.workspace_id = ?1 ORDER BY c.updated_at DESC"))?
            .query_map([project_id], from_row)?
            .collect()
    })
}

/// True if any conversation, in any project, still uses this folder grant.
pub fn grant_in_use(db: &Db, grant_id: &str) -> Result<bool> {
    db.with(|c| {
        c.query_row("SELECT EXISTS (SELECT 1 FROM conversations WHERE grant_id = ?1)", [grant_id], |r| r.get(0))
    })
}

pub fn rename(db: &Db, id: &str, title: &str) -> Result<()> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 80 {
        return Err(AppError::Invalid("a conversation name needs 1 to 80 characters".into()));
    }
    db.with(|c| c.execute("UPDATE conversations SET title = ?2 WHERE id = ?1", params![id, title]))?;
    Ok(())
}

pub fn set_persona(db: &Db, id: &str, persona: &str) -> Result<()> {
    db.with(|c| c.execute("UPDATE conversations SET persona = ?2 WHERE id = ?1", params![id, persona]))?;
    Ok(())
}

/// Deletes a conversation and its messages. Task records are kept for the
/// audit trail and undo, just no longer linked to the conversation.
/// Files on disk are never touched.
pub fn delete(db: &Db, id: &str) -> Result<()> {
    db.with(|c| {
        let tx = c.transaction()?;
        delete_in(&tx, "id = ?1", id)?;
        tx.commit()
    })
}

/// Shared by conversation and project deletion.
pub(super) fn delete_in(tx: &rusqlite::Transaction, filter: &str, value: &str) -> rusqlite::Result<()> {
    tx.execute(
        &format!("UPDATE tasks SET conversation_id = NULL WHERE conversation_id IN (SELECT id FROM conversations WHERE {filter})"),
        [value],
    )?;
    tx.execute(&format!("DELETE FROM conversations WHERE {filter}"), [value])?;
    Ok(())
}

pub fn get(db: &Db, id: &str) -> Result<Conversation> {
    db.with(|c| {
        c.query_row(&format!("{SELECT} WHERE c.id = ?1"), [id], from_row)
            .optional()
    })?
    .ok_or_else(|| AppError::NotFound(format!("conversation {id}")))
}

pub fn set_grant(db: &Db, id: &str, grant_id: Option<&str>) -> Result<()> {
    db.with(|c| {
        c.execute(
            &format!("UPDATE conversations SET grant_id = ?2, updated_at = {NOW} WHERE id = ?1"),
            params![id, grant_id],
        )
    })?;
    Ok(())
}

pub fn set_instructions(db: &Db, id: &str, instructions: &str) -> Result<()> {
    db.with(|c| {
        c.execute(
            "UPDATE conversations SET instructions = ?2 WHERE id = ?1",
            params![id, instructions],
        )
    })?;
    Ok(())
}

/// Appends a message. The first user message also becomes the title.
pub fn add_message(
    db: &Db,
    conversation_id: &str,
    role: Role,
    text: &str,
    task_id: Option<&str>,
) -> Result<Message> {
    db.with(|c| {
        let tx = c.transaction()?;
        let first: bool = tx.query_row(
            "SELECT NOT EXISTS (SELECT 1 FROM messages WHERE conversation_id = ?1)",
            [conversation_id],
            |r| r.get(0),
        )?;
        tx.execute(
            "INSERT INTO messages (conversation_id, role, text, task_id) VALUES (?1, ?2, ?3, ?4)",
            params![conversation_id, role.as_str(), text, task_id],
        )?;
        let id = tx.last_insert_rowid();
        if first && role == Role::User {
            tx.execute(
                "UPDATE conversations SET title = ?2 WHERE id = ?1",
                params![conversation_id, title_from(text)],
            )?;
        }
        tx.execute(
            &format!("UPDATE conversations SET updated_at = {NOW} WHERE id = ?1"),
            [conversation_id],
        )?;
        let msg = tx.query_row(
            "SELECT id, role, text, task_id, created_at FROM messages WHERE id = ?1",
            [id],
            message_from_row,
        )?;
        tx.commit()?;
        Ok(msg)
    })
}

pub fn messages(db: &Db, conversation_id: &str) -> Result<Vec<Message>> {
    db.with(|c| {
        c.prepare(
            "SELECT id, role, text, task_id, created_at FROM messages WHERE conversation_id = ?1 ORDER BY id",
        )?
        .query_map([conversation_id], message_from_row)?
        .collect()
    })
}

fn title_from(text: &str) -> String {
    let line = text.lines().next().unwrap_or("").trim();
    let mut title: String = line.chars().take(MAX_TITLE_CHARS).collect();
    if line.chars().count() > MAX_TITLE_CHARS {
        title = format!("{}…", title.trim_end());
    }
    if title.is_empty() {
        "A fresh start".into()
    } else {
        title
    }
}

fn from_row(r: &Row) -> rusqlite::Result<Conversation> {
    Ok(Conversation {
        id: r.get(0)?,
        project_id: r.get(1)?,
        title: r.get(2)?,
        persona: r.get(3)?,
        grant_id: r.get(4)?,
        folder: r.get(5)?,
        instructions: r.get(6)?,
        message_count: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
    })
}

fn message_from_row(r: &Row) -> rusqlite::Result<Message> {
    Ok(Message {
        id: r.get(0)?,
        role: r.get(1)?,
        text: r.get(2)?,
        task_id: r.get(3)?,
        created_at: r.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::repo;

    #[test]
    fn messages_titles_and_folder_grants() {
        let db = Db::open_in_memory().unwrap();
        let conv = create(&db, "default", "ario").unwrap();
        assert_eq!(conv.title, "A fresh start");

        add_message(&db, &conv.id, Role::User, "Help me organize my Downloads folder, please, by type", None).unwrap();
        add_message(&db, &conv.id, Role::Assistant, "Sure", None).unwrap();
        add_message(&db, &conv.id, Role::User, "Second message", None).unwrap();
        let conv = get(&db, &conv.id).unwrap();
        assert_eq!(conv.title, "Help me organize my Downloads folder,…");
        assert_eq!(conv.message_count, 3);
        assert_eq!(messages(&db, &conv.id).unwrap()[1].role, "assistant");

        let grant = repo::upsert_grant(&db, "/Users/x/Downloads").unwrap();
        set_grant(&db, &conv.id, Some(&grant.id)).unwrap();
        assert_eq!(get(&db, &conv.id).unwrap().folder.as_deref(), Some("/Users/x/Downloads"));

        // Revoking the grant detaches it from the conversation.
        repo::delete_grant(&db, &grant.id).unwrap();
        let conv = get(&db, &conv.id).unwrap();
        assert!(conv.grant_id.is_none() && conv.folder.is_none());

        rename(&db, &conv.id, "  Downloads clean-up ").unwrap();
        assert_eq!(get(&db, &conv.id).unwrap().title, "Downloads clean-up");
        assert!(rename(&db, &conv.id, "   ").is_err());
    }

    #[test]
    fn delete_keeps_task_records() {
        let db = Db::open_in_memory().unwrap();
        let conv = create(&db, "default", "ario").unwrap();
        let task = repo::create_task(&db, "organize", "ario", "/tmp/x", Some(&conv.id)).unwrap();
        add_message(&db, &conv.id, Role::Assistant, "plan", Some(&task)).unwrap();
        delete(&db, &conv.id).unwrap();
        assert!(get(&db, &conv.id).is_err());
        assert!(list(&db, "default").unwrap().is_empty());
        assert_eq!(repo::get_task(&db, &task).unwrap().conversation_id, None);
    }
}
