use object_store::WriteMultipart;
use std::io::{self, Write};

pub struct S3Writer {
    multipart: WriteMultipart,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl S3Writer {
    pub fn new(multipart: WriteMultipart, buffer_size: usize) -> Self {
        Self {
            multipart,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }

    pub async fn flush(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            self.multipart.write(&self.buffer);
            self.buffer.clear();
        }
        Ok(())
    } 

    pub async fn finish(mut self) -> io::Result<()> {
        self.flush().await?;
        self.multipart
            .finish()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }
}

impl Write for S3Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        if self.buffer.len() >= self.buffer_size {
            let buffer = std::mem::take(&mut self.buffer);
            self.multipart.write(&buffer);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Асинхронный flush не поддерживается в синхронном Write
        Ok(())
    }
}