//! QuantumBrowser — SQLite veritabanı katmanı
//! Yer imleri, geçmiş, indirmeler, profiller ve ayarlar burada tutulur.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------- Modeller ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub store_id: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabGroup {
    pub id: i64,
    pub profile_id: i64,
    pub name: String,
    pub color: String,
    pub collapsed: bool,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub profile_id: i64,
    pub url: String,
    pub title: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub profile_id: i64,
    pub url: String,
    pub title: String,
    pub visited_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEntry {
    pub id: i64,
    pub profile_id: i64,
    pub url: String,
    pub file_name: String,
    pub file_path: String,
    pub bytes_received: i64,
    pub bytes_total: i64,
    pub state: String,
    pub started_at: String,
    pub finished_at: Option<String>,
}

// ---------- Kurulum & migrasyon ----------

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TRIGGER IF NOT EXISTS history_ad AFTER DELETE ON history BEGIN
            INSERT INTO history_fts(history_fts, rowid, title, url) VALUES('delete', old.id, old.title, old.url);
        END;
        CREATE TRIGGER IF NOT EXISTS bookmarks_ad AFTER DELETE ON bookmarks BEGIN
            INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, url) VALUES('delete', old.id, old.title, old.url);
        END;

        CREATE TABLE IF NOT EXISTS profiles (
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            color       TEXT NOT NULL DEFAULT '#7c6cf5',
            store_id    TEXT NOT NULL UNIQUE,
            is_default  INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS tab_groups (
            id          INTEGER PRIMARY KEY,
            profile_id  INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            name        TEXT NOT NULL,
            color       TEXT NOT NULL DEFAULT '#7c6cf5',
            collapsed   INTEGER NOT NULL DEFAULT 0,
            position    INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS bookmarks (
            id          INTEGER PRIMARY KEY,
            profile_id  INTEGER NOT NULL,
            url         TEXT NOT NULL,
            title       TEXT NOT NULL DEFAULT '',
            created_at  TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            UNIQUE(profile_id, url)
        );

        CREATE TABLE IF NOT EXISTS history (
            id          INTEGER PRIMARY KEY,
            profile_id  INTEGER NOT NULL,
            url         TEXT NOT NULL,
            title       TEXT NOT NULL DEFAULT '',
            visited_at  TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        CREATE INDEX IF NOT EXISTS idx_history_profile_date ON history(profile_id, visited_at DESC);

        CREATE TABLE IF NOT EXISTS downloads (
            id              INTEGER PRIMARY KEY,
            profile_id      INTEGER NOT NULL,
            url             TEXT NOT NULL,
            file_name       TEXT NOT NULL,
            file_path       TEXT NOT NULL DEFAULT '',
            bytes_received  INTEGER NOT NULL DEFAULT 0,
            bytes_total     INTEGER NOT NULL DEFAULT -1,
            state           TEXT NOT NULL DEFAULT 'downloading',
            started_at      TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            finished_at     TEXT
        );

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS history_fts USING fts5(title, url, content='history', content_rowid='id');
        CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(title, url, content='bookmarks', content_rowid='id');

        CREATE TRIGGER IF NOT EXISTS history_ai AFTER INSERT ON history BEGIN
            INSERT INTO history_fts(rowid, title, url) VALUES (new.id, new.title, new.url);
        END;
        CREATE TRIGGER IF NOT EXISTS bookmarks_ai AFTER INSERT ON bookmarks BEGIN
            INSERT INTO bookmarks_fts(rowid, title, url) VALUES (new.id, new.title, new.url);
        END;
        "#,
    )?;

    // Varsayılan profil
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM profiles", [], |r| r.get(0))?;
    if count == 0 {
        let store_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO profiles (name, color, store_id, is_default) VALUES ('Kişisel', '#7c6cf5', ?1, 1)",
            params![store_id],
        )?;
    }

    // Varsayılan ayarlar
    for (k, v) in [
        ("search_engine", "duckduckgo"),
        ("theme", "light"),
        ("blocker_enabled", "1"),
        ("sidebar_collapsed", "0"),
    ] {
        conn.execute(
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
            params![k, v],
        )?;
    }
    Ok(())
}

// ---------- Profiller ----------

pub fn list_profiles(conn: &Connection) -> rusqlite::Result<Vec<Profile>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, color, store_id, is_default FROM profiles ORDER BY is_default DESC, id ASC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Profile {
            id: r.get(0)?,
            name: r.get(1)?,
            color: r.get(2)?,
            store_id: r.get(3)?,
            is_default: r.get::<_, i64>(4)? == 1,
        })
    })?;
    rows.collect()
}

pub fn default_profile(conn: &Connection) -> rusqlite::Result<Profile> {
    conn.query_row(
        "SELECT id, name, color, store_id, is_default FROM profiles ORDER BY is_default DESC, id ASC LIMIT 1",
        [],
        |r| {
            Ok(Profile {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                store_id: r.get(3)?,
                is_default: r.get::<_, i64>(4)? == 1,
            })
        },
    )
}

