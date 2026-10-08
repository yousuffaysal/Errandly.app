//! Projects group conversations (the PRD's workspaces, §14). Stored in the
//! `workspaces` table. The default "Personal" project always exists.

use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;

use super::conversations;
use super::sqlite::Db;
use crate::error::{AppError, Result};

const MAX_NAME_CHARS: usize = 40;
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub conversation_count: i64,
    /// The owner's "Personal" project: can be renamed, never deleted.
    pub is_default: bool,
    pub created_at: String,
}

const SELECT: &str = "SELECT w.id, w.name, COALESCE(w.description, ''),
        (SELECT COUNT(*) FROM conversations c WHERE c.workspace_id = w.id), w.is_default, w.created_at
     FROM workspaces w";

/// The owner's projects: their default first, then by name. Creates the
/// default "Personal" project the first time an owner is seen.
pub fn list(db: &Db, owner: &str) -> Result<Vec<Project>> {
    ensure_default(db, owner)?;
    db.with(|c| {
        c.prepare(&format!("{SELECT} WHERE w.owner = ?1 ORDER BY w.is_default DESC, w.name COLLATE NOCASE"))?
            .query_map([owner], from_row)?
            .collect()
    })
}

fn ensure_default(db: &Db, owner: &str) -> Result<()> {
    db.with(|c| {
        let exists: bool = c.query_row(
            "SELECT EXISTS (SELECT 1 FROM workspaces WHERE owner = ?1 AND is_default = 1)",
            [owner],
            |r| r.get(0),
        )?;
        if !exists {
            c.execute(
                "INSERT INTO workspaces (id, name, description, owner, is_default) VALUES (?1, 'Personal', '', ?2, 1)",
                params![uuid::Uuid::new_v4().to_string(), owner],
            )?;
        }
        Ok(())
    })
}

/// A project, only if it belongs to `owner`.
pub fn get(db: &Db, owner: &str, id: &str) -> Result<Project> {
    db.with(|c| c.query_row(&format!("{SELECT} WHERE w.id = ?1 AND w.owner = ?2"), params![id, owner], from_row).optional())?
        .ok_or_else(|| AppError::NotFound("project".into()))
}

pub fn create(db: &Db, owner: &str, name: &str, description: &str) -> Result<Project> {
    let name = valid_name(name)?;
    let id = uuid::Uuid::new_v4().to_string();
    db.with(|c| {
        c.execute(
            "INSERT INTO workspaces (id, name, description, owner) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, description.trim(), owner],
        )
    })?;
    get(db, owner, &id)
}

pub fn update(db: &Db, owner: &str, id: &str, name: &str, description: &str) -> Result<Project> {
    let name = valid_name(name)?;
    get(db, owner, id)?;
    db.with(|c| {
        c.execute(
            &format!("UPDATE workspaces SET name = ?2, description = ?3, updated_at = {NOW} WHERE id = ?1"),
            params![id, name, description.trim()],
        )
    })?;
    get(db, owner, id)
}

/// Deletes a project and its conversations. Files on disk are never touched,
/// and task records stay for the audit trail.
pub fn delete(db: &Db, owner: &str, id: &str) -> Result<()> {
    if get(db, owner, id)?.is_default {
        return Err(AppError::Invalid("the Personal project can be renamed but not deleted".into()));
    }
    db.with(|c| {
        let tx = c.transaction()?;
        conversations::delete_in(&tx, "workspace_id = ?1", id)?;
        tx.execute("DELETE FROM workspaces WHERE id = ?1", [id])?;
        tx.commit()
    })
}

/// Ids of a project's conversations, to check nothing is still running in it.
pub fn conversation_ids(db: &Db, id: &str) -> Result<Vec<String>> {
    db.with(|c| {
        c.prepare("SELECT id FROM conversations WHERE workspace_id = ?1")?
            .query_map([id], |r| r.get(0))?
            .collect()
    })
}

fn valid_name(name: &str) -> Result<&str> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
        return Err(AppError::Invalid(format!("a project name needs 1 to {MAX_NAME_CHARS} characters")));
    }
    Ok(name)
}

fn from_row(r: &Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: r.get(0)?,
        name: r.get(1)?,
        description: r.get(2)?,
        conversation_count: r.get(3)?,
        is_default: r.get(4)?,
        created_at: r.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::conversations::{self, Role};

    #[test]
    fn create_rename_delete() {
        let db = Db::open_in_memory().unwrap();
        let p = create(&db, "local", " Thesis ", "MSc work").unwrap();
        assert_eq!(p.name, "Thesis");
        let conv = conversations::create(&db, &p.id, "suf-4").unwrap();
        conversations::add_message(&db, &conv.id, Role::User, "hello", None).unwrap();
        assert_eq!(get(&db, "local", &p.id).unwrap().conversation_count, 1);

        let all = list(&db, "local").unwrap();
        assert_eq!(all.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["Personal", "Thesis"]);
        assert!(all[0].is_default);

        assert_eq!(update(&db, "local", &p.id, "Thesis 2026", "").unwrap().name, "Thesis 2026");
        assert!(create(&db, "local", "  ", "").is_err());
        assert!(delete(&db, "local", &all[0].id).is_err(), "the default project stays");

        delete(&db, "local", &p.id).unwrap();
        assert!(get(&db, "local", &p.id).is_err());
        assert!(conversations::get(&db, &conv.id).is_err());
    }

    #[test]
    fn owners_only_see_their_own_projects() {
        let db = Db::open_in_memory().unwrap();
        let mine = create(&db, "acct-a", "Mine", "").unwrap();
        let b = list(&db, "acct-b").unwrap();
        assert_eq!(b.len(), 1, "B only has its own Personal project");
        assert!(b[0].is_default && b[0].id != "default");
        assert!(get(&db, "acct-b", &mine.id).is_err());
        assert!(update(&db, "acct-b", &mine.id, "Stolen", "").is_err());
        assert!(delete(&db, "acct-b", &mine.id).is_err());
        assert_eq!(list(&db, "local").unwrap()[0].id, "default", "the original local project is kept");
    }
}
