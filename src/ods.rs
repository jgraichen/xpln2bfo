use std::io;
use std::io::{Read, Seek};
use std::convert;
use std::mem;
use std::str::FromStr;
use std::error::Error as _StdError;

use zip::read::ZipArchive;
use zip::result::ZipError;

use xml;
use xml::reader::EventReader;
use xml::reader::events::*;
use xml::attribute::OwnedAttribute;

#[derive(Debug)]
pub struct Error {
    description: String,
    cause: Option<ErrorCause>
}
impl Error {
    fn new(description: String, cause: Option<ErrorCause>) -> Error {
        Error { description: description, cause: cause }
    }
}

impl convert::From<ZipError> for Error {
    fn from(err: ZipError) -> Error {
        Error::new(
            String::from(err.description()),
            Some(ErrorCause::Zip(err))
        )
    }
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::new(
            String::from(err.description()),
            Some(ErrorCause::Io(err))
        )
    }
}

impl convert::From<xml::common::Error> for Error {
    fn from(err: xml::common::Error) -> Error {
        Error::new(
            String::from(err.description()),
            Some(ErrorCause::Xml(err))
        )
    }
}

impl convert::From<String> for Error {
    fn from(err: String) -> Error {
        Error::new(err, None)
    }
}

impl convert::From<&'static str> for Error {
    fn from(err: &str) -> Error {
        Error::new(err.to_string(), None)
    }
}

#[derive(Debug)]
enum ErrorCause {
    Io(io::Error),
    Zip(ZipError),
    Xml(xml::common::Error)
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
    pub fn rows(&self) -> &Vec<Row> {
        return &self.rows;
    }
}

impl ::std::fmt::Display for Table {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        let num_cols = self.rows.iter().fold(0, |max, r| {
            let len = r.values.len();
            if len < max { max } else { len }
        });

        let lengths : Vec<usize> = (0..num_cols).map(|index|
            self.rows.iter().fold(0, |max, r| {
                match r.values.get(index) {
                    Some(v) => if v.len() < max { max } else { v.len() },
                    None => max
                }
            })
        ).collect();

        let delimiter = "|";
        let headline  = format!(" {} ({}:{}) ", self.name, self.rows.len(), num_cols);
        let width     = 1 + lengths.iter().fold(0, |a, l| a + l + 3);

        try!(write!(fmt, "{:=^1$}", headline, width));
        try!(write!(fmt, "\n"));

        let nil = String::new();

        for row in self.rows.iter() {
            try!(write!(fmt, "{}", delimiter));

            for index in 0..num_cols {
                let val = row.values.get(index).unwrap_or(&nil);

                try!(write!(fmt, " {:<1$} ", val, lengths[index]));
                try!(write!(fmt, "{}", delimiter));
            }

            try!(write!(fmt, "\n"));
        }

        try!(write!(fmt, "{:=^1$}", "", width));

        Ok(())
    }
}


#[derive(Debug)]
pub struct Row {
    pub number: usize,
    pub values: Vec<String>
}

impl ::std::fmt::Display for Row {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        try!(write!(fmt, "{}: {:?}", self.number, self.values));

        Ok(())
    }
}

#[derive(PartialEq, Debug)]
enum Token {
    Bottom,
    Table,
    Row,
    Cell {
        number_columns_repeated: usize
    }
}

pub fn parse<T: Read+Seek>(file: T) -> Result<Spreadsheet, Error> {
    let mut archive = try!(ZipArchive::new(file));

    {
        let mut mime = String::new();
        let mut file = try!(archive.by_name("mimetype"));

        try!(file.read_to_string(&mut mime));

        if mime != "application/vnd.oasis.opendocument.spreadsheet" {
            return Err(Error::from(format!("Invalid mimetype: {}", mime)));
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
                            None => return Err(Error::from("Table without name attribute."))
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

                        let number_columns_repeated = match extract_attribute(&attributes, "number-columns-repeated") {
                            Some(value) => match usize::from_str(value.as_ref()) {
                                Ok(value) => value,
                                Err(_) => {
                                    return Err(Error::from(
                                        "Parser error on table:number-columns-repeated: Not a valid number."
                                    ));
                                }
                            },
                            None => 1
                        };

                        stack.push(Token::Cell {
                            number_columns_repeated: number_columns_repeated
                        });
                    },
                    _ => ()
                }
            },
            XmlEvent::EndElement { name } => {
                match name.local_name.as_ref() {
                    "table" => {
                        assert_eq!(Token::Table, stack.pop().unwrap());

                        let name = mem::replace(&mut table, None).unwrap();
                        let rvec = mem::replace(&mut rows, Vec::new());

                        let table = Table {
                            name: name,
                            rows: rvec
                        };

                        spreadsheet.tables.push(table);
                    },
                    "table-row" => {
                        assert_eq!(Token::Row, stack.pop().unwrap());

                        let     index = rows.len();
                        let mut vvec  = mem::replace(&mut values, Vec::new());
                        let     last  = vvec.iter().rposition(|&ref v : &String| v.len() > 0usize).unwrap_or(0);

                        vvec.truncate(last + 1);

                        rows.push(Row { number: index, values: vvec });
                    },
                    "table-cell" => {
                        match stack.pop() {
                            Some(Token::Cell {
                                number_columns_repeated
                            }) => {
                                let val = mem::replace(&mut value, String::new());

                                for _ in 0..number_columns_repeated {
                                    values.push(String::from(val.as_ref()));
                                }
                            },
                            _ => { panic!("Invalid ODS parser state"); }
                        }
                    },
                    _ => ()
                }
            },
            XmlEvent::CData(data) |
            XmlEvent::Characters(data) => {
                match *stack.last().unwrap() {
                    Token::Cell{..} => value.push_str(data.as_ref()),
                    _ => ()
                }
            },
            XmlEvent::Error(err) => return Err(Error::from(err)),
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
