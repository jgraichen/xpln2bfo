#![feature(slice_patterns)]
#![feature(convert)]
extern crate zip;
extern crate xml;

use std::fs;

mod ods {
  use std::io;
  use std::io::{Read, Seek};
  use std::convert;

  use zip::read::ZipArchive;
  use zip::result::ZipError;

  use xml;
  use xml::reader::EventReader;
  use xml::reader::events::*;
  use xml::attribute::OwnedAttribute;

  #[derive(Debug)]
  pub enum Event {
    StartTable(String),
    EndTable(String),
    Row(Vec<String>)
  }

  #[derive(Debug)]
  pub enum Error {
    IoError(io::Error),
    ZipError(ZipError),
    XmlError(xml::common::Error),
    InvalidMimetype
  }

  impl convert::From<ZipError> for Error {
    fn from(err: ZipError) -> Error {
      Error::ZipError(err)
    }
  }

  impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
      Error::IoError(err)
    }
  }

  pub fn parse<T: Read+Seek, H: FnMut(Event) -> ()>(file: T, handler: &mut H) -> Result<(), Error> {
    let mut archive = try!(ZipArchive::new(file));

    {
      let mut mime = String::new();
      let mut file = try!(archive.by_name("mimetype"));

      try!(file.read_to_string(&mut mime));

      if mime != "application/vnd.oasis.opendocument.spreadsheet" {
        return Err(Error::InvalidMimetype)
      }
    }

    {
      let file = try!(archive.by_name("content.xml"));
      let mut parser = Parser::new(file);

      loop {
        match parser.scan("table") {
          Some(XmlEvent::StartElement { name: _, attributes, namespace: _ }) => {
            let table_name = match extract_attribute(&attributes, "name") {
              Some(name) => name,
              None => continue
            };

            handler(Event::StartTable(table_name));

            loop {
              match parser.scan("table-row") {
                Some(XmlEvent::StartElement { name: _, attributes: _, namespace: _ }) => {
                  let mut values = Vec::new();

                  loop {
                    match parser.scan("table-cell") {
                      Some(XmlEvent::StartElement { name: _, attributes: _, namespace: _ }) => {
                        values.push(parser.text());
                      },
                      Some(_) => break,
                      None => break
                    }
                  }

                  handler(Event::Row(values));
                },
                Some(_) => break,
                None => break
              }
            }

            let table_name = match extract_attribute(&attributes, "name") {
              Some(name) => name,
              None => continue
            };

            handler(Event::EndTable(table_name));
          },
          Some(XmlEvent::Error(err)) => {
            return Err(Error::XmlError(err));
          },
          Some(XmlEvent::EndDocument) => { break },
          Some(_) => {},
          None => { continue /* to next table */ }
        }
      }
    }

    Ok(())
  }

  struct Parser<T: Read> {
    xml: EventReader<T>,
    stack: Vec<String>
  }

  impl<T: Read> Parser<T> {
    fn new(file: T) -> Parser<T> {
      Parser {
        xml: EventReader::new(file),
        stack: Vec::new()
      }
    }

    fn scan(&mut self, search: &str) -> Option<XmlEvent> {
      loop {
        match self.xml.next() {
          XmlEvent::StartElement { name, attributes, namespace: ns } => {
            if name.local_name == search {
              if self.stack.last()
                  .map(|&ref top| top != search)
                  .unwrap_or(true) {
                let mut current = String::new();
                current.push_str(search);

                self.stack.push(current);
              }

              return Some(XmlEvent::StartElement { name: name, attributes: attributes, namespace: ns });
            }
          },
          XmlEvent::EndElement { name } => {
            if self.stack.get(self.stack.len() - 2)
                .map(|&ref prev| *prev == name.local_name)
                .unwrap_or(false) {
              self.stack.pop().unwrap();
              return None;
            }
          }
          XmlEvent::Error(err) => {
            return Some(XmlEvent::Error(err));
          },
          XmlEvent::EndDocument => {
            return Some(XmlEvent::EndDocument);
          },
          _ => {}
        }
      }
    }

    fn text(&mut self) -> String {
      let mut str = String::new();

      loop {
        match self.xml.next() {
          XmlEvent::Characters(string) => {
            str.push_str(string.as_ref());
          }
          XmlEvent::EndElement { name } => {
            match self.stack.last() {
              Some(&ref current) => {
                if name.local_name == *current {
                  break;
                }
              },
              None => ()
            }
          },
          XmlEvent::Error(_) => break,
          XmlEvent::EndDocument => break,
          _ => {}
        }
      }

      return str;
    }
  }

  fn extract_attribute(attributes: &Vec<OwnedAttribute>, name: &str) -> Option<String> {
    for attr in attributes.iter() {
      if attr.name.local_name == name {
        let mut str = String::new();
        str.push_str(attr.value.as_ref());

        return Some(str);
      }
    }

    None
  }
}

