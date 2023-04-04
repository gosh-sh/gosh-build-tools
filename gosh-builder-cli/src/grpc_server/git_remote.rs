use std::{collections::HashMap, path::PathBuf, process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Child,
    sync::Mutex,
};

const GOSH_GRPC_CONTAINER: &str = "GOSH_GRPC_CONTAINER";
static DISPATCHER_ENDL: &str = "endl";

// TODO: trait

#[derive(Debug, Default)]
pub struct GitRemotePool {
    pub process_pool: HashMap<String, Arc<Mutex<GitRemoteProces>>>,
}

impl GitRemotePool {
    pub fn insert(&mut self, id: impl AsRef<str>, process: GitRemoteProces) {
        self.process_pool
            .insert(id.as_ref().into(), Arc::new(Mutex::new(process)));
    }
    pub fn get(&mut self, id: impl AsRef<str>) -> Arc<Mutex<GitRemoteProces>> {
        match self.process_pool.get(id.as_ref()) {
            Some(process) => process.clone(),
            None => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct GitRemoteProces {
    id: String,
    git_context_dir: PathBuf,
    process: Child,
    git_dir: PathBuf,
}

impl GitRemoteProces {
    pub async fn spawn(id: impl AsRef<str>, args: Vec<String>) -> Self {
        let git_context_dir: PathBuf = std::env::current_dir()
            .expect("current dir expected")
            .join(".git-cache")
            .join(id.as_ref());

        std::fs::create_dir_all(git_context_dir.clone().as_path())
            .expect("create specific directories and their parents");

        let _ = tokio::process::Command::new("git")
            .arg("init")
            .current_dir(&git_context_dir)
            .status()
            .await
            .expect("git init");
        let git_dir = git_context_dir.join(".git");
        let process = tokio::process::Command::new("git-remote-gosh")
            .args(args)
            .current_dir(&git_context_dir)
            .env("GIT_DIR", &git_dir)
            .env(GOSH_GRPC_CONTAINER, "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("process spawn successful");

        Self {
            id: id.as_ref().to_owned(),
            git_context_dir,
            process,
            git_dir,
        }
    }

    pub async fn command(&mut self, input: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let Some(ref mut stdin) = self.process.stdin else {
            anyhow::bail!("Can't take stdin");
        };
        stdin.write_all(&input).await?;
        stdin.flush().await?;

        let Some(ref mut stdout) = self.process.stdout else {
            anyhow::bail!("Can't take stdout");
        };
        let mut reader = BufReader::new(stdout).lines();
        let mut output = vec![];
        while let Some(line) = reader.next_line().await? {
            if line.contains(DISPATCHER_ENDL) {
                break;
            }
            output.push(line);
        }
        let mut buffer = vec![];
        for line in output {
            buffer.append(&mut format!("{line}\n").as_bytes().to_vec());
        }
        Ok(buffer)
    }

    pub async fn get_archive(&self) -> anyhow::Result<Vec<u8>> {
        let mut archive_buf = Vec::new();

        {
            let encoder = zstd::stream::Encoder::new(&mut archive_buf, 0)?.auto_finish();
            let mut tar_builder = tar::Builder::new(encoder);
            let obj_path = self.git_dir.join("objects");
            tar_builder.append_dir_all("objects", &obj_path)?;
        }
        Ok(archive_buf)
    }
}
