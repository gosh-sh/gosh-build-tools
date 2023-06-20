use clap::ArgMatches;
use git_registry::{git_context::GitContext, registry::GitCacheRegistry};
use gosh_builder::{
    docker_builder::{GoshBuilder, ImageBuilder},
    git_server,
};
use gosh_builder_config::GoshConfig;
use gosh_sbom::{load_bom, Sbom, SBOM_DEFAULT_FILE_NAME};
use std::{fs::File, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

pub const COMMAND: &str = "build";
pub const DEFAULT_CONFIG_PATH: &str = "Gosh.yaml";
pub const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:6054";

#[derive(Debug, Clone)]
pub struct BuildSettings {
    pub config_path: PathBuf,
    pub workdir: PathBuf,
    pub validate: bool,
    pub quiet: bool,
    pub git_context: Option<GitContext>,
    pub sbom_proxy_socket: SocketAddr,
}

pub fn command() -> clap::Command {
    // IMPORTANT
    // [ArgAction::Count] instead of [ArgAction::SetTrue] is intentional
    // because `--quiet` and other flags can be used multiple times (like in `docker build`)
    // and due to the chain of calls (e.g. telepresence -> bash_shortcut -> docker build)
    // some important flags might appear multiple times
    clap::Command::new(COMMAND)
        .about("Build GOSH image from `--config` or from [url]")
        .arg(
            clap::Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(clap::ArgAction::Count)
                .help("Suppress output"),
        )
        .arg(
            clap::Arg::new("validate")
                .long("validate")
                .action(clap::ArgAction::Count)
                .help("Validate the result image"),
        )
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
        .arg(
            clap::Arg::new("url")
                .value_name("gosh://0:...")
                .required(false),
        )
}

pub fn build_settings(matches: &ArgMatches) -> anyhow::Result<BuildSettings> {
    let git_context = match matches.try_get_one::<String>("url")? {
        Some(gosh_url) => Some(gosh_url.parse()?),
        None => None,
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

    let validate = matches.get_count("validate") > 0;
    let quiet = matches.get_count("quiet") > 0;

    let settings = BuildSettings {
        config_path: gosh_configfile,
        workdir,
        validate,
        quiet,
        git_context,
        sbom_proxy_socket,
    };

    tracing::debug!("{:?}", settings);

    Ok(settings)
}

pub async fn build_image(
    gosh_config: GoshConfig,
    quiet: bool,
    sbom_proxy_socket: SocketAddr,
    sbom: Arc<Mutex<Sbom>>,
    git_registry: Arc<GitCacheRegistry>,
) -> anyhow::Result<String> {
    // TODO: merge GRPC and Git servers
    // let stop_grpc_server = grpc_server::run(sbom_proxy_socket, sbom.clone(), git_registry).await?;
    let stop_git_server = git_server::run(sbom_proxy_socket, sbom.clone(), git_registry)?;

    let build_result = tokio::spawn(async move {
        tracing::info!("Start build...");

        let gosh_builder = GoshBuilder {
            config: gosh_config,
        };

        let inner_build_result = gosh_builder.run(quiet, sbom_proxy_socket).await;

        tracing::info!("End build...");
        inner_build_result
    })
    .await
    .expect("gosh builder subprocess join")?;

    tracing::info!("Stoping build server...");
    // stop_grpc_server();
    stop_git_server();

    if build_result.status.success() {
        tracing::info!("Build successful");
    } else {
        let exit_code = build_result.status.code().unwrap_or(1);
        anyhow::bail!("Docker build failed with exit code: {}", exit_code);
    };

    Ok(build_result.image_hash.unwrap_or("".to_owned()))
}

pub async fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    let build_settings = build_settings(matches)?;

    let git_cache_registry = Arc::new(GitCacheRegistry::default());

    let gosh_config = if let Some(ref git_context) = build_settings.git_context {
        GoshConfig::from_git_context(
            git_context,
            &build_settings.config_path,
            &git_cache_registry,
        )
        .await?
    } else {
        GoshConfig::from_file(&build_settings.config_path, &build_settings.workdir)?
    };

    tracing::debug!("Dockerfile:\n{}", gosh_config.dockerfile);

    let sbom = Arc::new(Mutex::new(Sbom::default()));

    let image_id = build_image(
        gosh_config,
        build_settings.quiet,
        build_settings.sbom_proxy_socket,
        sbom.clone(),
        git_cache_registry.clone(),
    )
    .await?;

    // SBOM

    if let Some(ref git_context) = &build_settings.git_context {
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
            anyhow::bail!("SBOM validation fail");
        } else {
            tracing::info!("SBOM validation success");
        }
    } else if build_settings.validate {
        tracing::info!("Validate SBOM...");
        let old_bom = load_bom(File::open(SBOM_DEFAULT_FILE_NAME)?)?;
        let bom = sbom.lock().await.get_bom()?;
        if bom != old_bom {
            anyhow::bail!("SBOM validation fail");
        } else {
            tracing::info!("SBOM validation success");
        }
    } else {
        let sbom_path = std::env::var("SBOM_OUT").unwrap_or(SBOM_DEFAULT_FILE_NAME.to_owned());

        tracing::info!("Writing SBOM to {}", sbom_path);
        sbom.lock().await.save_to(sbom_path).await?;
        tracing::info!("SBOM's ready");
    }

    if build_settings.quiet {
        // if EVERYTHING is OK than we just print image_id in a quite mode
        println!("{}", image_id);
    }

    Ok(())
}
