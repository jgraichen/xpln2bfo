extern crate zip;
extern crate xml;

use std::io::Read;
use std::fs;

use xml::reader::EventReader;
use xml::reader::events::*;

mod xpln {
    #[derive(Eq, Hash)]
    pub struct Train {
        number: u32,
        name: String,
    }

    impl Train {
        fn new(number: u32, name: String) -> Train {
            Train { number: number, name: name }
        }
    }

    impl PartialEq<Train> for Train {
        fn eq(&self, other: &Train) -> bool {
            self.number == other.number
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

    let mut archive = zip::ZipArchive::new(file).unwrap();

    {
        let mut file = match archive.by_name("mimetype") {
            Ok(file) => file,
            Err(..) => { println!("Error: Invalid file, no mimetype found."); return 2; }
        };

        let mut mimetype = String::new();
        file.read_to_string(&mut mimetype).unwrap();


        if mimetype != "application/vnd.oasis.opendocument.spreadsheet" {
            println!("Error: Not a spreadsheet.")
        }
    }

    {
        let file = match archive.by_name("content.xml") {
            Ok(file) => file,
            Err(..) => { println!("Error: Invalid file, no content.xml found."); return 2; }
        };

        let mut parser = EventReader::new(file);
        let mut trains = Vec::new();

        {
            parse_xpln(&mut parser, &mut trains);
        }

    }

    // TODO

    return 0;
}

fn parse_xpln<B: std::io::Read>(parser: &mut EventReader<B>, trains: &mut Vec<xpln::Train>) {
    loop {
        match parser.next() {
            XmlEvent::StartElement { name, attributes, namespace: _ } => {
                if name.local_name == "table" {
                    for attr in attributes.iter() {
                        if attr.name.local_name == "name" {
                            println!("Processing table {} ...", attr.value);

                            match attr.value.as_ref() {
                                "Trains" => parse_trains(parser, trains),
                                _ => {}
                            }
                        }
                    }
                }
            }
            XmlEvent::Error(e) => {
                println!("Error: {}", e);
                return;
            }
            XmlEvent::EndDocument => { return; }
            _ => {}
        }
    }
}

fn parse_trains<B: std::io::Read>(parser: &mut EventReader<B>, trains: &mut Vec<xpln::Train>) {
    loop {
        match parser.next() {
            XmlEvent::StartElement { name, attributes: _, namespace: _ } => {
                if name.local_name == "table-row" {
                    let values = parse_cells(parser);
                    println!("{:?}", values);
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "table" {
                    return;
                }
            }
            XmlEvent::Error(_) => { return; }
            _ => {}
        }
    }
}


fn parse_cells<B: std::io::Read>(parser: &mut EventReader<B>) -> Vec<String> {
    let mut values = Vec::new();
    loop {
        match parser.next() {
            XmlEvent::StartElement { name, attributes: _, namespace: _ } => {
                if name.local_name == "table-cell" {
                    values.push(parse_cell(parser));
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "table-row" {
                    break;
                }
            }
            XmlEvent::Error(_) => { break; }
            _ => {}
        }
    }

    return values;
}

fn parse_cell<B: std::io::Read>(parser: &mut EventReader<B>) -> String {
    let mut inside = false;
    loop {
        match parser.next() {
            XmlEvent::StartElement { name, attributes: _, namespace: _ } => {
                if name.local_name == "p" {
                    inside = true;
                }
            }
            XmlEvent::EndElement { name } => {
                if name.local_name == "table-cell" {
                    break;
                }
                if name.local_name == "p" {
                    inside = false;
                }
            }
            XmlEvent::Characters(string) => {
                if inside {
                    return string;
                }
            }
            XmlEvent::Error(_) => { break; }
            _ => {}
        }
    }

    return String::new();
}
