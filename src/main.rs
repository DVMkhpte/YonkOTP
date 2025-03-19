mod database;

use rusqlite::{Connection, Result};
use database::{init_db, insert_otp_object, select_keys};

fn main() -> Result<()> {
    // Connexion à la base SQLite
    let conn = Connection::open("otp_data.db")?;
    
    // Initialisation de la table
    init_db(&conn)?;

    // Clé de chiffrement (32 octets pour AES-256)
    let key = b"01234567890123456789012345678901";

    // Insertion de quelques enregistrements
    insert_otp_object(&conn, "Google", "user1", "JBSWY3DPEHPK3PXP", key)?;
    insert_otp_object(&conn, "GitHub", "user2", "GAYDAMBQGAYDAMBQ", key)?;
    insert_otp_object(&conn, "Facebook", "user3", "MYSECRETOTPBASE32", key)?;

    // Sélection des clés et services sous forme de vecteur de tuples
    let keys = select_keys(&conn, key)?;
    println!("Les clés récupérées : {:?}", keys);

    Ok(())
}
