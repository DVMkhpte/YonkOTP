use YonkOTP::database::{init_db, insert_otp_object, select_data};

#[test]
pub fn test_insert_and_select() {
    // Utilise une base temporaire pour le test
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();

    let service = "test";
    let username = "test@toto.com";
    let secret = "ABC123";

    let aes_key = b"01234567890123456789012345678901";

    insert_otp_object(&conn, service, username, secret, aes_key).unwrap();

    let results = select_data(&conn, aes_key).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, service);
    assert_eq!(results[0].2, username);
}
