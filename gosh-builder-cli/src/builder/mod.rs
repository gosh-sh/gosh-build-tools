pub mod config;

use std::path::Path;
use tokio::process::Command;

pub async fn run<P: AsRef<Path>>(workdir: P, docker_file_path: impl AsRef<Path>) {
    let mut command = Command::new("docker");
    command.arg("buildx").arg("build");
    command.arg("--progress=plain");
    command.arg("--no-cache");
    command.arg("--network=host");
    command.arg("--file").arg(docker_file_path.as_ref());
    command.arg("--tag").arg("gosh-build");
    command
        .arg("--build-arg")
        .arg("http_proxy=http://127.0.0.1:8000");
    command
        .arg("--build-arg")
        .arg("https_proxy=http://127.0.0.1:8000");
    command.arg(workdir.as_ref().as_os_str());

    println!("{:?}", command);

    command
        .spawn()
        .expect("no errors while build")
        .wait()
        .await
        .expect("no errors");
    // println!(
    //     "{:?}",
    //     command.output().await.expect("failed to execute process")
    // );
}
