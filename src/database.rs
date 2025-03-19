use rusqlite::{params, Connection, Result};
pub fn start_db() -> Result<()> {
    // Ouvre ou crée la base de données SQLite
    let conn = Connection::open("yonkotp_data.db")?;

    // Création de la table pour stocker les sites et clés OTP
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_keys (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            site TEXT NOT NULL,
            key  TEXT NOT NULL
        )",
        [],
    )?;

    // Vérifier si des données existent déjà
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM user_keys", [], |row| row.get(0))?;

    if count == 0 {
        // Insérer une clé OTP de test
        conn.execute(
            "INSERT INTO user_keys (site, key) VALUES (?1, ?2)",
            params!["example.com", "JBSWY3DPEHPK3PXP"], // Clé OTP en Base32
        )?;
        println!("Ajout d'une key")
    }

    // Récupération des enregistrements et génération d'OTP
    Ok(())
}