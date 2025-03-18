pub fn serach_data(data: &Vec<(String, String, String)>, query: &str) -> Vec<(String, String, String)> {
    let query_lower = query.to_lowercase();

    data.iter()
        .filter(|(service, username, _)| {
            service.to_lowercase().contains(&query_lower) || username.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect()
}

pub fn validate_data(service: &str, username: &str, secret: &str) -> Result<(), String> {
    if service.trim().is_empty() {
        return Err("The service name cannot be empty.".to_string());
    }

    if username.trim().is_empty() {
        return Err("The username or email cannot be empty.".to_string());
    }

    if secret.trim().is_empty() {
        return Err("The secret key cannot be empty.".to_string());
    }

    let secret_len = secret.len();
    if secret_len < 16 || secret_len > 32 {
        return Err("The secret key must be between 16 and 32 characters long.".to_string());
    }

    if secret.chars().any(|c| c.is_lowercase()) {
        return Err("The secret key must contain only uppercase letters and numbers.".to_string());
    }

    let valid_base32_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    if secret.chars().any(|c| !valid_base32_chars.contains(c)) {
        return Err("The secret key must contain only valid Base32 characters (A-Z, 2-7).".to_string());
    }

    if secret_len % 8 != 0 {
        return Err("The secret key length must be a multiple of 8 to avoid padding issues.".to_string());
    }

    Ok(())
}