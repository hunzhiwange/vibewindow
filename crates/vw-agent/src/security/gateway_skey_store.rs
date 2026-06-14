//! SQLite store for gateway skey metadata.

use std::path::{Path, PathBuf};
use vw_config_types::gateway::GatewaySkey;

#[cfg(not(target_arch = "wasm32"))]
use rusqlite::{Connection, params};
#[cfg(not(target_arch = "wasm32"))]
use sha2::{Digest, Sha256};

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from).filter(|path| !path.as_os_str().is_empty())
}

fn config_file_dir_from_env() -> Option<PathBuf> {
    env_path("VIBEWINDOW_CONFIG")
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .filter(|path| !path.as_os_str().is_empty())
}

fn gateway_skey_config_dir() -> PathBuf {
    config_file_dir_from_env()
        .or_else(|| env_path("VIBEWINDOW_CONFIG_DIR"))
        .unwrap_or_else(|| vw_config_types::paths::home_config_dir(&crate::global::paths().home))
}

fn gateway_skey_db_path_for_config_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("gateway").join("skeys.sqlite")
}

pub fn gateway_skey_db_path() -> PathBuf {
    gateway_skey_db_path_for_config_dir(&gateway_skey_config_dir())
}

fn legacy_gateway_skey_db_path() -> PathBuf {
    crate::global::paths().data.join("gateway").join("skeys.sqlite")
}

#[cfg(not(target_arch = "wasm32"))]
fn sql_error(error: rusqlite::Error) -> String {
    format!("gateway skey sqlite error: {error}")
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS gateway_skeys (
            skey_hash TEXT PRIMARY KEY NOT NULL,
            masked_skey TEXT NOT NULL DEFAULT '',
            name TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            expires_at TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .map_err(sql_error)?;

    let has_masked_skey = conn
        .prepare("PRAGMA table_info(gateway_skeys)")
        .map_err(sql_error)?
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(sql_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_error)?
        .iter()
        .any(|column| column == "masked_skey");
    if !has_masked_skey {
        conn.execute(
            "ALTER TABLE gateway_skeys ADD COLUMN masked_skey TEXT NOT NULL DEFAULT ''",
            [],
        )
        .map_err(sql_error)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn hash_skey(raw_skey: &str) -> String {
    format!("{:x}", Sha256::digest(raw_skey.trim().as_bytes()))
}

#[cfg(not(target_arch = "wasm32"))]
fn mask_skey_for_display(raw_skey: &str) -> String {
    let trimmed = raw_skey.trim();
    let chars = trimmed.chars().collect::<Vec<_>>();
    if chars.len() <= 25 {
        return trimmed.to_string();
    }
    let prefix = chars.iter().take(16).collect::<String>();
    let suffix =
        chars.iter().rev().take(9).collect::<Vec<_>>().into_iter().rev().collect::<String>();
    format!("{prefix}{}{suffix}", "*".repeat(15))
}

#[cfg(not(target_arch = "wasm32"))]
fn persistable_skey_hash(skey: &GatewaySkey) -> Option<String> {
    let configured_hash = skey.skey_hash.trim();
    if !configured_hash.is_empty() {
        return Some(configured_hash.to_ascii_lowercase());
    }
    skey.skey.as_deref().map(str::trim).filter(|raw_skey| !raw_skey.is_empty()).map(hash_skey)
}

#[cfg(not(target_arch = "wasm32"))]
fn persistable_masked_skey(skey: &GatewaySkey) -> String {
    let configured_mask = skey.masked_skey.trim();
    if !configured_mask.is_empty() {
        return configured_mask.to_string();
    }
    skey.skey
        .as_deref()
        .map(str::trim)
        .filter(|raw_skey| !raw_skey.is_empty())
        .map(mask_skey_for_display)
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_gateway_skeys_from_path(db_path: &Path) -> Result<Vec<GatewaySkey>, String> {
    let Some(parent) = db_path.parent() else {
        return Err("gateway skey sqlite path has no parent".to_string());
    };
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let conn = Connection::open(db_path).map_err(sql_error)?;
    ensure_schema(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT enabled, skey_hash, masked_skey, name, expires_at \
             FROM gateway_skeys \
             ORDER BY created_at ASC, rowid ASC",
        )
        .map_err(sql_error)?;
    let rows = stmt
        .query_map([], |row| {
            Ok(GatewaySkey {
                enabled: row.get::<_, i64>(0)? != 0,
                skey: None,
                skey_hash: row.get(1)?,
                masked_skey: row.get(2)?,
                name: row.get(3)?,
                expires_at: row.get(4)?,
            })
        })
        .map_err(sql_error)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(sql_error)
}

#[cfg(target_arch = "wasm32")]
pub fn load_gateway_skeys_from_path(_db_path: &Path) -> Result<Vec<GatewaySkey>, String> {
    Ok(Vec::new())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_existing_gateway_skeys_from_path(
    db_path: &Path,
) -> Result<Option<Vec<GatewaySkey>>, String> {
    if !db_path.exists() {
        return Ok(None);
    }
    load_gateway_skeys_from_path(db_path).map(Some)
}

#[cfg(target_arch = "wasm32")]
pub fn load_existing_gateway_skeys_from_path(
    _db_path: &Path,
) -> Result<Option<Vec<GatewaySkey>>, String> {
    Ok(None)
}

fn load_existing_gateway_skeys_from_paths(
    primary_path: &Path,
    legacy_path: &Path,
) -> Result<Option<Vec<GatewaySkey>>, String> {
    if let Some(skeys) = load_existing_gateway_skeys_from_path(primary_path)? {
        return Ok(Some(skeys));
    }
    if legacy_path != primary_path {
        return load_existing_gateway_skeys_from_path(legacy_path);
    }
    Ok(None)
}

pub fn load_existing_gateway_skeys() -> Result<Option<Vec<GatewaySkey>>, String> {
    load_existing_gateway_skeys_from_paths(&gateway_skey_db_path(), &legacy_gateway_skey_db_path())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_gateway_skeys_to_path(db_path: &Path, skeys: &[GatewaySkey]) -> Result<(), String> {
    let Some(parent) = db_path.parent() else {
        return Err("gateway skey sqlite path has no parent".to_string());
    };
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut conn = Connection::open(db_path).map_err(sql_error)?;
    ensure_schema(&conn)?;

    let tx = conn.transaction().map_err(sql_error)?;
    tx.execute("DELETE FROM gateway_skeys", []).map_err(sql_error)?;
    for skey in skeys {
        let Some(skey_hash) = persistable_skey_hash(skey) else {
            continue;
        };
        let masked_skey = persistable_masked_skey(skey);
        tx.execute(
            "INSERT INTO gateway_skeys (enabled, skey_hash, masked_skey, name, expires_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)",
            params![
                if skey.enabled { 1_i64 } else { 0_i64 },
                skey_hash,
                masked_skey,
                skey.name.trim(),
                skey.expires_at.as_deref().map(str::trim).filter(|value| !value.is_empty()),
            ],
        )
        .map_err(sql_error)?;
    }
    tx.commit().map_err(sql_error)
}

#[cfg(target_arch = "wasm32")]
pub fn save_gateway_skeys_to_path(_db_path: &Path, _skeys: &[GatewaySkey]) -> Result<(), String> {
    Ok(())
}

pub fn save_gateway_skeys(skeys: &[GatewaySkey]) -> Result<(), String> {
    save_gateway_skeys_to_path(&gateway_skey_db_path(), skeys)
}

#[cfg(test)]
#[path = "gateway_skey_store_tests.rs"]
mod gateway_skey_store_tests;
