use std::{borrow::Borrow, collections::HashMap, path::PathBuf, sync::Arc};

use tokio::sync::Mutex;

// TODO: trait

#[derive(Debug, Default)]
pub struct GitRemotePool {
    pub process_pool: HashMap<String, Arc<Mutex<GitRemoteProcess>>>,
}

impl GitRemotePool {
    pub fn get_process(&mut self, id: impl AsRef<str>) -> Arc<Mutex<GitRemoteProcess>> {
        match self.process_pool.get(id.as_ref()) {
            Some(process) => process.clone(),
            None => {
                let process = Arc::new(Mutex::new(GitRemoteProcess::new(id.as_ref().into())));
                self.process_pool
                    .insert(id.as_ref().into(), process.clone());
                process
            }
        }
    }
}

#[derive(Debug)]
pub struct GitRemoteProcess {
    git_context_dir: PathBuf,
}

impl GitRemoteProcess {
    pub fn new(id: String) -> Self {
        // create work (context) dir
        // run process
        // store pipes in the struct
        todo!()
    }

    pub async fn call(&self, input: String) -> anyhow::Result<Vec<u8>> {
        Ok(todo!())
    }
}
