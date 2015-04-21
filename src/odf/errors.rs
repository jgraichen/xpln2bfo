//
extern crate zip;

//
use std::io;
use std::io::Read;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ZipError(zip::result::ZipError),
    InvalidMimetype,
}

impl ::std::convert::From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Error {
        Error::Zip(err)
    }
}

impl ::std::convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref err) => (err as &error::Error).description(),
            Error::ZipError(ref err) => (err as &error::Error).description(),
            Error::InvalidMimetype => "Invalid mimetype: Not an open document mimetype."
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref err) => Some(err as &error::Error),
            Error::ZipError(ref err) => Some(err as &error::Error),
            _ => None,
        }
    }
}

impl fmt::Display for Error
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        fmt.write_str((self as &error::Error).description())
    }
}
