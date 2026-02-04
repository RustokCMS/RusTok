use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("User is inactive")]
    UserInactive,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Password hashing error: {0}")]
    PasswordHashing(String),

    #[error("Token generation error: {0}")]
    Token(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Script error: {0}")]
    ScriptError(String),
}
