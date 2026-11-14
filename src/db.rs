//! Fase 2.1: Base de datos SQLite de UrquiOffice.
//!
//! Tablas: `settings`, `api_keys` (cifradas con AES-256-GCM), `chat_history`, `recent_files`.
//! Regla de oro: sin `unwrap()`/`expect()`/`panic!`/`unsafe`; todo con `Result`.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand_core::{OsRng, TryRngCore};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Errores de la capa de datos.
#[derive(Debug)]
pub enum DbError {
    Sql(rusqlite::Error),
    Cifrado(String),
    Io(std::io::Error),
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Sql(e) => write!(f, "sqlite: {e}"),
            DbError::Cifrado(m) => write!(f, "cifrado: {m}"),
            DbError::Io(e) => write!(f, "io: {e}"),
        }
    }
}

impl std::error::Error for DbError {}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        DbError::Sql(e)
    }
}

impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self {
        DbError::Io(e)
    }
}

/// Deriva una clave AES-256 desde un secreto maestro (passphrase) con SHA-256.
fn derivar_clave(secreto: &str) -> Key<Aes256Gcm> {
    let mut hasher = Sha256::new();
    hasher.update(secreto.as_bytes());
    let digest = hasher.finalize();
    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&digest);
    Key::<Aes256Gcm>::from(key_bytes)
}

/// Cifra un texto plano con AES-256-GCM. Devuelve nonce(12) || ciphertext+tag.
pub fn cifrar(secreto: &str, plano: &str) -> Result<Vec<u8>, DbError> {
    let key = derivar_clave(secreto);
    let cipher = Aes256Gcm::new(&key);
    let mut nonce_bytes = [0u8; 12];
    OsRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| DbError::Cifrado(format!("rng: {e}")))?;
    let nonce = Nonce::from(nonce_bytes);
    let ct = cipher
        .encrypt(&nonce, plano.as_bytes())
        .map_err(|e| DbError::Cifrado(format!("encrypt: {e}")))?;
    let mut out = Vec::with_capacity(12 + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

/// Descifra datos producidos por [`cifrar`].
pub fn descifrar(secreto: &str, datos: &[u8]) -> Result<String, DbError> {
    if datos.len() < 13 {
        return Err(DbError::Cifrado("datos demasiado cortos".into()));
    }
    let key = derivar_clave(secreto);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&datos[..12]);
    let plano = cipher
        .decrypt(nonce, &datos[12..])
        .map_err(|_| DbError::Cifrado("clave incorrecta o datos corruptos".into()))?;
    String::from_utf8(plano).map_err(|e| DbError::Cifrado(format!("utf8: {e}")))
}

