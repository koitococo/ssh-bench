use std::path::Path;

use russh::keys::load_secret_key;

use crate::error::AppError;

pub fn load_private_key(path: &Path) -> Result<russh::keys::PrivateKey, AppError> {
    load_secret_key(path, None).map_err(|error| AppError::Key(error.to_string()))
}
