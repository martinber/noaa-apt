//! Contains my Error type.


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

    /// About Image handling.
    Image(String),

    /// Deserializing errors.
    Deserialize(String),

    /// Related to HTML requests.
    Request(String),

    /// `noaa-apt` internal errors.
    Internal(String),

    /// Overflow of variables holding sample rates, most likely because the user
    /// choose strange sample rates.
    RateOverflow(String),

    /// shapefile library errors.
    Shapefile(String),

    /// Functionality not available because the program was compiled without
    /// those features
    FeatureNotAvailable(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::WavOpen(ref msg) => f.write_str(msg.as_str()),
            Error::Image(ref msg) => f.write_str(msg.as_str()),
            Error::Deserialize(ref msg) => f.write_str(msg.as_str()),
            Error::Request(ref msg) => f.write_str(msg.as_str()),
            Error::Internal(ref msg) => f.write_str(msg.as_str()),
            Error::RateOverflow(ref msg) => f.write_str(msg.as_str()),
            Error::Shapefile(ref msg) => f.write_str(msg.as_str()),
            Error::FeatureNotAvailable(ref msg) => f.write_str(msg.as_str()),
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

impl From<image::error::ImageError> for Error {
    fn from(err: image::error::ImageError) -> Self {
        match err {
            image::error::ImageError::IoError(io_error) => Error::Io(io_error),
            e => Error::Image(e.to_string()),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Deserialize(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Request(err.to_string())
    }
}

impl From<shapefile::Error> for Error {
    fn from(err: shapefile::Error) -> Self {
        Error::Shapefile(err.to_string())
    }
}
