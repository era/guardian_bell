use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::{BufReader, Error as StdIOError, Write};
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error {0}")]
    IOError(#[from] StdIOError),
    #[error("Page index out of range")]
    PageIndexOutOfRange,
}

/// Log is basically a single-file WAL, it allow callers to write to the end of the file
/// and read X bytes from any offset. If there is nothing to be read, it returns None
struct Log {
    writer: File,
    reader: File,
    name: String,
}

impl Log {
    fn new(file_name: PathBuf, name: String) -> Result<Self, Error> {
        let writer = OpenOptions::new().append(true).open(&file_name)?;
        let reader = OpenOptions::new().read(true).open(&file_name)?;

        Ok(Self {
            writer,
            reader,
            name,
        })
    }

    /// write data to the end of the file
    fn write(&mut self, data: &Vec<u8>) -> Result<(), Error> {
        self.writer.write_all(data)?;
        Ok(())
    }

    fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<usize, Error> {
        self.reader.seek(SeekFrom::Start(offset))?;
        Ok(self.reader.read(buf)?)
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
    curr_page: usize,
}

impl WAL {
    fn new(path: PathBuf) -> Self {
        //TODO open logs
        Self {
            path,
            logs: vec![],
            curr_page: 0,
        }
    }

    /// writes to the end of the last page
    fn write(&mut self, data: &Vec<u8>) -> Result<(), Error> {
        self.logs.get_mut(self.curr_page).unwrap().write(data)
    }

    /// read page starting from offset. Each page is a log file.
    fn read(&mut self, page: usize, offset: u64, buf: &mut [u8]) -> Result<usize, Error> {
        self.logs
            .get_mut(page)
            .ok_or(Error::PageIndexOutOfRange)?
            .read(offset, buf)
    }
}
