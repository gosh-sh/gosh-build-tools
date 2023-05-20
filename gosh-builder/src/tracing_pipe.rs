use tokio::io::{AsyncBufReadExt, AsyncRead};

pub trait MapPerLine {
    fn map_per_line<F>(self, f: F)
    where
        F: Fn(&str) + Send + 'static;
}

impl<A> MapPerLine for A
where
    A: Unpin + AsyncRead + Send + 'static,
{
    fn map_per_line<F>(self, f: F)
    where
        F: Fn(&str) + Send + 'static,
    {
        tokio::spawn(async move {
            let mut lines = tokio::io::BufReader::new(self).lines();
            // Read lines one by one until the AsyncBufRead instance is exhausted
            while let Some(line) = lines.next_line().await.unwrap() {
                // Pass each line to the wrapper function for processing
                f(&line);
            }
        });
    }
}
