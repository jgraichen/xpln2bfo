extern crate zip;
extern crate xml;

use std::fs;

mod ods {
  use std::io;
  use std::io::{Read, Seek};
  use std::convert;
  use std::mem;

  use zip::read::ZipArchive;
  use zip::result::ZipError;

  use xml;
  use xml::reader::EventReader;
  use xml::reader::events::*;
  use xml::attribute::OwnedAttribute;

  #[derive(Debug)]
  pub enum Error {
    IoError(io::Error),
    ZipError(ZipError),
    XmlError(xml::common::Error),
    InvalidMimetype(String),
    InvalidSpreadsheet(String)
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

  #[derive(Debug)]
  pub struct Spreadsheet {
    tables: Vec<Table>
  }

  impl Spreadsheet {
    fn new() -> Spreadsheet {
      return Spreadsheet { tables: Vec::new() };
    }

    pub fn get(&self, table_name: &str) -> Option<&Table> {
      for i in 0..self.tables.len() {
        if self.tables[i].name == table_name {
          return self.tables.get(i);
        }
      }
      return None;
    }
  }

  #[derive(Debug)]
  pub struct Table {
    name: String,
    rows: Vec<Row>
  }

  impl Table {
    pub fn name(&self) -> &str {
      return self.name.as_ref();
    }

    pub fn rows(&self) -> &Vec<Row> {
      return &self.rows;
    }
  }

  #[derive(Debug)]
  pub struct Row {
    pub number: usize,
    pub values: Vec<String>
  }

  impl ::std::fmt::Display for Row {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
      try!(write!(fmt, "row {}: {}", self.number, self.values.connect(", ")));

      Ok(())
    }
  }

  #[derive(PartialEq, Debug)]
  enum Token { Bottom, Table, Row, Cell }

