use std::process::{Command, ExitCode};

fn main() -> Result<ExitCode, std::io::Error> {
    Command::new("gosh")
        .arg("build")
        .arg("--quiet")
        .args(std::env::args_os().skip(1))
        .spawn()
        .expect("failed to run `gosh` executable")
        .wait()
        // try to exit with the same code as the child process
        .map(|child_status| ExitCode::from(child_status.code().unwrap_or(1).clamp(0, 255) as u8))
}
