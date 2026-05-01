use std::path::Path;

use russh::keys::load_secret_key;

use crate::error::AppError;

pub fn load_private_key(path: &Path) -> Result<russh::keys::PrivateKey, AppError> {
    if !path.exists() {
        return Err(AppError::Config(format!(
            "key_read: identity file does not exist: {}",
            path.display()
        )));
    }

    load_secret_key(path, None).map_err(|error| AppError::Key(error.to_string()))
}
