use async_compression::tokio::bufread;
use tokio::io::{AsyncRead, BufReader};

#[async_trait::async_trait]
pub trait ZstdReadToEnd {
    async fn zstd_read_to_end(self) -> anyhow::Result<Vec<u8>>;
}

#[async_trait::async_trait]
impl<A> ZstdReadToEnd for A
where
    A: AsyncRead + Unpin + Send,
{
    async fn zstd_read_to_end(self) -> anyhow::Result<Vec<u8>> {
        let mut zstd_buffer = Vec::new();

        // in theory it should save memory
        {
            let io_buffer = BufReader::new(self);
            let mut encoder = bufread::ZstdEncoder::new(io_buffer);
            tokio::io::copy(&mut encoder, &mut zstd_buffer).await?;
        }

        Ok(zstd_buffer)
    }
}
