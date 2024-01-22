use nom::{
    bytes::complete::{tag, take_till},
    error::Error as NomError,
    Err as NomErr,
};
use std::{
    cmp,
    collections::VecDeque,
    fmt,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    str::FromStr,
};

use file_guard::FileGuard;

#[derive(Debug)]
pub enum LogError {
    /// Something went wrong with the file io, e.g. we couldn't open the file
    Io(io::Error),
    /// The format of the log file is invalid
    InvalidFormat,
}

impl From<io::Error> for LogError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl<I> From<NomErr<NomError<I>>> for LogError {
    fn from(_: NomErr<NomError<I>>) -> Self {
        Self::InvalidFormat
    }
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(..) => write!(f, "while using shared logfile (maybe it was deleted?)"),
            Self::InvalidFormat => write!(f, "corrupted logfile"),
        }
    }
}

/// A shared log file that synchronizes writes
#[derive(Debug, Clone)]
pub struct Log {
    path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub pid: i32,
}

impl LogEntry {
    /// The size estimate for this LogEntry when encoded as plaintext (utf8).
    /// This includes the delimiter.
    ///
    /// # Notes
    /// - Proc IDs for procs we make are typically 5 bytes.  
    pub const ENCODED_SIZE_ESTIMATE: usize = 8;

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LogError> {
        let s = std::str::from_utf8(bytes).map_err(|_| LogError::InvalidFormat)?;
        Self::from_str(s)
    }
}

impl FromStr for LogEntry {
    type Err = LogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // if this ever gets more complicated we should do a nom impl
        Ok(Self {
            pid: s.trim().parse().map_err(|_| LogError::InvalidFormat)?,
        })
    }
}

impl Log {
    /// The delimeter to be used between entries
    pub const ENTRY_DELIM: u8 = b'|';

    /// The maximum chunk size when processing files
    pub const MAX_CHUNK_SIZE: usize = 1024;

    fn open(&self) -> Result<File, LogError> {
        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&self.path)?;
        Ok(file)
    }

    fn lock(file: &File) -> Result<FileGuard<&File>, LogError> {
        let lock = file_guard::lock(file, file_guard::Lock::Exclusive, 0, isize::MAX as usize)?;
        Ok(lock)
    }

    fn write_all(&self, buf: &[u8]) -> Result<(), LogError> {
        let file = self.open()?;
        let _lock = Self::lock(&file);
        (&file).write_all(buf)?;
        Ok(())
    }

    /// Log the completion of the "CPU-intensive task" that we are doing for a
    /// given pid.
    pub fn log_task_completion(&self, pid: i32) -> Result<(), LogError> {
        let s = format!("{pid}{}", Self::ENTRY_DELIM as char);
        self.write_all(s.as_bytes())?;
        Ok(())
    }

    /// Read up to `count` entries from the end of the logfile
    pub fn read_entries(&self, count: usize) -> Result<VecDeque<LogEntry>, LogError> {
        /// Process a buffer, outputting all processed entries to `out`. Returns
        /// the 'remainder'. That is any unprocessed input at the start of the
        /// buffer that still needs to be processed
        ///
        /// Use `upto` to limit the number of entries that are processed.
        fn process_buf<'a>(
            buf: &'a [u8],
            out: &mut VecDeque<LogEntry>,
            upto: usize,
        ) -> Result<&'a [u8], LogError> {
            if buf == [] {
                return Ok(&[]);
            }

            let input = buf;
            let (input, entry_str) = take_till(|b| b == Log::ENTRY_DELIM)(input)?;
            let (input, _) = tag(&[Log::ENTRY_DELIM])(input)?;
            let rem = &buf[0..=entry_str.len()];

            let mut chunk = vec![];
            let mut input = input;
            loop {
                let (newinput, entry_str) = take_till(|b| b == Log::ENTRY_DELIM)(input)?;
                input = newinput;

                if let Ok((newinput, _)) =
                    tag::<_, &[u8], NomError<&[u8]>>(&[Log::ENTRY_DELIM])(input)
                {
                    input = newinput;
                } else {
                    // EOF. We do not care about the case where the input starts
                    // with 0 whitespace and then the delimiter because we handle
                    // that with rem
                    break;
                }
                if chunk.len() >= upto {
                    break;
                }

                chunk.push(LogEntry::from_bytes(entry_str)?);
            }
            for entry in chunk.into_iter().rev() {
                out.push_front(entry);
            }

            Ok(rem)
        }

        let file = self.open()?;
        let file = &mut &file;
        let _lock = Self::lock(file);

        let mut entries = VecDeque::with_capacity(count);
        let mut rem = vec![];
        let chunk_size = cmp::min(
            count * LogEntry::ENCODED_SIZE_ESTIMATE,
            Self::MAX_CHUNK_SIZE,
        );
        let mut buf = vec![];
        let mut seek_offset = file.seek(SeekFrom::End(0))?;
        while seek_offset > 0 {
            buf.resize(chunk_size, 0);
            let to_read = cmp::min(seek_offset as usize, chunk_size);
            seek_offset -= to_read as u64;
            file.seek(SeekFrom::Start(seek_offset))?;
            buf.truncate(to_read);
            file.read_exact(&mut buf)?;
            buf.extend(&rem);
            rem.clear();
            let upto = count - entries.len();
            rem.extend(process_buf(&buf, &mut entries, upto)?);
        }

        Ok(entries)
    }

    /// Reset the log file, and return a handle to it (this [`Log`])
    pub fn create(path: PathBuf) -> Result<Self, LogError> {
        _ = File::create(&path)?;
        Ok(Self { path })
    }
}
