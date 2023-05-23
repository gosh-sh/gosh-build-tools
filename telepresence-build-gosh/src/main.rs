use std::process::{Command, ExitCode};

static HELP_PAGE: &str = "\
GOSH image builder for Telepresence
https://gosh.sh

Usage:

    telepresence-build-gosh <build-args> gosh://0:...
";

fn main() -> Result<ExitCode, std::io::Error> {
    let args = std::env::args_os();
    if args.len() == 1 {
        print!("{}", HELP_PAGE);
        return Ok(ExitCode::FAILURE);
    }
    Command::new("gosh")
        .arg("build")
        .arg("--quiet")
        .args(args.skip(1))
        .spawn()
        .expect("Failed to run the `gosh` executable.")
        .wait()
        // Try to exit with the same code as the child process.
        .map(|child_status| {
            ExitCode::from(
                child_status
                    .code()
                    .unwrap_or(1)
                    .clamp(u8::MIN as i32, u8::MAX as i32) as u8,
            )
        })
}
