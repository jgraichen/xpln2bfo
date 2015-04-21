//
pub use self::errors::Error;

mod errors;
mod ods;

extern crate xml;
extern crate zip;

use std::io;
use std::io::Read;
use std::error;
use std::fmt;


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

        if !mimetype.starts_with("application/vnd.oasis.opendocument.") {
            return Err(Error::InvalidMimetype);
        }

        return Ok(File { mimetype: mimetype });
    }
}
