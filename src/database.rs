use rusqlite::{params, Connection, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose, Engine};
use csv::Writer;

/// Crée la table si nécessaire.
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS otp_object (
            id          INTEGER PRIMARY KEY,
            service     TEXT NOT NULL,
            u_m         TEXT NOT NULL,
            secret_key  TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn encrypt_and_base64(key: &[u8], plaintext: &str) -> String {
    let nonce = Nonce::from_slice(b"unique nonce");
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let ct = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();
    general_purpose::STANDARD.encode(ct)
}

fn decrypt_from_base64(key: &[u8], b64: &str) -> String {
    let nonce = Nonce::from_slice(b"unique nonce");
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let ct = general_purpose::STANDARD.decode(b64).unwrap();
    let pt = cipher.decrypt(nonce, ct.as_ref()).unwrap();
    String::from_utf8(pt).unwrap()
}

pub fn insert_otp_object(
    conn: &Connection,
    service: &str,
    u_m: &str,
    secret_key: &str,
    encryption_key: &[u8],
) -> Result<usize> {
    let s = encrypt_and_base64(encryption_key, service);
    let u = encrypt_and_base64(encryption_key, u_m);
    let k = encrypt_and_base64(encryption_key, secret_key);
    conn.execute(
        "INSERT INTO otp_object (service, u_m, secret_key) VALUES (?1, ?2, ?3)",
        params![s, u, k],
    )
}

pub fn update_otp_object(
    conn: &Connection,
    id: i64,
    new_service: &str,
    new_u_m: &str,
    encryption_key: &[u8],
) -> Result<usize> {
    let s = encrypt_and_base64(encryption_key, new_service);
    let u = encrypt_and_base64(encryption_key, new_u_m);
    conn.execute(
        "UPDATE otp_object SET service = ?1, u_m = ?2 WHERE id = ?3",
        params![s, u, id],
    )
}

pub fn delete_otp_object(conn: &Connection, id: i64) -> Result<usize> {
    conn.execute("DELETE FROM otp_object WHERE id = ?1", params![id])
}

pub fn select_data(
    conn: &Connection,
    encryption_key: &[u8],
) -> Result<Vec<(i64, String, String)>> {
    let mut stmt = conn.prepare("SELECT id, service, u_m FROM otp_object")?;
    let mut rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let s: String = row.get(1)?;
        let u: String = row.get(2)?;
        Ok((
            id,
            decrypt_from_base64(encryption_key, &s),
            decrypt_from_base64(encryption_key, &u),
        ))
    })?;

    let mut out = Vec::new();
    while let Some(row_res) = rows.next() {
        out.push(row_res?);
    }
    Ok(out)
}

pub fn select_data_secret(
    conn: &Connection,
    encryption_key: &[u8],
    id_to_search: i64,
) -> Result<String> {
    let enc: String = conn
        .prepare("SELECT secret_key FROM otp_object WHERE id = ?1")?
        .query_row([id_to_search], |r| r.get(0))?;
    Ok(decrypt_from_base64(encryption_key, &enc))
}

pub fn export_to_csv(
    conn: &Connection,
    key: &[u8],
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = Writer::from_path(file_path)?;
    wtr.write_record(&["id", "service", "username/mail", "secret_key"])?;
    let mut stmt = conn.prepare("SELECT id, service, u_m, secret_key FROM otp_object")?;
    let mut rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let s: String = row.get(1)?;
        let u: String = row.get(2)?;
        let k: String = row.get(3)?;
        Ok((
            id.to_string(),
            decrypt_from_base64(key, &s),
            decrypt_from_base64(key, &u),
            decrypt_from_base64(key, &k),
        ))
    })?;

    while let Some(rec) = rows.next() {
        let (id, s, u, k) = rec?;
        wtr.write_record(&[&id, &s, &u, &k])?;
    }
    wtr.flush()?;
    Ok(())
}
