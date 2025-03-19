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

/// Chiffre une chaîne en AES-256-GCM et encode le résultat en Base64.
fn encrypt_and_base64(key: &[u8], plaintext: &str) -> String {
    // Attention : nonce fixe pour la démo uniquement !
    let nonce = Nonce::from_slice(b"unique nonce"); // 12 octets
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();
    general_purpose::STANDARD.encode(ciphertext)
}

/// Décode en Base64 et déchiffre une chaîne en AES-256-GCM.
fn decrypt_from_base64(key: &[u8], b64_ciphertext: &str) -> String {
    let nonce = Nonce::from_slice(b"unique nonce");
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let ciphertext = general_purpose::STANDARD
        .decode(b64_ciphertext)
        .expect("Base64 invalide");
    let decrypted_data = cipher.decrypt(nonce, ciphertext.as_ref()).unwrap();
    String::from_utf8(decrypted_data).unwrap()
}

/// Insère un enregistrement dans la table `otp_object` en chiffrant les champs.
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

/// Récupère l'OTP secret et le service, déchiffrés, et renvoie un vecteur de tuples (secret_key, service).
pub fn select_keys(
    conn: &Connection,
    encryption_key: &[u8],
) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT secret_key, service FROM otp_object")?;
    let rows = stmt.query_map([], |row| {
        let enc_secret_key: String = row.get(0)?;
        let enc_service: String = row.get(1)?;

        let secret_key = decrypt_from_base64(encryption_key, &enc_secret_key);
        let service = decrypt_from_base64(encryption_key, &enc_service);

        Ok((secret_key, service))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}