pub fn profile_by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<Profile>> {
    conn.query_row(
        "SELECT id, name, color, store_id, is_default FROM profiles WHERE id = ?1",
        params![id],
        |r| {
            Ok(Profile {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                store_id: r.get(3)?,
                is_default: r.get::<_, i64>(4)? == 1,
            })
        },
    )
    .optional()
}

pub fn add_profile(conn: &Connection, name: &str, color: &str) -> rusqlite::Result<i64> {
    let store_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO profiles (name, color, store_id) VALUES (?1, ?2, ?3)",
        params![name, color, store_id],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Profil adını günceller. (Boş isim koruması çağıran tarafta yapılır.)
pub fn rename_profile(conn: &Connection, profile_id: i64, name: &str) -> rusqlite::Result<()> {
    let clean = name.trim();
    conn.execute(
        "UPDATE profiles SET name = ?2 WHERE id = ?1",
        params![profile_id, clean],
    )?;
    Ok(())
}

/// Profili ve ona bağlı tüm verileri siler (yer imleri, geçmiş, indirmeler, sekme grupları).
/// Varsayılan profil silinemez; çağrı yerinde aktif profil kontrolü yapılır.
pub fn delete_profile(conn: &Connection, profile_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM bookmarks WHERE profile_id = ?1",
        params![profile_id],
    )?;
    conn.execute(
        "DELETE FROM history WHERE profile_id = ?1",
        params![profile_id],
    )?;
    conn.execute(
        "DELETE FROM downloads WHERE profile_id = ?1",
        params![profile_id],
    )?;
    conn.execute(
        "DELETE FROM tab_groups WHERE profile_id = ?1",
        params![profile_id],
    )?;
    conn.execute("DELETE FROM profiles WHERE id = ?1", params![profile_id])?;
    Ok(())
}

// ---------- Ayarlar ----------

pub fn get_setting(conn: &Connection, key: &str) -> String {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |r| r.get(0),
    )
    .unwrap_or_else(|_| String::new())
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// ---------- Yer imleri ----------

pub fn add_bookmark(
    conn: &Connection,
    profile_id: i64,
    url: &str,
    title: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO bookmarks (profile_id, url, title) VALUES (?1, ?2, ?3)
         ON CONFLICT(profile_id, url) DO UPDATE SET title = excluded.title",
        params![profile_id, url, title],
    )?;
    Ok(())
}

pub fn remove_bookmark(conn: &Connection, profile_id: i64, url: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM bookmarks WHERE profile_id = ?1 AND url = ?2",
        params![profile_id, url],
    )?;
    Ok(())
}

pub fn is_bookmarked(conn: &Connection, profile_id: i64, url: &str) -> rusqlite::Result<bool> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bookmarks WHERE profile_id = ?1 AND url = ?2",
        params![profile_id, url],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

pub fn list_bookmarks(
    conn: &Connection,
    profile_id: i64,
    query: Option<&str>,
) -> rusqlite::Result<Vec<Bookmark>> {
    let trimmed = query.map(str::trim).filter(|q| !q.is_empty());
    if let Some(q) = trimmed {
        let escaped = format!("\"{}\"*", q.replace('"', "\"\""));
        let mut stmt = conn.prepare(
            "SELECT b.id, b.profile_id, b.url, b.title, b.created_at
             FROM bookmarks_fts f JOIN bookmarks b ON b.id = f.rowid
             WHERE b.profile_id = ?1 AND bookmarks_fts MATCH ?2
             ORDER BY b.created_at DESC LIMIT 200",
        )?;
        let rows = stmt.query_map(params![profile_id, escaped], |r| bookmark_row(r))?;
        rows.collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, url, title, created_at FROM bookmarks
             WHERE profile_id = ?1 ORDER BY created_at DESC LIMIT 200",
        )?;
        let rows = stmt.query_map(params![profile_id], |r| bookmark_row(r))?;
        rows.collect()
    }
}

fn bookmark_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Bookmark> {
    Ok(Bookmark {
        id: r.get(0)?,
        profile_id: r.get(1)?,
        url: r.get(2)?,
        title: r.get(3)?,
        created_at: r.get(4)?,
    })
}

// ---------- Geçmiş ----------

pub fn record_history(
    conn: &Connection,
    profile_id: i64,
    url: &str,
    title: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO history (profile_id, url, title) VALUES (?1, ?2, ?3)",
        params![profile_id, url, title],
    )?;
    Ok(())
}

pub fn update_history_title(conn: &Connection, url: &str, title: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE history SET title = ?2 WHERE id = (
            SELECT id FROM history WHERE url = ?1 ORDER BY visited_at DESC LIMIT 1)",
        params![url, title],
    )?;
    Ok(())
}