/// Esquema SQL de UrquiOffice.
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS api_keys (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre        TEXT NOT NULL,
    clave_cifrada BLOB NOT NULL,   -- nonce || ciphertext (AES-256-GCM)
    activa        INTEGER NOT NULL DEFAULT 1,
    creada_en     TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS chat_history (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    rol       TEXT NOT NULL,       -- 'user' | 'assistant' | 'system' | 'tool'
    contenido TEXT NOT NULL,
    ts        TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS recent_files (
    path          TEXT PRIMARY KEY,
    ultimo_acceso TEXT NOT NULL DEFAULT (datetime('now'))
);
";

/// Base de datos de UrquiOffice.
pub struct Database {
    conn: Connection,
    secreto: String,
}

impl Database {
    /// Abre (o crea) la base de datos en `path`. `secreto` cifra las claves de API.
    pub fn abrir(path: &Path, secreto: &str) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Database {
            conn,
            secreto: secreto.to_string(),
        })
    }

    /// Base de datos en memoria (para tests).
    pub fn en_memoria(secreto: &str) -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Database {
            conn,
            secreto: secreto.to_string(),
        })
    }

    /// Guarda un valor en settings.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn
            .execute(
                "INSERT INTO settings(key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )?;
        Ok(())
    }

    /// Lee un valor de settings.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get::<_, String>(0));
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sql(e)),
        }
    }

    /// Guarda una clave de API cifrada. Devuelve el id asignado.
    pub fn guardar_api_key(&self, nombre: &str, clave: &str) -> Result<i64, DbError> {
        let cifrada = cifrar(&self.secreto, clave)?;
        self.conn.execute(
            "INSERT INTO api_keys(nombre, clave_cifrada) VALUES (?1, ?2)",
            params![nombre, cifrada],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Lista las claves activas descifradas (para rotación multikey).
    pub fn api_keys_activas(&self) -> Result<Vec<(i64, String, String)>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, nombre, clave_cifrada FROM api_keys WHERE activa = 1 ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, nombre, blob) = r?;
            let clave = descifrar(&self.secreto, &blob)?;
            out.push((id, nombre, clave));
        }
        Ok(out)
    }

    /// Desactiva una clave (rotación tras fallo).
    pub fn desactivar_api_key(&self, id: i64) -> Result<(), DbError> {
        self.conn
            .execute("UPDATE api_keys SET activa = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Añade un mensaje al historial de chat.
    pub fn add_chat(&self, rol: &str, contenido: &str) -> Result<i64, DbError> {
        self.conn.execute(
            "INSERT INTO chat_history(rol, contenido) VALUES (?1, ?2)",
            params![rol, contenido],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Devuelve los últimos `n` mensajes del chat.
    pub fn chat_reciente(&self, n: usize) -> Result<Vec<(String, String)>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT rol, contenido FROM chat_history ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![n as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        out.reverse();
        Ok(out)
    }

    /// Registra un archivo reciente.
    pub fn add_recent_file(&self, path: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO recent_files(path) VALUES (?1)
             ON CONFLICT(path) DO UPDATE SET ultimo_acceso = datetime('now')",
            params![path],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esquema_y_api_keys_cifradas() {
        let db = Database::en_memoria("secreto-de-prueba");
        assert!(
            matches!(db, Ok(_)),
            "la DB en memoria debe abrirse y crear el esquema"
        );
        let db = match db {
            Ok(d) => d,
            Err(_) => return,
        };

        // settings
        assert!(db.set_setting("tema", "oscuro").is_ok());
        let tema = db.get_setting("tema");
        assert!(matches!(tema, Ok(Some(ref v)) if v == "oscuro"), "tema leído: {tema:?}");
        assert!(matches!(db.get_setting("inexistente"), Ok(None)));

        // api keys: se guardan cifradas y se leen descifradas
        let id = db.guardar_api_key("nvidia", "sk-test-123");
        assert!(matches!(id, Ok(_)), "guardar clave: {id:?}");
        let id = match id {
            Ok(i) => i,
            Err(_) => return,
        };
        let keys = db.api_keys_activas();
        match keys {
            Ok(k) if !k.is_empty() => {
                assert_eq!(k[0].2, "sk-test-123", "la clave debe descifrarse igual");
                assert_eq!(k[0].0, id);
            }
            other => panic!("api_keys_activas debería devolver 1 clave: {other:?}"),
        }

        // la clave en disco NO está en claro
        let mut stmt = db.conn.prepare("SELECT clave_cifrada FROM api_keys WHERE id = ?1").ok();
        if let Some(mut s) = stmt.take() {
            let blob: Vec<u8> = s.query_row(params![id], |r| r.get(0)).unwrap_or_default();
            let texto = String::from_utf8_lossy(&blob);
            assert!(
                !texto.contains("sk-test-123"),
                "la clave no debe estar en claro en la DB"
            );
        }

        // rotación: desactivar
        assert!(db.desactivar_api_key(id).is_ok());
        let keys = db.api_keys_activas();
        assert!(matches!(keys, Ok(ref k) if k.is_empty()), "tras desactivar no quedan claves: {keys:?}");

        // chat + recientes
        assert!(db.add_chat("user", "hola").is_ok());
        assert!(db.add_chat("assistant", "qué tal").is_ok());
        let chat = db.chat_reciente(10);
        assert!(matches!(chat, Ok(ref c) if c.len() == 2 && c[0].0 == "user"), "chat: {chat:?}");
        assert!(db.add_recent_file("/tmp/doc.docx").is_ok());
    }

    #[test]
    fn cifrado_roundtrip_y_clave_incorrecta() {
        let ct = cifrar("secreto", "dato sensible");
        assert!(matches!(ct, Ok(_)));
        if let Ok(ct) = ct {
            let ok = descifrar("secreto", &ct);
            assert!(matches!(ok, Ok(ref s) if s == "dato sensible"));
            let mal = descifrar("otro-secreto", &ct);
            assert!(mal.is_err(), "con otra clave debe fallar el descifrado");
        }
    }
}
