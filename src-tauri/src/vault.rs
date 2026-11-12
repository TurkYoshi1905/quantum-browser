//! QuantumBrowser — Şifre K Kasası (master parola korumalı)
//! Aralık kapalı: metin açıklamalar belleğin dışında hiçbir zaman saklanmaz.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::password_hash::rand_core::RngCore;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::thread_rng;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::tabs::AppState;

pub struct VaultItem {
    pub id: i64,
    pub host: String,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
    pub has_password_char_hint: bool,
}

#[derive(Serialize, Deserialize)]
struct StoredItem {
    host: String,
    username: String,
    password: String,
    note: String,
}

/// Kasanın şema migrate'i (ayrı dosya: vault.db)
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS vault_meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS vault_items (
            id         INTEGER PRIMARY KEY,
            nonce      BLOB NOT NULL,
            ciphertext BLOB NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        "#,
    )?;
    Ok(())
}

fn open(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).ok();
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn open_vault(path: &Path) -> rusqlite::Result<Connection> {
    open(path)
}

fn derive_key(password: &str, salt_b64: &str, out: &mut [u8; 32]) -> Result<(), String> {
    let salt = B64
        .decode(salt_b64)
        .map_err(|e| format!("tuz bozuk: {e}"))?;
    let params = Params::new(19456, 2, 1, Some(32)).unwrap_or(Params::DEFAULT);
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    argon2
        .hash_password_into(password.as_bytes(), &salt, out)
        .map_err(|e| format!("anahtar türetilemedi: {e}"))
}

fn master_hash_ok(password: &str, phc: &str) -> bool {
    match PasswordHash::new(phc) {
        Ok(hash) => Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok(),
        Err(_) => false,
    }
}

pub struct VaultStatus {
    pub has_master_password: bool,
}

pub fn status(conn: &Connection) -> rusqlite::Result<VaultStatus> {
    let mut stmt = conn.prepare("SELECT value FROM vault_meta WHERE key = 'master_hash'")?;
    let phc: Option<String> = stmt.query_row([], |r| r.get(0)).optional()?;
    Ok(VaultStatus {
        has_master_password: phc.is_some(),
    })
}

pub fn unlocked(state: &AppState) -> bool {
    state.vault.lock().unwrap().is_some()
}