pub fn list_history(
    conn: &Connection,
    profile_id: i64,
    query: Option<&str>,
    limit: i64,
) -> rusqlite::Result<Vec<HistoryEntry>> {
    let trimmed = query.map(str::trim).filter(|q| !q.is_empty());
    if let Some(q) = trimmed {
        let escaped = format!("\"{}\"*", q.replace('"', "\"\""));
        let mut stmt = conn.prepare(
            "SELECT h.id, h.profile_id, h.url, h.title, h.visited_at
             FROM history_fts f JOIN history h ON h.id = f.rowid
             WHERE h.profile_id = ?1 AND history_fts MATCH ?2
             ORDER BY h.visited_at DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![profile_id, escaped, limit], |r| history_row(r))?;
        rows.collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, url, title, visited_at FROM history
             WHERE profile_id = ?1 ORDER BY visited_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![profile_id, limit], |r| history_row(r))?;
        rows.collect()
    }
}

fn history_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntry> {
    Ok(HistoryEntry {
        id: r.get(0)?,
        profile_id: r.get(1)?,
        url: r.get(2)?,
        title: r.get(3)?,
        visited_at: r.get(4)?,
    })
}

pub fn clear_history(conn: &Connection, profile_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM history WHERE profile_id = ?1",
        params![profile_id],
    )?;
    Ok(())
}

/// Seçili geçmiş girdilerini mevcut profil kapsamında siler; kaç satırın etkilendiğini döner.
pub fn delete_history_entries(
    conn: &Connection,
    profile_id: i64,
    ids: &[i64],
) -> rusqlite::Result<usize> {
    if ids.is_empty() {
        return Ok(0);
    }
    let placeholders = std::iter::repeat("?")
        .take(ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let mut stmt = conn.prepare(&format!(
        "DELETE FROM history WHERE profile_id = ?1 AND id IN ({placeholders})"
    ))?;
    let mut args: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(ids.len() + 1);
    args.push(&profile_id);
    for id in ids {
        args.push(id);
    }
    stmt.execute(args.as_slice())
}

// ---------- İndirmeler ----------

pub fn insert_download(
    conn: &Connection,
    profile_id: i64,
    url: &str,
    file_name: &str,
    file_path: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO downloads (profile_id, url, file_name, file_path) VALUES (?1, ?2, ?3, ?4)",
        params![profile_id, url, file_name, file_path],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_download(
    conn: &Connection,
    id: i64,
    bytes_received: i64,
    bytes_total: i64,
    state: &str,
    finished: bool,
) -> rusqlite::Result<()> {
    if finished {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "UPDATE downloads SET bytes_received = ?2, bytes_total = ?3, state = ?4, finished_at = ?5 WHERE id = ?1",
            params![id, bytes_received, bytes_total, state, now],
        )?;
    } else {
        conn.execute(
            "UPDATE downloads SET bytes_received = ?2, bytes_total = ?3, state = ?4 WHERE id = ?1",
            params![id, bytes_received, bytes_total, state],
        )?;
    }
    Ok(())
}

pub fn list_downloads(conn: &Connection, profile_id: i64) -> rusqlite::Result<Vec<DownloadEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, url, file_name, file_path, bytes_received, bytes_total, state, started_at, finished_at
         FROM downloads WHERE profile_id = ?1 ORDER BY id DESC LIMIT 100",
    )?;
    let rows = stmt.query_map(params![profile_id], |r| {
        Ok(DownloadEntry {
            id: r.get(0)?,
            profile_id: r.get(1)?,
            url: r.get(2)?,
            file_name: r.get(3)?,
            file_path: r.get(4)?,
            bytes_received: r.get(5)?,
            bytes_total: r.get(6)?,
            state: r.get(7)?,
            started_at: r.get(8)?,
            finished_at: r.get(9)?,
        })
    })?;
    rows.collect()
}

pub fn clear_finished_downloads(conn: &Connection, profile_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM downloads WHERE profile_id = ?1 AND state != 'downloading'",
        params![profile_id],
    )?;
    Ok(())
}

// ---------- Sekme grupları ----------

pub fn list_groups(conn: &Connection, profile_id: i64) -> rusqlite::Result<Vec<TabGroup>> {
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, name, color, collapsed, position FROM tab_groups
         WHERE profile_id = ?1 ORDER BY position ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![profile_id], |r| {
        Ok(TabGroup {
            id: r.get(0)?,
            profile_id: r.get(1)?,
            name: r.get(2)?,
            color: r.get(3)?,
            collapsed: r.get::<_, i64>(4)? == 1,
            position: r.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn create_group(
    conn: &Connection,
    profile_id: i64,
    name: &str,
    color: &str,
) -> rusqlite::Result<i64> {
    let pos: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position), 0) + 1 FROM tab_groups WHERE profile_id = ?1",
        params![profile_id],
        |r| r.get(0),
    )?;
    conn.execute(
        "INSERT INTO tab_groups (profile_id, name, color, position) VALUES (?1, ?2, ?3, ?4)",
        params![profile_id, name, color, pos],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn toggle_group_collapsed(conn: &Connection, group_id: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE tab_groups SET collapsed = 1 - collapsed WHERE id = ?1",
        params![group_id],
    )?;
    Ok(())
}

pub fn delete_group(conn: &Connection, group_id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM tab_groups WHERE id = ?1", params![group_id])?;
    Ok(())
}