mod xpln {
  use std::str::FromStr;
  use std::num::ParseIntError;

  #[derive(Debug)]
  pub struct Train {
    pub number: usize,
    pub name: String,
    pub timetable: Vec<Record>,
  }

  #[derive(Debug)]
  pub struct Record {
    pub track: usize,
    pub remark: String,
    pub station: String,
    pub arrival: String,
    pub departure: String,
  }

  impl Train {
    pub fn parse(num: &str, name: &str) -> Result<Train, ParseIntError> {
      let train = Train {
        name: String::from(name),
        number: try!(usize::from_str(num)),
        timetable: Vec::new(),
      };

      Ok(train)
    }

    pub fn add_record(&mut self, record: Record) {
      self.timetable.push(record);
    }
  }

  impl Record {
    pub fn parse(station: &str, track: &str, arrival: &str, departure: &str,
                 remark: &str)  -> Result<Record, ParseIntError> {

      let record = Record {
        station: String::from(station),
        track: try!(usize::from_str(track)),
        arrival: String::from(arrival),
        departure: String::from(departure),
        remark: String::from(remark),
      };

      Ok(record)
    }
  }
}

fn main() {
  std::process::exit(run());
}

fn run() -> i32 {
  let args: Vec<_> = std::env::args().collect();
  if args.len() < 2 {
    println!("Usage: {} <input>", args[0]);
    return 1;
  }

  let fname = std::path::Path::new(&*args[1]);
  let file = match fs::File::open(&fname) {
    Ok(file) => file,
    Err(..) => { println!("Error: File {:?} not found.", fname); return 2; }
  };

  let mut in_trains_table = false;
  let mut trains = Vec::new();

  ods::parse(file, &mut |event| {
    match event {
      ods::Event::StartTable(name) => {
        if name == "Trains" { in_trains_table = true; }
        println!("Processing table {}...", name);
      },
      ods::Event::Row(values) => {
        if !in_trains_table { return; }

        let vals : Vec<&str> = values.iter().map(|&ref s| (*s).as_str()).collect();

        match vals.as_slice() {
          [num, _, _, _, _, _, _, "traindef", name, _, ..] => {
            match xpln::Train::parse(num, name) {
              Ok(train) => trains.push(train),
              Err(err) => { println!("WARN: Invalid row: {:?}: {}", vals, err) }
            }
          },
          [num, _, station, track, arrival, departure, _, "timetable", name, remark, ..] => {
            let out_of_order = match trains.last() {
              Some(ref train) => train.name != name,
              None => true
            };

            if out_of_order {
              println!("WARN: Out-of-order timetable entry: {:?}", vals)
            } else {
              match xpln::Record::parse(station, track, arrival, departure, remark) {
                Ok(record) => {
                  trains.last_mut().unwrap().add_record(record);
                },
                Err(err) => { println!("WARN: Invalid row: {:?}: {}", vals, err) }
              }
            }
          }
          _ => ()
        }
      },
      ods::Event::EndTable(name) => {
        if name == "Trains" { in_trains_table = false; }
      }
    }
  }).unwrap();

  println!("{:?}", trains);

  return 0;
}
