use std::str::FromStr;
use std::num::ParseIntError;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Error};
use std::iter::Iterator;

use ods;

#[derive(Debug)]
pub struct Xpln {
    pub trains: HashMap<usize, Train>,
    pub stations: HashMap<String, Station>,
}

#[derive(Debug)]
pub struct Train{
    pub number: usize,
    pub class: String,
    pub remark: String,
    pub timetables: Vec<Timetable>
}

#[derive(Debug)]
pub struct Station {
    pub name: String,
    pub remark: String,
    pub tracks: Vec<Track>
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

impl Xpln{
    pub fn new() -> Xpln {
        let xpln = Xpln {
            trains: HashMap::new(),
            stations: HashMap::new()
        };

        return xpln;
    }

    fn add_train(&mut self, train: Train) {
        self.trains.insert(train.number, train);
    }

    fn add_station(&mut self, station: Station) {
        self.stations.insert(String::from(station.name.as_ref()), station);
    }

    fn get_station_mut(&mut self, name: &str) -> Option<&mut Station> {
        return self.stations.get_mut(name);
    }

    pub fn load(&mut self, document: &ods::Spreadsheet) {
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
                    let station = Station::parse(
                        &row.values[0],
                        &row.values[4]
                    );

                    match station {
                        Ok(station) => {
                            self.add_station(station);
                        },
                        Err(err) => {
                            println!("ERR: Invalid station object: {}\n     {}", err, row);
                        }
                    }
                },
                _ => ()
            }
        }
    }

    fn load_station_tracks(&mut self, document: &ods::Spreadsheet) {
        let table = document
            .get("StationTrack")
            .expect("PANIC: Missing StationTrack table.");

        println!("Loading station track objects...");

        for row in table.rows() {
            if row.values.len() < 7 { continue }

            match row.values[5].as_ref() {
                "Track" => {
                    let track = Track::parse(
                        &row.values[0],
                        &row.values[6],
                        &row.values.get(7).unwrap_or(&String::new())
                    );

                    match self.get_station_mut(&track.station) {
                        Some(station) => {
                            station.add_track(track);
                        },
                        None => {
                            println!("ERR: Illegal station reference in track object.\n     {}", row);
                        }
                    };
                },
                _ => ()
            }
        }
    }

    fn load_traindefs(&mut self, document: &ods::Spreadsheet) {
        let table = document
            .get("Trains")
            .expect("PANIC: Missing Trains table.");

        println!("Loading traindef objects...");

        for row in table.rows() {
            // Require at least 10 fields for matching and parsing
            if row.values.len() < 10 { continue }

            match row.values[8].as_ref() {
                "traindef" => {
                    let train = Train::parse(
                        &row.values[0],
                        &row.values[9],
                        &row.values.get(10).unwrap_or(&String::new())
                    );

                    match train {
                        Ok(train) => {
                            self.add_train(train);
                        },
                        Err(err) => {
                            println!("ERR: Invalid traindef: {}\n     {}", err, row);
                        }
                    };

                },
                _ => ()
            }
        }
    }

    fn load_timetables(&mut self, document: &ods::Spreadsheet) {
        let table = document
            .get("Trains")
            .expect("PANIC: Missing Trains table.");

        println!("Loading timetable objects...");

        for row in table.rows() {
            if row.values.len() < 11 { continue }

            match row.values[8].as_ref() {
                "timetable" => {
                    let timetable = match Timetable::parse(&row.values) {
                        Ok(timetable) => timetable,
                        Err(err) => {
                            println!("ERR: Invalid timetable: {}\n     {}", err, row);
                            continue;
                        }
                    };

                    match self.trains.get_mut(&timetable.train) {
                        Some(train) => {
                            train.timetables.push(timetable);
                        },
                        None => {
                            println!("ERR: Illegal train reference in timetable object: {}\n     {}", row.values[0], row);
                        }
                    };
                },
                _ => ()
            }
        }
    }
}

impl Display for Xpln {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(writeln!(f, "Xpln ["));
        try!(writeln!(f, "  Stations:"));

        let mut stations : Vec<_> = self.stations.keys().collect();
        stations.sort();

        for name in stations {
            let station = self.stations.get(name).unwrap();
            try!(writeln!(f, "    {}", station));
        }

        try!(writeln!(f, ""));
        try!(writeln!(f, "  Trains:"));

        let mut trains : Vec<_> = self.trains.keys().collect();
        trains.sort();

        for number in trains {
            let train = self.trains.get(number).unwrap();
            try!(writeln!(f, "    {}", train));
        }

        try!(writeln!(f, "]"));

        Ok(())
    }
}

impl Train {
    fn new<S0, S1>(number: usize, class: S0, remark: S1) -> Train
                where S0 : Into<String>, S1 : Into<String> {
        Train {
            number: number,
            class: class.into(),
            remark: remark.into(),
            timetables: Vec::new()
        }
    }

    pub fn name(&self) -> String {
        format!("{} {}", self.class, self.number)
    }

    fn parse(number: &str, name: &str, remark: &str) -> Result<Train, ParseIntError> {
        let class : String = name.chars().take_while(|c| c != &' ').collect();

        Ok(Train::new(try!(usize::from_str(number)), class, remark))
    }
}

impl Display for Train {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(write!(f, "{:>3} {:>4} : {}", self.class, self.number, self.remark));
        Ok(())
    }
}

impl Station {
    fn new<S: Into<String>>(name: S, remark: S) -> Station {
        Station {
            name: name.into(),
            remark: remark.into(),
            tracks: Vec::new()
        }
    }

    fn parse(name: &str, remark: &str) -> Result<Station, ParseIntError> {
        Ok(Station::new(name, remark))
    }

    fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }
}

impl Display for Station {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(write!(f, "{:>3} : {}", self.name, self.remark));
        Ok(())
    }
}

impl Track {
    fn new<S: Into<String>>(station: S, name: S, owner: S) -> Track {
        Track {
            name: name.into(),
            owner: owner.into(),
            station: station.into()
        }
    }

    fn parse(station: &str, name: &str, owner: &str) -> Track {
        Track::new(station, name, owner)
    }
}

impl Timetable {
    fn new(train: usize, track: String, station: String, arrival: String,
                 departure: String, remark: String) -> Timetable {
        Timetable {
            train: train,
            track: track,
            station: station,
            arrival: arrival,
            departure: departure,
            remark: remark
        }
    }

    fn parse(values: &Vec<String>) -> Result<Timetable, ParseIntError> {
        Ok(Timetable::new(
            try!(usize::from_str(values.get(0).unwrap())),
            String::from(values.get(3).unwrap().as_ref()),
            String::from(values.get(2).unwrap().as_ref()),
            String::from(values.get(4).unwrap().as_ref()),
            String::from(values.get(5).unwrap().as_ref()),
            String::from(values.get(10).unwrap().as_ref()),
        ))
    }
}
