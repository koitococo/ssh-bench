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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientProfile {
    Default,
    Throughput,
}

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
    connect_authenticated_with_profile(target, identity_path, ClientProfile::Default).await
}

pub async fn connect_authenticated_with_profile(
    target: &Target,
    identity_path: &std::path::Path,
    profile: ClientProfile,
) -> Result<client::Handle<AcceptAllClient>, AppError> {
    let config = client_config(profile);
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
        .map_err(|error| AppError::Config(format!("authentication: {}", error)))?;

    if !auth_result.success() {
        return Err(AppError::Config(
            "authentication: public key authentication failed".to_string(),
        ));
    }

    Ok(session)
}

pub fn client_config(profile: ClientProfile) -> Arc<client::Config> {
    Arc::new(match profile {
        ClientProfile::Default => client::Config {
            inactivity_timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        },
        ClientProfile::Throughput => client::Config {
            inactivity_timeout: Some(Duration::from_secs(5)),
            window_size: 16 * 1024 * 1024,
            channel_buffer_size: 1024,
            nodelay: true,
            ..Default::default()
        },
    })
}

pub async fn disconnect(session: &mut client::Handle<AcceptAllClient>) -> Result<(), AppError> {
    session
        .disconnect(Disconnect::ByApplication, "", "English")
        .await?;
    Ok(())
}

fn map_connect_error(error: russh::Error, target: &Target) -> AppError {
    let label = match &error {
        russh::Error::IO(_) | russh::Error::ConnectionTimeout | russh::Error::Disconnect => {
            ErrorKind::TcpConnect
        }
        russh::Error::KexInit
        | russh::Error::Kex
        | russh::Error::Version
        | russh::Error::NoCommonAlgo { .. }
        | russh::Error::KeyChanged { .. }
        | russh::Error::WrongServerSig
        | russh::Error::UnknownKey => ErrorKind::SshHandshake,
        _ if target.host.parse::<std::net::Ipv4Addr>().is_ok() => ErrorKind::TcpConnect,
        _ => ErrorKind::SshHandshake,
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
