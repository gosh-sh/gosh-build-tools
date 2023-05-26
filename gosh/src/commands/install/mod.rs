use std::{fs::File, sync::Arc};

use crate::commands::build::{build_image, build_settings, gosh_config};
use clap::ArgMatches;
use git_registry::registry::GitCacheRegistry;
use gosh_builder::sbom::{load_bom, Sbom, SBOM_DEFAULT_FILE_NAME};
use tokio::sync::Mutex;

pub const COMMAND: &str = "install";

pub fn command() -> clap::Command {
    clap::Command::new(COMMAND)
        .arg(clap::Arg::new("gosh_url").value_name("gosh://0:..."))
        .about("Install GOSH repo")
}

pub async fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    let build_settings = build_settings(matches)?;

    let git_cache_registry = GitCacheRegistry::default();

    let gosh_config = gosh_config(&build_settings, &git_cache_registry).await?;

    tracing::debug!("Dockerfile:\n{}", gosh_config.dockerfile);

    let sbom = Arc::new(Mutex::new(Sbom::default()));

    let image_id = build_image(
        gosh_config,
        build_settings.quiet,
        build_settings.sbom_proxy_socket,
        sbom.clone(),
        git_cache_registry,
    )
    .await?;

    // SBOM
    tracing::info!("Validate SBOM...");
    let old_bom = load_bom(File::open(SBOM_DEFAULT_FILE_NAME)?)?;
    let bom = sbom.lock().await.get_bom()?;
    if bom != old_bom {
        tracing::error!("SBOM validation fail");
        anyhow::bail!("SBOM validation fail");
    } else {
        tracing::info!("SBOM validation success");
    }

    // INSTALL

    todo!()
}
