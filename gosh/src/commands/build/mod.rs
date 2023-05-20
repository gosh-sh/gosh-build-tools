use clap::ArgMatches;

pub async fn build_command(matches: &ArgMatches) -> anyhow::Result<()> {
    gosh_builder::cli::run(matches).await
}
