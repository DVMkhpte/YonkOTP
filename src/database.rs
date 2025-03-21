use rusqlite::{params, Connection, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose, Engine};

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

/// Chiffre une chaîne avec AES-256-GCM et encode en Base64.
fn encrypt_and_base64(key: &[u8], plaintext: &str) -> String {
    // Attention : nonce fixe pour la démo uniquement !
    let nonce = Nonce::from_slice(b"unique nonce"); // 12 octets
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();
    general_purpose::STANDARD.encode(ciphertext)
}

/// Décode en Base64 et déchiffre avec AES-256-GCM.
fn decrypt_from_base64(key: &[u8], b64_ciphertext: &str) -> String {
    let nonce = Nonce::from_slice(b"unique nonce");
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let ciphertext = general_purpose::STANDARD
        .decode(b64_ciphertext)
        .expect("Base64 invalide");
    let decrypted_data = cipher.decrypt(nonce, ciphertext.as_ref()).unwrap();
    String::from_utf8(decrypted_data).unwrap()
}

/// Insère un enregistrement dans la table `otp_object`.
pub fn insert_otp_object(
    conn: &Connection,
    service: &str,
    u_m: &str,
    secret_key: &str,
    encryption_key: &[u8],
) -> Result<usize> {
    let enc_service = encrypt_and_base64(encryption_key, service);
    let enc_u_m = encrypt_and_base64(encryption_key, u_m);
    let enc_secret_key = encrypt_and_base64(encryption_key, secret_key);

    conn.execute(
        "INSERT INTO otp_object (service, u_m, secret_key) VALUES (?1, ?2, ?3)",
        params![enc_service, enc_u_m, enc_secret_key],
    )
}

/// Récupère tous les enregistrements et renvoie un vecteur de tuples (id, service, u_m).
pub fn select_data(
    conn: &Connection,
    encryption_key: &[u8],
) -> Result<Vec<(i64, String, String)>> {
    let mut stmt = conn.prepare("SELECT id, service, u_m FROM otp_object")?;
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let enc_service: String = row.get(1)?;
        let enc_u_m: String = row.get(2)?;
        
        let service = decrypt_from_base64(encryption_key, &enc_service);
        let u_m = decrypt_from_base64(encryption_key, &enc_u_m);
        
        Ok((id, service, u_m))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn select_data_secret(
    conn: &Connection,
    encryption_key: &[u8],
    id_to_search: i64,
) -> Result<String> {
    let mut stmt = conn.prepare("SELECT secret_key FROM otp_object WHERE id = ?1")?;
    
    let secret_key = stmt.query_row([id_to_search], |row| {
        let enc_secret_key: String = row.get(0)?;
        Ok(decrypt_from_base64(encryption_key, &enc_secret_key))
    }).unwrap_or_else(|_| String::new()); 

    Ok(secret_key)
}
/// Récupère les enregistrements dont le champ déchiffré u_m correspond au filtre donné.
pub fn select_data_cond(
    conn: &Connection,
    u_m_filter: &str,
    encryption_key: &[u8],
) -> Result<Vec<(i64, String, String)>> {
    let mut stmt = conn.prepare("SELECT id, service, u_m FROM otp_object")?;
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let enc_service: String = row.get(1)?;
        let enc_u_m: String = row.get(2)?;
        let service = decrypt_from_base64(encryption_key, &enc_service);
        let u_m = decrypt_from_base64(encryption_key, &enc_u_m);
        Ok((id, service, u_m))
    })?;

    let mut results = Vec::new();
    for row in rows {
        let (id, service, u_m) = row?;
        if u_m == u_m_filter {
            results.push((id, service, u_m));
        }
    }
    Ok(results)
}
