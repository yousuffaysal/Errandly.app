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
    /// Individually attached files whose grants still exist.
    pub files: Vec<String>,
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
    /// A result card (document summary or spreadsheet report), if any.
    pub card: Option<serde_json::Value>,
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
        (SELECT COUNT(*) FROM messages m WHERE m.conversation_id = c.id), c.created_at, c.updated_at,
        (SELECT group_concat(f.path, char(31)) FROM conversation_files f
           JOIN permission_grants fg ON fg.path = f.path WHERE f.conversation_id = c.id)
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

/// Deletes every conversation in every project of `owner`; task records stay.
pub fn delete_all(db: &Db, owner: &str) -> Result<()> {
    db.with(|c| {
        let tx = c.transaction()?;
        delete_in(&tx, "workspace_id IN (SELECT id FROM workspaces WHERE owner = ?1)", owner)?;
        tx.commit()
    })
}

/// The owner of the project a conversation lives in.
pub fn owner_of(db: &Db, id: &str) -> Result<Option<String>> {
    db.with(|c| {
        c.query_row(
            "SELECT w.owner FROM conversations c JOIN workspaces w ON w.id = c.workspace_id WHERE c.id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()
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
    add_message_with_card(db, conversation_id, role, text, task_id, None)
}

pub fn add_message_with_card(
    db: &Db,
    conversation_id: &str,
    role: Role,
    text: &str,
    task_id: Option<&str>,
    card: Option<&serde_json::Value>,
) -> Result<Message> {
    let card = card.map(|c| c.to_string());
    db.with(|c| {
        let tx = c.transaction()?;
        let first: bool = tx.query_row(
            "SELECT NOT EXISTS (SELECT 1 FROM messages WHERE conversation_id = ?1)",
            [conversation_id],
            |r| r.get(0),
        )?;
        tx.execute(
            "INSERT INTO messages (conversation_id, role, text, task_id, card) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![conversation_id, role.as_str(), text, task_id, card],
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
            "SELECT id, role, text, task_id, created_at, card FROM messages WHERE id = ?1",
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
            "SELECT id, role, text, task_id, created_at, card FROM messages WHERE conversation_id = ?1 ORDER BY id",
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
        files: r
            .get::<_, Option<String>>(10)?
            .map(|s| s.split('\u{1f}').map(str::to_owned).collect())
            .unwrap_or_default(),
    })
}

/// Attaches a file (already granted) to a conversation.
pub fn attach_file(db: &Db, id: &str, path: &str) -> Result<()> {
    db.with(|c| {
        c.execute("INSERT OR IGNORE INTO conversation_files (conversation_id, path) VALUES (?1, ?2)", params![id, path])?;
        c.execute(&format!("UPDATE conversations SET updated_at = {NOW} WHERE id = ?1"), [id])
    })?;
    Ok(())
}

pub fn detach_file(db: &Db, id: &str, path: &str) -> Result<()> {
    db.with(|c| c.execute("DELETE FROM conversation_files WHERE conversation_id = ?1 AND path = ?2", params![id, path]))?;
    Ok(())
}

/// True if any conversation still has this file attached.
pub fn file_in_use(db: &Db, path: &str) -> Result<bool> {
    db.with(|c| c.query_row("SELECT EXISTS (SELECT 1 FROM conversation_files WHERE path = ?1)", [path], |r| r.get(0)))
}

fn message_from_row(r: &Row) -> rusqlite::Result<Message> {
    Ok(Message {
        id: r.get(0)?,
        role: r.get(1)?,
        text: r.get(2)?,
        task_id: r.get(3)?,
        created_at: r.get(4)?,
        card: r.get::<_, Option<String>>(5)?.and_then(|c| serde_json::from_str(&c).ok()),
    })
}

/// Removes the messages after `message_id` (for answering again).
pub fn delete_after(db: &Db, id: &str, message_id: i64) -> Result<()> {
    db.with(|c| c.execute("DELETE FROM messages WHERE conversation_id = ?1 AND id > ?2", params![id, message_id]))?;
    Ok(())
}

/// A message and the conversation it belongs to.
pub fn message(db: &Db, message_id: i64) -> Result<Option<(String, Message)>> {
    db.with(|c| {
        c.query_row(
            "SELECT conversation_id, id, role, text, task_id, created_at, card FROM messages WHERE id = ?1",
            [message_id],
            |r| {
                Ok((
                    r.get(0)?,
                    Message {
                        id: r.get(1)?,
                        role: r.get(2)?,
                        text: r.get(3)?,
                        task_id: r.get(4)?,
                        created_at: r.get(5)?,
                        card: r.get::<_, Option<String>>(6)?.and_then(|c| serde_json::from_str(&c).ok()),
                    },
                ))
            },
        )
        .optional()
    })
}

/// A message's card and the conversation it belongs to, for exporting.
pub fn card(db: &Db, message_id: i64) -> Result<Option<(String, serde_json::Value)>> {
    let row: Option<(String, Option<String>)> = db.with(|c| {
        c.query_row("SELECT conversation_id, card FROM messages WHERE id = ?1", [message_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()
    })?;
    Ok(row.and_then(|(conv, card)| Some((conv, serde_json::from_str(&card?).ok()?))))
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
    fn files_attach_and_follow_their_grants() {
        let db = Db::open_in_memory().unwrap();
        let conv = create(&db, "default", "ario").unwrap();
        let g = repo::upsert_grant(&db, "/Users/x/Documents/thesis.pdf").unwrap();
        attach_file(&db, &conv.id, &g.path).unwrap();
        attach_file(&db, &conv.id, &g.path).unwrap();
        assert_eq!(get(&db, &conv.id).unwrap().files, vec!["/Users/x/Documents/thesis.pdf"]);
        assert!(file_in_use(&db, &g.path).unwrap());
        repo::delete_grant(&db, &g.id).unwrap();
        assert!(get(&db, &conv.id).unwrap().files.is_empty(), "revoking access hides the file");
        let g = repo::upsert_grant(&db, "/Users/x/a.txt").unwrap();
        attach_file(&db, &conv.id, &g.path).unwrap();
        detach_file(&db, &conv.id, &g.path).unwrap();
        assert!(get(&db, &conv.id).unwrap().files.is_empty());
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
