extern crate xml;
extern crate zip;

use std::io;
use std::io::Read;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Zip(zip::result::ZipError)
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
            Error::Io(ref err) => (err as &error::Error).description(),
            Error::Zip(ref err) => (err as &error::Error).description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err as &error::Error),
            Error::Zip(ref err) => Some(err as &error::Error),
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


pub struct File {
    pub mimetype: String
}

pub enum Event {
    StartTable {
        name: String
    },

    Error(xml::common::Error)
}

impl File {
    pub fn new<R: io::Read+io::Seek>(read: R) -> Result<File, Error> {
        let mut archive  = try!(zip::read::ZipArchive::new(read));
        let mut mimefile = try!(archive.by_name("mimetype"));
        let mut mimetype = String::new();

        try!(mimefile.read_to_string(&mut mimetype));


        return Ok(File { mimetype: mimetype });
    }
}
