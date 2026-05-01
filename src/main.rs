use clap::Parser;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), ssh_bench::error::AppError> {
    let cli = ssh_bench::cli::Cli::parse();
    let config = cli
        .into_config()
        .map_err(ssh_bench::error::AppError::Config)?;
    let report = ssh_bench::bench::execute(&config).await?;

    if config.json {
        println!("{}", ssh_bench::report::render_json_report(&report)?);
    } else {
        println!("{}", ssh_bench::report::render_text_report(&report));
    }

    Ok(())
}
