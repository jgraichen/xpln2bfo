extern crate zip;
extern crate xml;
extern crate xpln2bfo;

use std::io::Read;
use std::fs;
use std::result;

use xml::reader::EventReader;
use xml::reader::events::*;

use xpln2bfo::odf;
use xpln2bfo::odf::ods;

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

    let odf = odf::File::new(file).unwrap();
    let ods = ods::Parser::new(odf).unwrap();

    println!("{}", odf.mimetype);

    // loop {
    //     match ods.next() {
    //         ods::Event::StartTable { name } => {
    //             println!("Start table {}", name);
    //         }
    //         ods::Event::Error(e) => {
    //             println!("Error: {}", e);
    //             break;
    //         }
    //         _ => {}
    //     }
    // }

    // let mut archive = zip::ZipArchive::new(file).unwrap();

    // {
    //     let mut file = match archive.by_name("mimetype") {
    //         Ok(file) => file,
    //         Err(..) => { println!("Error: Invalid file, no mimetype found."); return 2; }
    //     };

    //     let mut mimetype = String::new();
    //     file.read_to_string(&mut mimetype).unwrap();


    //     if mimetype != "application/vnd.oasis.opendocument.spreadsheet" {
    //         println!("Error: Not a spreadsheet.")
    //     }
    // }

    // {
    //     let file = match archive.by_name("content.xml") {
    //         Ok(file) => file,
    //         Err(..) => { println!("Error: Invalid file, no content.xml found."); return 2; }
    //     };

    //     let mut parser = EventReader::new(file);

    //     for e in parser.events() {
    //         match e {
    //             XmlEvent::StartElement { name, attributes, namespace: _ } => {
    //                 if name.local_name == "table" {
    //                     println!("{} => {:?}", name.local_name, attributes);
    //                 }
    //             }
    //             // XmlEvent::EndElement { name } => {
    //             //     depth -= 1;
    //             //     println!("{}/{}", indent(depth), name);
    //             // }
    //             XmlEvent::Error(e) => {
    //                 println!("Error: {}", e);
    //                 break;
    //             }
    //             _ => {}
    //         }
    //     }
    // }

    // TODO

    return 0;
}
