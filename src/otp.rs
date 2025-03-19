use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, TOTP};

fn generate_otp(secret: &str) -> String {
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.as_bytes().to_vec())
        .expect("Failed to create TOTP instance");

    totp.generate_current().unwrap_or_else(|_| "ERROR".to_string())
}

pub fn start_otp_generator(secret: &'static str, label: &'static str) {
    thread::spawn(move || {
        loop {
            let otp = generate_otp(secret);
            let remaining = 30 - (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() % 30);

            println!("🔑 [{}] OTP: {} | ⏳ Refresh in: {}s", label, otp, remaining);
            thread::sleep(Duration::from_secs(1)); 
        }
    });
}