use std::str::FromStr;
use std::num::ParseIntError;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Error};
use std::iter::Iterator;

use ods;

#[derive(Debug)]
pub struct Xpln<'a> {
    trains: Vec<Train>,
    stations: Vec<Station>,

    train_map: HashMap<usize, &'a Train>,
    station_map: HashMap<&'a str, &'a Station>,
}

#[derive(Debug)]
pub struct Train{
    pub number: usize,
    pub class: String,
    pub remark: String,
    pub timetable: Option<Box<Timetable>>
}

#[derive(Debug)]
pub struct Station {
    pub name: String,
    pub remark: String,
    pub tracks: Vec<Track>,
    pub timetables: Vec<Box<Timetable>>
}

#[derive(Debug)]
pub struct Track {
    pub name: String,
    pub owner: String,
    pub station: Box<Station>,
}

#[derive(Debug)]
pub struct Timetable {
    pub next: Option<Box<Timetable>>,
    pub prev: Option<Box<Timetable>>,
    pub train: Box<Train>,
    pub track: Box<Track>,
    pub remark: String,
    pub arrival: String,
    pub departure: String,
}

impl<'a> Xpln<'a> {
    pub fn new() -> Xpln<'a> {
        let xpln = Xpln {
            trains: Vec::new(),
            stations: Vec::new(),

            train_map: HashMap::new(),
            station_map: HashMap::new()
        };

        return xpln;
    }

    pub fn trains(&self) -> Vec<&Train> {
        self.trains.iter().map(|&ref t| t).collect()
    }

    pub fn stations(&self) -> Vec<&Station> {
        self.stations.iter().map(|&ref s| s).collect()
    }

    pub fn train(&self, number: usize) -> Option<&Train> {
        self.train_map.get(&number).map(|t| *t)
    }

    pub fn station(&self, name: &str) -> Option<&Station> {
        self.station_map.get(name).map(|s| *s)
    }

    pub fn load(&mut self, document: &ods::Spreadsheet) {
        let station_tracks_table = match document.get("StationTrack") {
            Some(table) => table,
            None => { panic!("PANIC: Missing StationTrack table."); }
        };

        self.load_stations(document);
        self.load_station_tracks(document);
        self.load_traindefs(document);
        self.load_timetables(document);
    }

    fn load_stations(&mut self, document: &ods::Spreadsheet) {
        let table = document
            .get("StationTrack")
            .expect("PANIC: Missing StationTrack table.");

        println!("Loading station objects...");

        for row in table.rows() {
            if row.values.len() < 6 { continue }

            match row.values[5].as_ref() {
                "Station" => {
                    let name   = row.values[0];
                    let remark = row.values[4];

                    self.stations.push(Station {
                        name: name,
                        remark: remark,
                        tracks: Vec::new(),
                        timetable: Vec::new()
                    });

                    let station : &Station = self.stations.last().unwrap();

                    self.station_map.insert(station.name, station);

                    // let station = Station::parse(
                    //     &self,
                    //     &row.values[0],
                    //     &row.values[4]
                    // );

                    // match station {
                    //     Ok(station) => {
                    //         self.add_station(station);
                    //     },
                    //     Err(err) => {
                    //         println!("ERR: Invalid station object: {}\n     {}", err, row);
                    //     }
                    // }
                },
                _ => ()
            }
        }
    }

    fn load_station_tracks(&mut self, document: &ods::Spreadsheet) {
        // let table = document
        //     .get("StationTrack")
        //     .expect("PANIC: Missing StationTrack table.");

        // println!("Loading station track objects...");

        // for row in table.rows() {
        //     if row.values.len() < 7 { continue }

        //     match row.values[5].as_ref() {
        //         "Track" => {
        //             let track = Track::parse(
        //                 &row.values[0],
        //                 &row.values[6],
        //                 &row.values.get(7).unwrap_or(&String::new())
        //             );

        //             match self.get_station_mut(&track.station) {
        //                 Some(station) => {
        //                     station.add_track(track);
        //                 },
        //                 None => {
        //                     println!("ERR: Illegal station reference in track object.\n     {}", row);
        //                 }
        //             };
        //         },
        //         _ => ()
        //     }
        // }
    }

    fn load_traindefs(&mut self, document: &ods::Spreadsheet) {
        // let table = document
        //     .get("Trains")
        //     .expect("PANIC: Missing Trains table.");

        // println!("Loading traindef objects...");

        // for row in table.rows() {
        //     // Require at least 10 fields for matching and parsing
        //     if row.values.len() < 10 { continue }

        //     match row.values[8].as_ref() {
        //         "traindef" => {
        //             let train = Train::parse(
        //                 &row.values[0],
        //                 &row.values[9],
        //                 &row.values.get(10).unwrap_or(&String::new())
        //             );

        //             match train {
        //                 Ok(train) => {
        //                     self.add_train(train);
        //                 },
        //                 Err(err) => {
        //                     println!("ERR: Invalid traindef: {}\n     {}", err, row);
        //                 }
        //             };

        //         },
        //         _ => ()
        //     }
        // }
    }

    fn load_timetables(&mut self, document: &ods::Spreadsheet) {
        // let table = document
        //     .get("Trains")
        //     .expect("PANIC: Missing Trains table.");

        // println!("Loading timetable objects...");

        // for row in table.rows() {
        //     if row.values.len() < 11 { continue }

        //     match row.values[8].as_ref() {
        //         "timetable" => {
        //             let timetable = match Timetable::parse(&row.values) {
        //                 Ok(timetable) => timetable,
        //                 Err(err) => {
        //                     println!("ERR: Invalid timetable: {}\n     {}", err, row);
        //                     continue;
        //                 }
        //             };

        //             match self.trains.get_mut(&timetable.train) {
        //                 Some(train) => {
        //                     train.timetables.push(timetable);
        //                 },
        //                 None => {
        //                     println!("ERR: Illegal train reference in timetable object: {}\n     {}", row.values[0], row);
        //                 }
        //             };
        //         },
        //         _ => ()
        //     }
        // }
    }
}

impl<'a> Display for Xpln<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        // try!(writeln!(f, "Xpln ["));
        // try!(writeln!(f, "  Stations:"));

        // let mut stations : Vec<_> = self.stations.keys().collect();
        // stations.sort();

        // for name in stations {
        //     let station = self.stations.get(name).unwrap();
        //     try!(writeln!(f, "    {}", station));
        // }

        // try!(writeln!(f, ""));
        // try!(writeln!(f, "  Trains:"));

        // let mut trains : Vec<_> = self.trains.keys().collect();
        // trains.sort();

        // for number in trains {
        //     let train = self.trains.get(number).unwrap();
        //     try!(writeln!(f, "    {}", train));
        // }

        // try!(writeln!(f, "]"));

        Ok(())
    }
}

impl Display for Train {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(write!(f, "{:>3} {:>4} : {}", self.class, self.number, self.remark));
        Ok(())
    }
}

impl Display for Station {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(write!(f, "{:>3} : {}", self.name, self.remark));
        Ok(())
    }
}
