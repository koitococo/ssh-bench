use std::sync::Arc;
use std::time::Duration;

use russh::Disconnect;
use russh::client;
use russh::keys::PrivateKeyWithHashAlg;
use russh::keys::ssh_key;

use crate::error::AppError;
use crate::error::ErrorKind;
use crate::ssh::auth::load_private_key;
use crate::target::Target;

#[derive(Debug, Default)]
pub struct AcceptAllClient;

impl client::Handler for AcceptAllClient {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub async fn connect_authenticated(
    target: &Target,
    identity_path: &std::path::Path,
) -> Result<client::Handle<AcceptAllClient>, AppError> {
    let config = Arc::new(client::Config {
        inactivity_timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    });
    let mut session = client::connect(config, (target.host.as_str(), target.port), AcceptAllClient)
        .await
        .map_err(|error| map_connect_error(error, target))?;
    let key = load_private_key(identity_path).map_err(map_key_error)?;
    let auth_result = session
        .authenticate_publickey(
            target.user.as_str(),
            PrivateKeyWithHashAlg::new(
                Arc::new(key),
                session.best_supported_rsa_hash().await?.flatten(),
            ),
        )
        .await
        .map_err(|error| AppError::Ssh(error))?;

    if !auth_result.success() {
        return Err(AppError::Config("public key authentication failed".to_string()));
    }

    Ok(session)
}

pub async fn disconnect(session: &mut client::Handle<AcceptAllClient>) -> Result<(), AppError> {
    session
        .disconnect(Disconnect::ByApplication, "", "English")
        .await?;
    Ok(())
}

fn map_connect_error(error: russh::Error, target: &Target) -> AppError {
    let label = if target.host.parse::<std::net::Ipv4Addr>().is_ok() {
        ErrorKind::TcpConnect
    } else {
        ErrorKind::SshHandshake
    };

    AppError::Config(format!("{}: {}", format_error_kind(&label), error))
}

fn map_key_error(error: AppError) -> AppError {
    match error {
        AppError::Key(message) => AppError::Config(format!(
            "{}: {}",
            format_error_kind(&ErrorKind::KeyFormat),
            message
        )),
        other => other,
    }
}

fn format_error_kind(kind: &ErrorKind) -> &'static str {
    match kind {
        ErrorKind::TcpConnect => "tcp_connect",
        ErrorKind::SshHandshake => "ssh_handshake",
        ErrorKind::KeyFormat => "key_format",
        _ => "protocol",
    }
}
