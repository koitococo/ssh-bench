#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub user: String,
    pub host: String,
    pub port: u16,
}

impl Target {
    pub fn new(user: &str, host: &str, port: u16) -> Self {
        Self {
            user: user.to_string(),
            host: host.to_string(),
            port,
        }
    }
}

pub fn parse_target(input: &str) -> Result<Target, String> {
    let (user, host_port) = input
        .split_once('@')
        .ok_or_else(|| "missing @ separator".to_string())?;
    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or_else(|| "missing :port suffix".to_string())?;

    if user.is_empty() || host.is_empty() || port.is_empty() {
        return Err("target must include user, host and port".to_string());
    }

    if host.contains(':') {
        return Err("ipv6 targets are not supported".to_string());
    }

    let port = port
        .parse::<u16>()
        .map_err(|_| "port must be a valid u16".to_string())?;

    Ok(Target::new(user, host, port))
}

pub fn pick_target_for_worker(
    targets: &[Target],
    worker_index: usize,
    iteration: usize,
) -> Option<Target> {
    if targets.is_empty() {
        return None;
    }

    let index = (iteration + worker_index) % targets.len();
    targets.get(index).cloned()
}
