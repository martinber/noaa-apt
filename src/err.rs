use std;
use hound;
use log;

// So I can use std::error::Error::description(), but I don't want to shadow
// Error in this scope
use std::error::Error as StdError;

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    WavOpen(String), // About WAV decoding/opening
    PngWrite(String), // About PNG encoding/writing
    Internal(String), // noaa-apt internal errors because of some bug
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::WavOpen(ref msg) => f.write_str(msg.as_str()),
            Error::PngWrite(ref msg) => f.write_str(msg.as_str()),
            Error::Internal(ref msg) => f.write_str(msg.as_str()),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::WavOpen(ref msg) => msg.as_str(),
            Error::PngWrite(ref msg) => msg.as_str(),
            Error::Internal(ref msg) => msg.as_str(),
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
            hound::Error::IoError(io_error) => Error::Io(io_error),
            hound::Error::FormatError(_) => Error::WavOpen(err.to_string()),
            hound::Error::TooWide => Error::Internal(err.to_string()),
            hound::Error::UnfinishedSample => Error::Internal(err.to_string()),
            hound::Error::Unsupported => Error::WavOpen(err.to_string()),
            hound::Error::InvalidSampleFormat => Error::Internal(err.to_string()),
        }
    }
}

impl From<log::SetLoggerError> for Error {
    fn from(err: log::SetLoggerError) -> Error {
        Error::Internal(err.description().to_string())
    }
}
