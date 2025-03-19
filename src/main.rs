use std::time::Duration;
use std::thread;

mod otp;
use otp::start_otp_generator;

fn main() {
    let keys = vec![
        ("JBSWY3DPEHPK3PXP", "Google"),
        ("GAYDAMBQGAYDAMBQ", "GitHub"),
        ("MYSECRETOTPBASE32", "Facebook"),
    ];

    for (key, label) in keys {
        start_otp_generator(key, label);
    }

    // Permet de garder le programme en exécution
    loop {
        thread::sleep(Duration::from_secs(10));
    }
}