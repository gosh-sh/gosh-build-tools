use clap::ArgMatches;
use gosh_builder::{
    docker_builder::{git_context::GitContext, GoshBuilder, ImageBuilder},
    git_cache::registry::GitCacheRegistry,
    grpc_server,
    sbom::{load_bom, Sbom, SBOM_DEFAULT_FILE_NAME},
};
use gosh_builder_config::{
    raw_config::{Dockerfile, RawGoshConfig},
    GoshConfig, GoshConfigBuilder,
};
use std::{fs::File, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

pub const COMMAND: &str = "build";
pub const DEFAULT_CONFIG_PATH: &str = "Gosh.yaml";
pub const DEFAULT_SOCKET_ADDR: &str = "127.0.0.1:6054";

#[derive(Debug, Clone)]
pub struct CliSettings {
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

pub fn settings(matches: &ArgMatches) -> anyhow::Result<CliSettings> {
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

    let validate = matches.get_count("validate") > 0;
    let quiet = matches.get_count("quiet") > 0;

    let cli_config = CliSettings {
        config_path: gosh_configfile,
        workdir,
        validate,
        quiet,
        git_context,
        sbom_proxy_socket,
    };

    tracing::debug!("{:?}", cli_config);

    Ok(cli_config)
}

async fn gosh_config(
    cli_settings: &CliSettings,
    git_cache_registry: &GitCacheRegistry,
) -> anyhow::Result<GoshConfig> {
    let Some(ref git_context) = &cli_settings.git_context else {
        return Ok(GoshConfig::from_file(
            &cli_settings.config_path,
            &cli_settings.workdir,
        ))
    };

    // TODO: fix pessimistic cases
    // 1. abs paths (config shouldn't be absolute)
    // 2. config path can lead out of the git repo dir like '../../../../' many times

    let file_path = PathBuf::from(git_context.sub_dir.as_str()).join(&cli_settings.config_path);
    tracing::debug!("Config file_path: {:?}", file_path);

    let mut workdir = file_path.clone();
    workdir.pop();
    tracing::debug!("Config workdir: {:?}", workdir);

    let raw_config = RawGoshConfig::try_from_reader(
        zstd::decode_all(
            git_cache_registry
                .git_show(
                    git_context.remote.as_str(),
                    git_context.git_ref.as_str(),
                    file_path.to_string_lossy(),
                )
                .await?
                .as_slice(),
        )?
        .as_slice(),
    )?;

    let mut builder = GoshConfigBuilder::default();

    builder.dockerfile(match raw_config.dockerfile {
        Dockerfile::Content(content) => content,
        Dockerfile::Path { ref path } => {
            let dockerfile_path = workdir.join(path);
            tracing::debug!("Dockerfile path: {:?}", dockerfile_path);
            String::from_utf8(zstd::decode_all(
                git_cache_registry
                    .git_show(
                        git_context.remote.as_str(),
                        git_context.git_ref.as_str(),
                        dockerfile_path.to_string_lossy(),
                    )
                    .await?
                    .as_slice(),
            )?)?
        }
    });
    builder.tag(raw_config.tag);

    if let Some(ref args) = raw_config.args {
        builder.args(args.clone());
    };

    Ok(builder.build().expect("gosh config builder"))
}

pub async fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    let cli_settings = settings(matches)?;

    let git_cache_registry = GitCacheRegistry::default();

    let gosh_config = gosh_config(&cli_settings, &git_cache_registry).await?;

    let sbom = Arc::new(Mutex::new(Sbom::default()));

    let stop_grpc_server = grpc_server::run(
        cli_settings.sbom_proxy_socket,
        sbom.clone(),
        git_cache_registry,
    )
    .await?;

    tracing::debug!("Dockerfile:\n{}", gosh_config.dockerfile);

    let builder_exit_status = tokio::spawn(async move {
        tracing::info!("Start build...");

        let gosh_builder = GoshBuilder {
            config: gosh_config,
        };

        let inner_build_result = gosh_builder
            .run(cli_settings.quiet, &cli_settings.sbom_proxy_socket)
            .await;

        tracing::info!("End build...");
        inner_build_result
    })
    .await
    .expect("gosh builder subprocess join")?;

    tracing::info!("Stoping build server...");
    stop_grpc_server();

    if builder_exit_status.success() {
        tracing::info!("Build successful");
    } else {
        let exit_code = builder_exit_status.code().unwrap_or(1);
        anyhow::bail!("Docker build failed with exit code: {}", exit_code);
    }

    // SBOM

    // TODO: fix SBOM_OUT env var confusion in case of --validate

    if cli_settings.validate {
        tracing::info!("Validate SBOM...");
        let old_bom = load_bom(File::open(SBOM_DEFAULT_FILE_NAME)?)?;
        let bom = sbom.lock().await.get_bom()?;
        if bom != old_bom {
            tracing::error!("SBOM validation fail");
            anyhow::bail!("SBOM validation fail");
        } else {
            tracing::info!("SBOM validation success");
            return Ok(());
        }
    } else {
        let sbom_path = std::env::var("SBOM_OUT").unwrap_or(SBOM_DEFAULT_FILE_NAME.to_owned());

        tracing::info!("Writing SBOM to {}", sbom_path);
        sbom.lock().await.save_to(sbom_path).await?;
        tracing::info!("SBOM's ready");
    }

    Ok(())
}
