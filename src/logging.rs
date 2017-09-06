//! `logging` is a module which allows the custom writing of outputs to a file.
//!
//! #Last Modified
//!
//! Author --- Daniel Bechaz</br>
//! Date --- 06/09/2017

use std::fs::File;
use std::path::Path;
use std::io::Error;
use std::io::prelude::*;
use std::time::UNIX_EPOCH;

type WriteFunc = fn(&mut Logger, &str) -> Result<(), Error>;

/// A `Logger` writes formated strings to a file.
pub struct Logger {
    /// The `File` which the `Logger` writes to.
    file: File,
    /// A function for prettying strings before writing them to the `File`.
    write_func: WriteFunc
}

/// The default function for formatting the output to the log file.
///
/// # Params
///
/// log --- The `Logger` instance to write to.</br>
/// out --- The `str` slice to format and write.
fn default_write(log: &mut Logger, out: &str) -> Result<(), Error> {
    // Write the current timestamp, followed by the passed string.
    log.write_to_file(
        format!("\nTIMESTAMP: {}\n{}\n",
            UNIX_EPOCH
                .elapsed()
                .unwrap()
                .subsec_nanos(), 
            out
        ).as_str()
    )
}

impl Logger {
    /// Start a new instance of `Logger` attached to the file at the end of `path`.
    ///
    /// # Params
    ///
    /// path --- The `Path` of the file this `Logger` will write to.
    pub fn start<P: AsRef<Path>>(path: P) -> Result<Logger, Error> {
        match Logger::start_custom(path, default_write) {
            Ok(mut logger) => match logger.file
                .write_all(
                    format!("TIMESTAMP: {}\n",
                        UNIX_EPOCH
                            .elapsed()
                            .unwrap()
                            .subsec_nanos())
                            .as_bytes()
                ) {
                Ok(_) => match logger.file.flush() {
                    Ok(_) => Ok(logger),
                    Err(e) => Err(e)
                },
                Err(e) => Err(e)
            },
            Err(e) => Err(e)
        }
    }
    /// Start a new instance of `Logger` attached to the file at the end of `path`
    /// and using the customised formatting function.
    ///
    /// # Params
    ///
    /// path --- The `Path` of the file this `Logger` will write to.
    /// write_func --- The formatting function to apply to logged strings.
    pub fn start_custom<P: AsRef<Path>>(path: P, write_func: WriteFunc) -> Result<Logger, Error> {
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => match File::create(path) {
                Ok(file) => file,
                Err(e) => return Err(e)
            }
        };
        
        Ok(Logger { file, write_func })
    }
    #[inline]
    /// Writes the passed `str` slice directly to the log file, without formatting.
    ///
    /// # Params
    ///
    /// out --- `str` slice to log.
    pub fn write_to_file(&mut self, out: &str) -> Result<(), Error> {
        match self.file.write_all(out.as_bytes()) {
            Ok(_) => self.file.flush(),
            Err(e) => Err(e)
        }
    }
    #[inline]
    /// Writes the passed `str` slice to the log file after applying the formatting function.
    ///
    /// # Params
    ///
    /// out --- `str` slice to log.
    pub fn write(&mut self, out: &str) -> Result<(), Error> {
        (self.write_func)(self, out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;
    
    #[test]
    fn test_logger() {
        if let Err(_) = Logger::start("test.log") {
            panic!("Logger test-1 failed.");
        } else if let Err(_) = remove_file("test.log") {
            panic!("Logger test-1 failed in cleanup.");
        }
    }
}
