use std::fs;
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
    #[error("Error while loading logs")]
    CorruptedLogFile,
}

/// Log is a single-file WAL, it allows callers to write to the end of the file
/// and read X bytes from any offset.
struct Log {
    writer: File,
    reader: File,
    name: String,
}

impl Log {
    fn new(file_name: PathBuf, name: String) -> Result<Self, Error> {
        let writer = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_name)?;
        let reader = OpenOptions::new().read(true).open(&file_name)?;

        Ok(Self {
            writer,
            reader,
            name,
        })
    }

    /// write data to the end of the file. Returns the offset where the entry
    /// starts on the file.
    fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        let curr_offset = self.len()?;
        self.writer.write_all(data)?;

        Ok(curr_offset)
    }

    fn read(&mut self, offset: u64, buf: &mut [u8]) -> Result<usize, Error> {
        self.reader.seek(SeekFrom::Start(offset))?;
        Ok(self.reader.read(buf)?)
    }

    fn len(&self) -> Result<usize, Error> {
        Ok(self.writer.metadata()?.len() as usize)
    }
}

/// Implements a WAL for the application
/// is up to the caller to handle snapshot (to reduce the disk usage, probably
/// should create a new WAL dump the content from memory and delete the old one), validating
/// the content of the files (e.g. making sure all entries are valid and not corrupted)
/// and handling any indexing needed.
pub struct WAL {
    path: PathBuf,
    logs: Vec<Log>,
    curr_page: usize,
    max_size_per_page: usize,
}

pub struct Config {
    pub dir: PathBuf,
    pub max_size_per_page: usize,
}

impl WAL {
    const LOG_PREFIX: &'static str = "log_page_";

    pub fn new(config: Config) -> Result<Self, Error> {
        let mut logs = Self::find_logs(&config.dir)?;

        let mut curr_page = logs.len();

        if curr_page == 0 {
            let first_log = Self::create_log(config.dir.clone(), 0)?;
            curr_page = 1;
            logs.push(first_log);
        }

        Ok(Self {
            path: config.dir,
            max_size_per_page: config.max_size_per_page,
            logs,
            curr_page,
        })
    }

    fn create_log(mut path: PathBuf, page: usize) -> Result<Log, Error> {
        path.push(format!("{}{}", Self::LOG_PREFIX, page));
        let file_name = path
            .as_path()
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        Log::new(path, file_name)
    }

    fn find_logs(path: &PathBuf) -> Result<Vec<Log>, Error> {
        let mut entries: Vec<_> = fs::read_dir(path)?
            .filter_map(Result::ok)
            .filter(|e| {
                if let Some(file_name) = e.path().file_name().and_then(|s| s.to_str()) {
                    e.path().is_file() && file_name.starts_with(Self::LOG_PREFIX)
                } else {
                    false
                }
            })
            .collect();

        entries.sort_by_key(|e| e.path().file_name().unwrap().to_os_string());

        let mut logs = Vec::with_capacity(entries.len());

        for entry in entries {
            let name = entry.path().file_name().unwrap().to_os_string();
            logs.push(Log::new(entry.path(), name.into_string().unwrap())?);
        }

        Ok(logs)
    }

    /// writes to the end of the last page
    /// returns the (page, offset) so that you can retrieve the entry later
    pub fn write(&mut self, data: &[u8]) -> Result<(usize, usize), Error> {
        if self.logs.get(self.curr_page - 1).unwrap().len()? + data.len() > self.max_size_per_page {
            let log = Self::create_log(self.path.clone(), self.curr_page)?;
            self.logs.push(log);
            self.curr_page += 1;
        }

        let offset = self.logs.get_mut(self.curr_page - 1).unwrap().write(data)?;

        Ok((self.curr_page - 1, offset))
    }

    /// read page starting from offset. Each page is a log file.
    /// if usize = 0 there was nothing to be read.
    pub fn read(&mut self, page: usize, offset: u64, buf: &mut [u8]) -> Result<usize, Error> {
        self.logs
            .get_mut(page)
            .ok_or(Error::PageIndexOutOfRange)?
            .read(offset, buf)
    }

    pub fn last_page(&self) -> usize {
        self.curr_page
    }

    pub fn curr_page_size(&self) -> usize {
        self.logs.get(self.curr_page - 1).unwrap().len().unwrap()
    }

    pub fn is_empty_wal(&self) -> bool {
        self.last_page() == 1 && self.curr_page_size() == 0
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use temp_dir::TempDir;

    #[test]
    fn read_and_write_on_log() {
        let dir = TempDir::new().unwrap();
        let mut log = Log::new(dir.path().join("mylog"), "test".into()).unwrap();
        let entry = "my_entry".as_bytes();
        let result = log.write(&entry).unwrap();

        assert_eq!(0 as usize, result);

        let mut buf = [0; 8];
        log.read(0, &mut buf).unwrap();
        assert_eq!(entry, buf);

        // read starting from 3
        let mut buf = [0; 5];
        log.read(3, &mut buf).unwrap();
        assert_eq!("entry".as_bytes(), buf);

        let result = log.write(&entry).unwrap();
        assert_eq!(8 as usize, result);
    }

    #[test]
    fn read_and_write_on_wal() {
        let dir = TempDir::new().unwrap();
        let mut wal = WAL::new(Config {
            dir: dir.path().to_path_buf(),
            max_size_per_page: 8,
        })
        .unwrap();
        let entry = "my_entry".as_bytes();
        let result = wal.write(&entry).unwrap();
        assert_eq!((0 as usize, 0 as usize), result);

        let mut buf = [0; 8];
        wal.read(0, 0, &mut buf).unwrap();
        assert_eq!(entry, buf);

        // write on second page
        let entry = "second".as_bytes();
        let result = wal.write(&entry).unwrap();
        assert_eq!((1 as usize, 0 as usize), result);

        let mut buf = [0; 6];
        wal.read(1, 0, &mut buf).unwrap();
        assert_eq!(entry, buf);
    }
}