pub fn set_master_password(conn: &Connection, password: &str) -> Result<(), String> {
    let clean = password.trim();
    if clean.len() < 4 {
        return Err("Ana şifre en az 4 karakter olmalı".to_string());
    }
    let mut salt_bytes = [0u8; 16];
    thread_rng().fill_bytes(&mut salt_bytes);
    let salt = B64.encode(salt_bytes);
    let protein = format!("{salt}");
    let salt_string = SaltString::encode_b64(salt_bytes.as_slice())
        .map_err(|e| format!("tuz kodlanamadı: {e}"))?;
    let hash = Argon2::default()
        .hash_password(clean.as_bytes(), &salt_string)
        .map_err(|e| format!("hash üretilemedi: {e}"))?;
    conn.execute(
        "INSERT INTO vault_meta (key, value) VALUES ('master_hash', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![hash.to_string()],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO vault_meta (key, value) VALUES ('salt', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![protein],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn unlock(conn: &Connection, state: &AppState, password: &str) -> Result<bool, String> {
    let clean = password.trim();
    let phc: Option<String> = conn
        .query_row(
            "SELECT value FROM vault_meta WHERE key = 'master_hash'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let Some(phc) = phc else {
        return Err("Kasa henüz oluşturulmadı — önce bir ana şifre belirleyin".to_string());
    };
    if !master_hash_ok(clean, &phc) {
        return Ok(false);
    }
    let salt: Option<String> = conn
        .query_row("SELECT value FROM vault_meta WHERE key = 'salt'", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    let salt = salt.ok_or_else(|| "kasa tuzu bozuk".to_string())?;
    let mut key = [0u8; 32];
    derive_key(clean, &salt, &mut key)?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    *state.vault.lock().unwrap() = Some(cipher);
    Ok(true)
}

pub fn lock(state: &AppState) {
    *state.vault.lock().unwrap() = None;
}

fn decrypt_row(cipher: &Aes256Gcm, nonce: &[u8], ciphertext: &[u8]) -> Result<StoredItem, String> {
    let nonce = Nonce::from_slice(nonce);
    let plain = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("kayıt çözülemedi: {e}"))?;
    serde_json::from_slice(&plain).map_err(|e| format!("kayıt bozuk: {e}"))
}

pub fn list(conn: &Connection, state: &AppState) -> Result<Vec<VaultItem>, String> {
    let mut out = Vec::new();
    {
        let locked = state.vault.lock().unwrap();
        let cipher = locked.as_ref().ok_or_else(|| "Kasa kilitli".to_string())?;
        let mut stmt = conn
            .prepare("SELECT id, nonce, ciphertext, created_at, updated_at FROM vault_items ORDER BY id DESC")
            .map_err(|e| e.to_string())?;
        let rows: Vec<(i64, Vec<u8>, Vec<u8>, String, String)> = stmt
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        for (id, nonce, ciphertext, created_at, updated_at) in rows {
            if let Ok(item) = decrypt_row(cipher, &nonce, &ciphertext) {
                out.push(VaultItem {
                    id,
                    host: item.host,
                    username: item.username,
                    created_at,
                    updated_at,
                    has_password_char_hint: !item.password.is_empty(),
                });
            }
        }
    }
    Ok(out)
}

pub fn find_for_host(
    conn: &Connection,
    state: &AppState,
    host: &str,
) -> Result<Vec<VaultItem>, String> {
    let needle = host.to_lowercase();
    let all = list(conn, state)?;
    Ok(all
        .into_iter()
        .filter(|v| v.host.to_lowercase().contains(&needle))
        .collect())
}

pub fn add(
    conn: &Connection,
    state: &AppState,
    host: &str,
    username: &str,
    password: &str,
    note: &str,
) -> Result<(), String> {
    let h = host.trim().to_string();
    let u = username.trim().to_string();
    if h.is_empty() || password.is_empty() {
        return Err("Site ve şifre zorunludur".to_string());
    }
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce_bytes);
    let item = StoredItem {
        host: h,
        username: u,
        password: password.to_string(),
        note: note.to_string(),
    };
    let plain = serde_json::to_vec(&item).map_err(|e| e.to_string())?;
    let locked = state.vault.lock().unwrap();
    let cipher = locked.as_ref().ok_or_else(|| "Kasa kilitli".to_string())?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plain.as_slice())
        .map_err(|e| format!("şifrelenemedi: {e}"))?;
    conn.execute(
        "INSERT INTO vault_items (nonce, ciphertext) VALUES (?1, ?2)",
        params![&nonce_bytes[..], &ciphertext[..]],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_password(conn: &Connection, state: &AppState, id: i64) -> Result<String, String> {
    let row: (Vec<u8>, Vec<u8>) = conn
        .query_row(
            "SELECT nonce, ciphertext FROM vault_items WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    let locked = state.vault.lock().unwrap();
    let cipher = locked.as_ref().ok_or_else(|| "Kasa kilitli".to_string())?;
    let item = decrypt_row(cipher, &row.0, &row.1)?;
    Ok(item.password)
}

pub fn update(
    conn: &Connection,
    state: &AppState,
    id: i64,
    host: &str,
    username: &str,
    password: &str,
    note: &str,
) -> Result<(), String> {
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce_bytes);
    let item = StoredItem {
        host: host.trim().to_string(),
        username: username.trim().to_string(),
        password: password.to_string(),
        note: note.to_string(),
    };
    let plain = serde_json::to_vec(&item).map_err(|e| e.to_string())?;
    let locked = state.vault.lock().unwrap();
    let cipher = locked.as_ref().ok_or_else(|| "Kasa kilitli".to_string())?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plain.as_slice())
        .map_err(|e| format!("şifrelenemedi: {e}"))?;
    conn.execute(
        "UPDATE vault_items SET nonce = ?1, ciphertext = ?2, updated_at = datetime('now','localtime') WHERE id = ?3",
        params![&nonce_bytes[..], &ciphertext[..], id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete(conn: &Connection, state: &AppState, id: i64) -> Result<(), String> {
    if unlocked(state) {
        conn.execute("DELETE FROM vault_items WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Kasa kilitli".to_string())
    }
}

fn generate_charset() -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(62);
    out.extend(b"abcdefghijklmnopqrstuvwxyz");
    out.extend(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    out.extend(b"0123456789");
    out.extend(b"!@#$%^&*-_=+?");
    out
}

pub fn generate_password(len: usize) -> String {
    let pool = generate_charset();
    let mut rng = rand::thread_rng();
    (0..len.max(8).min(64))
        .map(|_| pool[(rng.next_u32() as usize) % pool.len()] as char)
        .collect()
}

pub fn export(conn: &Connection, state: &AppState) -> Result<String, String> {
    let mut out: Vec<serde_json::Value> = Vec::new();
    {
        let locked = state.vault.lock().unwrap();
        let cipher = locked.as_ref().ok_or_else(|| "Kasa kilitli".to_string())?;
        let mut stmt = conn
            .prepare("SELECT nonce, ciphertext FROM vault_items")
            .map_err(|e| e.to_string())?;
        let rows: Vec<(Vec<u8>, Vec<u8>)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        for (nonce, ciphertext) in rows {
            if let Ok(item) = decrypt_row(cipher, &nonce, &ciphertext) {
                out.push(serde_json::json!({
                    "host": item.host,
                    "username": item.username,
                    "password": item.password,
                    "note": item.note,
                }));
            }
        }
    }
    Ok(serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?)
}

pub fn import(conn: &Connection, state: &AppState, contents: &str) -> Result<usize, String> {
    let parsed: Vec<serde_json::Value> =
        serde_json::from_str(contents).map_err(|e| format!("Yedek dosyası bozuk: {e}"))?;
    let mut count = 0usize;
    for item in parsed {
        let host = item
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let username = item
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let password = item
            .get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let note = item
            .get("note")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if host.is_empty() || password.is_empty() {
            continue;
        }
        add(conn, state, &host, &username, &password, &note)?;
        count += 1;
    }
    Ok(count)
}
