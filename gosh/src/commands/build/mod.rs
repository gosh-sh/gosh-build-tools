use clap::ArgMatches;
use tokio::process::Command;

pub async fn build_command(matches: &ArgMatches) -> anyhow::Result<()> {
    let mut builder = Command::new("gosh-builder-cli");
    let args = matches.try_get_many::<String>("args")?.unwrap_or_default();

    tracing::info!("{:?}", args);
    builder.args(args);

    let _status = builder.status().await?;

    Ok(())
}
