use validator:: ValidationError;

pub fn validate_password(password: &str) -> Result<() , ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let is_valid_length = password.len() >= 8 && password.len() <= 20;

    if !is_valid_length {
        let mut errror = ValidationError::new("Length");
        errror.message = Some("Password must be 8-20 characters long".into());
        return Err(errror);
    }

    if !has_uppercase || !has_lowercase || !has_digit {
        let mut error = ValidationError::new("format");
        error.message = Some("Password must contain as least one uppercase, one lowercase, and number".into());
        return Err(error);
    }
    Ok(())
}

