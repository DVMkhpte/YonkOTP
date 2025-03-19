mod database;

use rusqlite::{Connection, Result};
use database::{init_db, insert_otp_object, select_data, select_data_cond};

fn main() -> Result<()> {
    // Connexion à la base SQLite
    let conn = Connection::open("otp_data.db")?;
    // Initialisation de la table
    init_db(&conn)?;

    // Clé de chiffrement (32 octets pour AES-256)
    let key = b"01234567890123456789012345678901";

    // Insertion d'exemples
    insert_otp_object(&conn, "Google", "user_email", "SecretGoogle", key)?;
    insert_otp_object(&conn, "GitHub", "user_username", "SecretGitHub", key)?;
    insert_otp_object(&conn, "Facebook", "user_email", "SecretFacebook", key)?;

    // Sélection de tous les enregistrements
    let all_data = select_data(&conn, key)?;
    println!("Tous les enregistrements (id, service, u_m) :");
    for (id, service, u_m) in all_data {
        println!("ID: {}, Service: {}, u_m: {}", id, service, u_m);
    }

    // Sélection des enregistrements filtrés par u_m (exemple : "user_email")
    let filtered_data = select_data_cond(&conn, "user_email", key)?;
    println!("\nEnregistrements avec u_m == \"user_email\" :");
    for (id, service, u_m) in filtered_data {
        println!("ID: {}, Service: {}, u_m: {}", id, service, u_m);
    }

    Ok(())
}
