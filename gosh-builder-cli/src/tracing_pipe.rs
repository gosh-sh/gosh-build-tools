use tokio::{
    io::AsyncBufReadExt,
    process::{ChildStderr, ChildStdout},
};

pub trait MapPerLine {
    fn map_per_line<F>(self, f: F)
    where
        F: Fn(&str) + Send + 'static;
}

impl MapPerLine for ChildStdout {
    fn map_per_line<F>(self, f: F)
    where
        F: Fn(&str) + Send + 'static,
    {
        let mut lines = tokio::io::BufReader::new(self).lines();
        tokio::spawn(async move {
            // Read lines one by one until the AsyncBufRead instance is exhausted
            while let Some(line) = lines.next_line().await.unwrap() {
                // Pass each line to the wrapper function for processing
                f(&line);
            }
        });
    }
}

impl MapPerLine for ChildStderr {
    fn map_per_line<F>(self, f: F)
    where
        F: Fn(&str) + Send + 'static,
    {
        let mut lines = tokio::io::BufReader::new(self).lines();
        tokio::spawn(async move {
            // Read lines one by one until the AsyncBufRead instance is exhausted
            while let Some(line) = lines.next_line().await.unwrap() {
                // Pass each line to the wrapper function for processing
                f(&line);
            }
        });
    }
}
