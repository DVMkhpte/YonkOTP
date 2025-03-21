use std::thread;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, TOTP};

fn generate_otp(secret: &str) -> String {
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.as_bytes().to_vec())
        .expect("Failed to create TOTP instance");

    totp.generate_current().unwrap_or_else(|_| "ERROR".to_string())
}

pub fn start_otp_generator(id: i64, secret: &'static str) -> Receiver<(i64, String, u64)> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        loop {
            let otp = generate_otp(secret);
            let remaining = 30 - (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() % 30);

            // **Envoie les données au récepteur**
            if tx.send((id, otp.clone(), remaining)).is_err() {
                break; // Quitter le thread si le canal est fermé
            }

            thread::sleep(Duration::from_secs(1));
        }
    });

    rx 
}