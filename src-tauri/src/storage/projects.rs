//! Projects group conversations (the PRD's workspaces, §14). Stored in the
//! `workspaces` table. The default "Personal" project always exists.

use rusqlite::{params, OptionalExtension, Row};
use serde::Serialize;

use super::conversations;
use super::sqlite::Db;
use crate::error::{AppError, Result};

pub const DEFAULT_PROJECT: &str = "default";
const MAX_NAME_CHARS: usize = 40;
const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub conversation_count: i64,
    pub created_at: String,
}

const SELECT: &str = "SELECT w.id, w.name, COALESCE(w.description, ''),
        (SELECT COUNT(*) FROM conversations c WHERE c.workspace_id = w.id), w.created_at
     FROM workspaces w";

/// The default project first, then by name.
pub fn list(db: &Db) -> Result<Vec<Project>> {
    db.with(|c| {
        c.prepare(&format!("{SELECT} ORDER BY w.id != 'default', w.name COLLATE NOCASE"))?
            .query_map([], from_row)?
            .collect()
    })
}

pub fn get(db: &Db, id: &str) -> Result<Project> {
    db.with(|c| c.query_row(&format!("{SELECT} WHERE w.id = ?1"), [id], from_row).optional())?
        .ok_or_else(|| AppError::NotFound(format!("project {id}")))
}

pub fn create(db: &Db, name: &str, description: &str) -> Result<Project> {
    let name = valid_name(name)?;
    let id = uuid::Uuid::new_v4().to_string();
    db.with(|c| {
        c.execute(
            "INSERT INTO workspaces (id, name, description) VALUES (?1, ?2, ?3)",
            params![id, name, description.trim()],
        )
    })?;
    get(db, &id)
}

pub fn update(db: &Db, id: &str, name: &str, description: &str) -> Result<Project> {
    let name = valid_name(name)?;
    db.with(|c| {
        c.execute(
            &format!("UPDATE workspaces SET name = ?2, description = ?3, updated_at = {NOW} WHERE id = ?1"),
            params![id, name, description.trim()],
        )
    })?;
    get(db, id)
}

/// Deletes a project and its conversations. Files on disk are never touched,
/// and task records stay for the audit trail.
pub fn delete(db: &Db, id: &str) -> Result<()> {
    if id == DEFAULT_PROJECT {
        return Err(AppError::Invalid("the Personal project can be renamed but not deleted".into()));
    }
    db.with(|c| {
        let tx = c.transaction()?;
        conversations::delete_in(&tx, "workspace_id = ?1", id)?;
        tx.execute("DELETE FROM workspaces WHERE id = ?1", [id])?;
        tx.commit()
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
        created_at: r.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::conversations::{self, Role};

    #[test]
    fn create_rename_delete() {
        let db = Db::open_in_memory().unwrap();
        let p = create(&db, " Thesis ", "MSc work").unwrap();
        assert_eq!(p.name, "Thesis");
        let conv = conversations::create(&db, &p.id, "suf-4").unwrap();
        conversations::add_message(&db, &conv.id, Role::User, "hello", None).unwrap();
        assert_eq!(get(&db, &p.id).unwrap().conversation_count, 1);

        let names: Vec<_> = list(&db).unwrap().into_iter().map(|p| p.name).collect();
        assert_eq!(names, ["Personal", "Thesis"]);

        assert_eq!(update(&db, &p.id, "Thesis 2026", "").unwrap().name, "Thesis 2026");
        assert!(create(&db, "  ", "").is_err());
        assert!(delete(&db, DEFAULT_PROJECT).is_err());

        delete(&db, &p.id).unwrap();
        assert!(get(&db, &p.id).is_err());
        assert!(conversations::get(&db, &conv.id).is_err());
    }
}
