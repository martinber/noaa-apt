use std::fmt;

use std::io;
use hound;
use log;

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    WavOpen(String), // About WAV decoding/opening
    PngWrite(String), // About PNG encoding/writing
    Internal(String), // noaa-apt internal errors because of some bug
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::WavOpen(msg) => f.write_str(msg.to_str()),
            Error::PngWrite(msg) => f.write_str(msg.to_str()),
            Error::Internal(msg) => f.write_str(msg.to_str()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::WavOpen(msg) => msg.to_str(),
            Error::PngWrite(msg) => msg.to_str(),
            Error::Internal(msg) => msg.to_str(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<hound::Error> for Error {
    fn from(err: hound::Error) -> Error {
        match err {
            hound::Error::IoError(io_error) => Io(io_error),
            hound::Error::FormatError => WavOpen(err.to_string()),
            hound::Error::TooWide => Internal(err.to_string()),
            hound::Error::UnfinishedSample => Internal(err.to_string()),
            hound::Error::Unsupported => WavOpen(err.to_string()),
            hound::Error::InvalidSampleFormat => Internal(err.to_string()),
        }
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(err: log::SetLoggerError) -> Error {
        Internal(err.description())
    }
}
