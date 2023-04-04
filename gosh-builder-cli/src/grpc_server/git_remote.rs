use std::{collections::HashMap, path::PathBuf, process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Child,
    sync::Mutex,
};

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
    git_context_dir: PathBuf,
    process: Child,
}

impl GitRemoteProces {
    pub fn spawn(id: impl AsRef<str>, args: Vec<String>) -> Self {
        let cwd: PathBuf = std::env::current_dir()
            .expect("current dir expected")
            .join(".git-cache")
            .join(id.as_ref());

        let process = tokio::process::Command::new("git-remote-gosh")
            .args(args)
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("process spawn successful");

        Self {
            git_context_dir: cwd,
            process,
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

        while let Some(line) = reader.next_line().await? {
            return Ok(line.into());
        }
        anyhow::bail!("Unexpected end of stdout")
    }
    pub fn get_archive(&self) -> anyhow::Result<Vec<u8>> {
        todo!()
    }
}
