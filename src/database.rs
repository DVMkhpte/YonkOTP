use rusqlite::{params, Connection, Result};
use oath::totp_raw_now;

fn main() -> Result<()> {
    // Ouvre ou crée la base de données
    let conn = Connection::open("otp_data.db")?;

    // Création de la table pour stocker les sites et clés
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_keys (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            site TEXT NOT NULL,
            key  TEXT NOT NULL
        )",
        [],
    )?;

    // Insertion d'un exemple (vous pouvez adapter pour vérifier l'existence ou récupérer depuis une interface)
    conn.execute(
        "INSERT INTO user_keys (site, key) VALUES (?1, ?2)",
        params!["example.com", "ma_cle_secrete"],
    )?;

    // Récupération des enregistrements et génération d'OTP
    let mut stmt = conn.prepare("SELECT site, key FROM user_keys")?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let site: String = row.get(0)?;
        let key: String = row.get(1)?;

        // Pour cet exemple, on considère que la clé est en clair.
        // Convertissez la clé en bytes, et générez un OTP à 6 chiffres avec une période de 30 secondes.
        let secret = key.as_bytes();
        let otp = totp_raw_now(secret, 6, 0, 30);

        println!("Site : {}", site);
        println!("OTP : {}\n", otp);
    }

    Ok(())
}
