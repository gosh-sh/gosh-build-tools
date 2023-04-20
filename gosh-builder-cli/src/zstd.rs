use async_compression::tokio::write::ZstdEncoder;
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
            let mut io_buffer = BufReader::new(self);
            let mut encoder = ZstdEncoder::new(&mut zstd_buffer);
            tokio::io::copy(&mut io_buffer, &mut encoder).await?;
        }

        Ok(zstd_buffer)
    }
}