  pub fn parse<T: Read+Seek>(file: T) -> Result<Spreadsheet, Error> {
    let mut archive = try!(ZipArchive::new(file));

    {
      let mut mime = String::new();
      let mut file = try!(archive.by_name("mimetype"));

      try!(file.read_to_string(&mut mime));

      if mime != "application/vnd.oasis.opendocument.spreadsheet" {
        return Err(Error::InvalidMimetype(mime));
      }
    }

    let file  = try!(archive.by_name("content.xml"));

    let mut stack       = Vec::new();
    let mut parser      = EventReader::new(file);

    let mut value       = String::new();
    let mut values      = Vec::new();
    let mut rows        = Vec::new();
    let mut table       = None;
    let mut spreadsheet = Spreadsheet::new();

    stack.push(Token::Bottom);

    for event in parser.events() {
      match event {
        XmlEvent::StartElement { name, attributes, namespace: _ } => {
          match name.local_name.as_ref() {
            "table" => {
              assert_eq!(Token::Bottom, *stack.last().unwrap());

              let name  = match extract_attribute(&attributes, "name") {
                Some(name) => name,
                None => return Err(Error::InvalidSpreadsheet(String::from("Table without name attribute.")))
              };

              table = Some(name);

              stack.push(Token::Table);
            },
            "table-row" => {
              assert_eq!(Token::Table, *stack.last().unwrap());
              stack.push(Token::Row);
            },
            "table-cell" => {
              assert_eq!(Token::Row, *stack.last().unwrap());
              stack.push(Token::Cell);
            },
            _ => ()
          }
        },
        XmlEvent::EndElement { name } => {
          match name.local_name.as_ref() {
            "table"      => {
              assert_eq!(Token::Table, stack.pop().unwrap());

              let name = mem::replace(&mut table, None).unwrap();
              let rvec = mem::replace(&mut rows, Vec::new());

              let table = Table {
                name: name,
                rows: rvec
              };

              spreadsheet.tables.push(table);
            },
            "table-row"  => {
              assert_eq!(Token::Row, stack.pop().unwrap());

              let vvec  = mem::replace(&mut values, Vec::new());
              let index = rows.len();

              rows.push(Row { number: index, values: vvec });
            },
            "table-cell" => {
              assert_eq!(Token::Cell, stack.pop().unwrap());

              let val = mem::replace(&mut value, String::new());

              values.push(val);
            },
            _ => ()
          }
        },
        XmlEvent::CData(data) |
        XmlEvent::Characters(data) => {
          match *stack.last().unwrap() {
            Token::Cell => value.push_str(data.as_ref()),
            _ => ()
          }
        },
        XmlEvent::Error(err) => return Err(Error::XmlError(err)),
        _ => ()
      }
    }

    return Ok(spreadsheet);
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
  use std::collections::HashMap;

  use ods;

  #[derive(Debug)]
  pub struct Xpln {
    trains: HashMap<usize, Train>,
    stations: HashMap<String, Station>,
  }

  impl Xpln {
    pub fn new() -> Xpln {
      let xpln = Xpln {
        trains: HashMap::new(),
        stations: HashMap::new()
      };

      return xpln;
    }

    pub fn load(&mut self, document: ods::Spreadsheet) {
      let trains_table = document.get("Trains").unwrap();

      for row in trains_table.rows() {
        // Require at least 11 fields for matching and parsing
        if row.values.len() < 11 { continue; }

        match row.values[7].as_ref() {
          "traindef" => {
            let train = Train::parse(&row.values);

            match train {
              Ok(train) => { self.trains.insert(train.number, train); },
              Err(err) => { println!("ERR: Invalid traindef: {}\n     {}", err, row); }
            }
          },
          // "timetable" => {
          //   let timetable = Timetable::parse(&row.values);

          //   match timetable {
          //     Ok(train) => { self.timetables.push(timetable); },
          //     Err(err) => { println!("ERR: Invalid timetable: {}\n     {}", err, row); }
          //   }
          // }
          _ => ()
        }
      }
    }
  }

  #[derive(Debug)]
  pub struct Train{
    pub number: usize,
    pub name: String
  }

  impl Train {
    fn parse(values: &Vec<String>) -> Result<Train, ParseIntError> {
      let v : Vec<&str> = values.iter().map(|s| s.as_ref()).collect();

      Ok(Train {
        number: try!(usize::from_str(v[0])),
        name: String::from(v[8])
      })
    }
  }

  #[derive(Debug)]
  pub struct Station {
    pub name: String,
    pub remark: String,
  }

  #[derive(Debug)]
  pub struct Track {
    pub name: String,
    pub owner: String,
    pub station: String,
  }

  #[derive(Debug)]
  pub struct Timetable {
    pub train: usize,
    pub track: String,
    pub remark: String,
    pub station: String,
    pub arrival: String,
    pub departure: String,
  }

  // impl Timetable {
  //   fn parse(values: &Vec<String>) -> Result<Train, ParseIntError> {
  //     Ok(Timetable {
  //       train: try!(usize::from_str(values.get(0).unwrap())),
  //       track: String::from(values.get(3).unwrap().as_ref()),
  //       remark: String::from(values.get(10).unwrap().as_ref()),
  //       station: String::from(values.get(2).unwrap().as_ref()),
  //     });
  //   }
  // }
}

fn main() {
  std::process::exit(run());
}

fn run() -> i32 {
  let args: Vec<_> = std::env::args().collect();
  if args.len() < 3 {
    println!("Usage: {} <input> <outdir>", args[0]);
    return 1;
  }

  let fname = std::path::Path::new(&*args[1]);
  let file = match fs::File::open(&fname) {
    Ok(file) => file,
    Err(..) => { println!("Error: File {:?} not found.", fname); return 2; }
  };

  let outdir = std::path::Path::new(&*args[2]);
  std::fs::create_dir_all(outdir).unwrap();

  println!("Loading {:?}...", fname.to_str().unwrap());

  let document = ods::parse(file).unwrap();

  println!("Extracting XPLN objects...");

  let mut xpln = xpln::Xpln::new();
  xpln.load(document);

  println!("{:?}", xpln);

  return 0;
}
