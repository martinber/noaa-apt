//! Contains my Error type.

use hound;
use png;
use log;


/// Uses my custom error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Custom error type.
#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {

    /// Input/output.
    Io(std::io::Error),

    /// About WAV decoding/opening.
    WavOpen(String),

    /// About PNG encoding/writing.
    PngWrite(String),

    /// Deserializing errors.
    Deserialize(String),

    /// `noaa-apt` internal errors.
    Internal(String),

    /// Overflow of variables holding sample rates, most likely because the user
    /// choose strange sample rates.
    RateOverflow(String),

    /// Functionality not available because the program was compiled without
    /// those features
    FeatureNotAvailable(Vec<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::WavOpen(ref msg) => f.write_str(msg.as_str()),
            Error::PngWrite(ref msg) => f.write_str(msg.as_str()),
            Error::Deserialize(ref msg) => f.write_str(msg.as_str()),
            Error::Internal(ref msg) => f.write_str(msg.as_str()),
            Error::RateOverflow(ref msg) => f.write_str(msg.as_str()),
            Error::FeatureNotAvailable(ref features) =>
                write!(f, "Program compiled without support for features: {:?}",
                    features),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<hound::Error> for Error {
    fn from(err: hound::Error) -> Self {
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
    fn from(err: log::SetLoggerError) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<png::EncodingError> for Error {
    fn from(err: png::EncodingError) -> Self {
        match err {
            png::EncodingError::IoError(io_error) => Error::Io(io_error),
            png::EncodingError::Format(_) => Error::PngWrite(err.to_string()),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Deserialize(err.to_string())
    }
}
