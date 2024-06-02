use std::fs::{File, OpenOptions};
use std::io::{BufReader, Error as StdIOError, Write};
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error {0}")]
    IOError(#[from] StdIOError),
}

/// RecoverableLog is basically a WAL, it allow callers to write to the end to the file
/// and read from any position. If there is nothing to be read, it returns None
pub trait RecoverableLog {
    /// write data to the end of the file
    fn write(&mut self, data: &Vec<u8>) -> Result<(), Error>;
    fn read<R>(&self, offset: u64, buf: &mut BufReader<R>, lines: u64) -> Result<Option<u64>, Error>;
}

struct Log {
    file: File,
    name: String,
}

impl Log {
    fn new(file_name: PathBuf, name: String) -> Result<Self, Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&file_name)?;

        Ok(Self { file, name })
    }
}

impl RecoverableLog for Log {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), Error> {
        self.file.write_all(data)?;
        Ok(())
    }
    fn read<R>(&self, offset: u64, buf: &mut BufReader<R>, lines: u64) -> Result<Option<u64>, Error> {
        todo!()
    }
}

pub struct WAL {
    path: PathBuf,
    logs: Vec<Log>,
}

impl WAL {
    fn new(path: PathBuf) -> Self {
        Self { path, logs: vec![] }
    }
}
