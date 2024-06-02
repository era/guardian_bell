use std::fs::{File, OpenOptions};
use std::io::{BufReader, Error as StdIOError, Write};
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error {0}")]
    IOError(#[from] StdIOError),
}

pub trait WriteAheadLog {
    fn write(&mut self, data: &Vec<u8>) -> Result<(), Error>;
    fn read<R>(
        &self,
        offset: u64,
        buf: &mut BufReader<R>,
        lines: u64,
    ) -> Result<Option<u64>, Error>;
}

/// Log is basically a single-file WAL, it allow callers to write to the end of the file
/// and read R bytes from any offset. If there is nothing to be read, it returns None
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

    /// write data to the end of the file
    fn write(&mut self, data: &Vec<u8>) -> Result<(), Error> {
        self.file.write_all(data)?;
        Ok(())
    }

    /// R is the size of the entries on the log
    fn read<R>(
        &self,
        offset: u64,
        buf: &mut BufReader<R>,
        lines: u64,
    ) -> Result<Option<u64>, Error> {
        todo!()
    }
}

/// Implements a WAL for the application
/// is up to the caller to handle snapshot (to reduce the disk usage, probably
/// should create a new WAL dump the content from memory and deleting the old one), validating
/// the content of the files (e.g. making sure all entries are valid and not corrupted)
/// and handling any indexing needed.
pub struct WAL {
    path: PathBuf,
    logs: Vec<Log>,
    page: u32,
}

impl WAL {
    fn new(path: PathBuf) -> Self {
        //TODO open logs
        Self {
            path,
            logs: vec![],
            page: 0,
        }
    }

    /// writes to the end of the last page
    fn write(&mut self, data: &Vec<u8>) -> Result<(), Error> {
        todo!()
    }

    /// read page starting from offset. Each page is a log file.
    fn read<R>(
        &self,
        page: u32,
        offset: u64,
        buf: &mut BufReader<R>,
        lines: u64,
    ) -> Result<Option<u64>, Error> {
        todo!()
    }
}
