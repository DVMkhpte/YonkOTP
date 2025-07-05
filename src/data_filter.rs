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
    if service.is_empty() || username.is_empty() || secret.is_empty() {
        return Err("Champs vides".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_data_success() {
        assert!(validate_data("gmail", "user", "secret").is_ok());
    }

    #[test]
    fn test_validate_data_fail_empty_service() {
        assert!(validate_data("", "user", "secret").is_err());
    }
}
