use crate::init::GOSH_YAML_PATH;
use tokio::process::Command;

pub async fn build_command() -> anyhow::Result<()> {
    let _status = Command::new("gosh-builder-cli")
        .arg("--config")
        .arg(GOSH_YAML_PATH)
        .status()
        .await?;

    Ok(())
}
