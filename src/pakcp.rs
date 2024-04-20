use {crc32fast::Hasher,
     std::{fs::File,
           io::{Read,
                Result,
                Write}},
     zstd::stream::Encoder};

pub(crate) struct CringeWriter<W: Write> {
    writer: W,
    hashr: Hasher,
}

impl<W: Write> CringeWriter<W> {
    fn new(writer: W) -> Self {
        CringeWriter {
            writer,
            hashr: Hasher::new(),
        }
    }

    fn checksum(&self) -> u32 {
        // ?
        <crc32fast::Hasher as Clone>::clone(&self.hashr).finalize()
    }
}

impl<W: Write> Write for CringeWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let written = self.writer.write(buf)?;
        self.hashr.update(&buf[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> Result<()> {
        self.writer.flush()
    }
}

// ?
pub(crate) struct CringeReader<W: Read> {
    reader: W,
    hashr: Hasher,
}

impl<W: Read> CringeReader<W> {
    pub(crate) fn new(writer: W) -> Self {
        CringeReader {
            reader: writer,
            hashr: Hasher::new(),
        }
    }

    fn checksum(&self) -> u32 {
        // ?
        <crc32fast::Hasher as Clone>::clone(&self.hashr).finalize()
    }
}

impl<W: Read> Read for CringeReader<W> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read = self.reader.read(buf)?;
        self.hashr.update(&buf[..read]);
        Ok(read)
    }
}

const CHUNK_SIZE: usize = 64 * 1024; // 64KB

pub fn fscow(inpf: &mut File, encw: &mut Encoder<'static, File>) -> Result<(u64, u32)> {
    let mut crcw = CringeWriter::new(encw);
    let mut buf = [0u8; CHUNK_SIZE];
    let mut written = 0;
    loop {
        let rs = inpf.read(&mut buf)?;
        if rs == 0 {
            break;
        }
        written += crcw.write(&buf[..rs])?;
        crcw.flush()?;
    }
    Ok((written as u64, crcw.checksum()))
}
