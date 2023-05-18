use clap::ArgMatches;

pub async fn build_command(matches: &ArgMatches) -> anyhow::Result<()> {
    gosh_builder_cli::cli_builder::run(matches).await
}
