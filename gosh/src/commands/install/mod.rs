use clap::ArgMatches;

pub const COMMAND: &str = "install";

pub fn command() -> clap::Command {
    clap::Command::new(COMMAND)
        .arg(clap::Arg::new("gosh_url").value_name("gosh://0:..."))
        .about("Install GOSH repo")
}

pub async fn run(matches: &ArgMatches) -> anyhow::Result<()> {
    // 1. build the image
    // gosh_builder::cli::run(matches).await
    // 2. validate the image
    // 3. copy files from the image to the host
    todo!()
}
