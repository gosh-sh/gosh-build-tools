use crate::commands::build::{build_image, gosh_config_url};
use clap::ArgMatches;
use git_registry::registry::GitCacheRegistry;
use gosh_builder::{
    docker_builder::git_context::GitContext,
    sbom::{load_bom, Sbom, SBOM_DEFAULT_FILE_NAME},
};
use std::{net::SocketAddr, path::PathBuf, process::Stdio, sync::Arc};
use tokio::{process::Command, sync::Mutex};

pub const COMMAND: &str = "install";
pub const DEFAULT_CONFIG_PATH: &str = "Gosh.yaml";
pub const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:6054";

#[derive(Debug, Clone)]
pub struct InstallSettings {
    pub config_path: PathBuf,
    pub workdir: PathBuf,
    pub git_context: Option<GitContext>,
    pub sbom_proxy_socket: SocketAddr,
}

pub fn command() -> clap::Command {
    clap::Command::new(COMMAND)
        .arg(
            clap::Arg::new("socket")
                .short('s')
                .long("socket")
                .help("Socket address for the SBOM proxy server")
                .value_name("IP:PORT")
                .default_value(DEFAULT_SOCKET_ADDR),
        )
        .arg(
            clap::Arg::new("config")
                .short('c')
                .long("config")
                .value_name("PATH")
                .help("Config path (in case of GOSH url context it should be relative to the root)")
                .default_value(DEFAULT_CONFIG_PATH),
        )
        .arg(clap::Arg::new("gosh_url").value_name("gosh://0:..."))
        .about("Install GOSH repo")
}

pub fn install_settings(matches: &ArgMatches) -> anyhow::Result<InstallSettings> {
    let git_context = match matches.try_get_one::<String>("gosh_url")? {
        Some(gosh_url) => Some(gosh_url.parse()?),
        None => anyhow::bail!("url required"),
    };

    let mut gosh_configfile = PathBuf::from(
        matches
            .get_one::<String>("config")
            .expect("should never fail due to `.default_value`"),
    );

    // TODO: don't check gosh_configfile.exists(), propogate it to git registry checks
    if git_context.is_none() {
        // local

        if !gosh_configfile.exists() {
            tracing::error!("Gosh config path doesn't exist");
            anyhow::bail!("Gosh config path doesn't exist");
        }

        if !gosh_configfile.is_absolute() {
            gosh_configfile = std::env::current_dir()?.join(gosh_configfile);
        }

        gosh_configfile
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("gosh configfile path canonicalize: {}", e))?;
    } else {
        // gosh remote
        if gosh_configfile.is_absolute() {
            anyhow::bail!("in case of gosh remote url `--config` path should be relative to the root of the repository");
        }
    }

    // TODO: git registry checks
    let mut workdir = gosh_configfile.clone();
    workdir.pop();

    let sbom_proxy_socket = matches
        .get_one::<String>("socket")
        .expect("should never fail due to `.default_value`")
        .parse()?;

    let settings = InstallSettings {
        config_path: gosh_configfile,
        workdir,
        git_context,
        sbom_proxy_socket,
    };

    tracing::debug!("{:?}", settings);

    Ok(settings)
}

pub async fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    let build_settings = install_settings(matches)?;

    let git_cache_registry = Arc::new(GitCacheRegistry::default());

    // let some
    let Some(ref git_context) = build_settings.git_context else {
        anyhow::bail!("url is required")
    };

    let mut gosh_config = gosh_config_url(
        git_context,
        &build_settings.config_path,
        &git_cache_registry,
    )
    .await?;

    // let mut gosh_config = gosh_config(&build_settings, &git_cache_registry).await?;
    // TODO: build doesn't need install paths, probably we should make
    // another config for build
    let install_paths = std::mem::take(&mut gosh_config.install);

    tracing::debug!("Dockerfile:\n{}", gosh_config.dockerfile);

    let sbom = Arc::new(Mutex::new(Sbom::default()));

    let image_id = build_image(
        gosh_config,
        true,
        build_settings.sbom_proxy_socket,
        sbom.clone(),
        git_cache_registry.clone(),
    )
    .await?;

    if image_id.is_empty() {
        tracing::error!("Build image fail");
        anyhow::bail!("Build image fail");
    }

    // SBOM
    tracing::info!("Validate SBOM...");
    let file_path = PathBuf::from(git_context.sub_dir.as_str()).join(SBOM_DEFAULT_FILE_NAME);
    let old_bom = load_bom(
        git_cache_registry
            .git_show_uncompressed(
                git_context.remote.as_str(),
                git_context.git_ref.as_str(),
                file_path.to_string_lossy(),
            )
            .await?
            .as_slice(),
    )?;

    let bom = sbom.lock().await.get_bom()?;
    if bom != old_bom {
        tracing::error!("SBOM validation fail");
        anyhow::bail!("SBOM validation fail");
    } else {
        tracing::info!("SBOM validation success");
    }

    tracing::info!("Image ID: {}", image_id);

    // INSTALL

    let create_container_output = Command::new("docker")
        .arg("container")
        .arg("create")
        .arg(image_id)
        .output()
        .await?;

    let container_id = String::from_utf8(create_container_output.stdout)?;
    tracing::debug!("Container ID: {}", container_id);

    // TODO: check install paths
    for path in install_paths {
        tracing::info!("Installing {}...", path);
        // TODO: ask for permissions
        Command::new("sudo")
            .arg("docker")
            .arg("container")
            .arg("cp")
            .arg(format!("{}:{}", container_id, path))
            .arg(path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()
            .await?;
    }

    Ok(())
}
