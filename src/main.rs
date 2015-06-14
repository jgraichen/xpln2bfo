extern crate zip;
extern crate xml;

use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

mod ods {
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
		pub fn name(&self) -> &str {
			return self.name.as_ref();
		}

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
									Err(ref err) => {
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
}

mod xpln {
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

	impl Xpln {
		pub fn new() -> Xpln {
			let xpln = Xpln {
				trains: HashMap::new(),
				stations: HashMap::new()
			};

			return xpln;
		}

		pub fn stations(&self) -> Vec<&Station> {
			self.stations.values().collect()
		}

		pub fn load(&mut self, document: ods::Spreadsheet) {
			let station_tracks_table = match document.get("StationTrack") {
				Some(table) => table,
				None => { panic!("PANIC: Missing StationTrack table."); }
			};

			//
			// first pass: load stations
			//
			println!("Loading station objects...");

			for row in station_tracks_table.rows() {
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

			//
			// second pass: load station tracks
			//
			println!("Loading station track objects...");

			for row in station_tracks_table.rows() {
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


			let trains_table = match document.get("Trains") {
				Some(table) => table,
				None => { panic!("PANIC: Missing Trains table."); }
			};

			//
			// First pass: load train definitions
			//
			println!("Loading traindef objects...");

			for row in trains_table.rows() {
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

			//
			// second pass: load timetable objects
			//
			println!("Loading timetable objects...");

			for row in trains_table.rows() {
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

		fn add_train(&mut self, train: Train) {
			self.trains.insert(train.number, train);
		}

		fn add_station(&mut self, station: Station) {
			self.stations.insert(String::from(station.name.as_ref()), station);
		}

		fn get_station_mut(&mut self, name: &str) -> Option<&mut Station> {
			return self.stations.get_mut(name);
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

	#[derive(Debug)]
	pub struct Train{
		pub number: usize,
		pub class: String,
		pub remark: String,
		pub timetables: Vec<Timetable>
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

	#[derive(Debug)]
	pub struct Station {
		pub name: String,
		pub remark: String,
		pub tracks: Vec<Track>
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

	#[derive(Debug)]
	pub struct Track {
		pub name: String,
		pub owner: String,
		pub station: String,
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

	#[derive(Debug)]
	pub struct Timetable {
		pub train: usize,
		pub track: String,
		pub remark: String,
		pub station: String,
		pub arrival: String,
		pub departure: String,
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
}

fn main() {
	std::process::exit(run());
}

fn run() -> i32 {
	let args: Vec<_> = std::env::args().collect();
	if args.len() < 2 {
		println!("Usage: {} <input> [<outdir>]", args[0]);
		return 1;
	}

	let fname = PathBuf::from(&*args[1]);
	let file = match File::open(&fname) {
		Ok(file) => file,
		Err(..) => { println!("Error: File {:?} not found.", fname); return 2; }
	};

	println!("Loading {:?}...", fname.to_str().unwrap());

	let document = ods::parse(file).unwrap();

	println!("Extracting XPLN objects...");

	let mut xpln = xpln::Xpln::new();
	xpln.load(document);

	//
	// Export BFO
	//

	let outdir = match args.get(2) {
		Some(dir) => PathBuf::from(dir),
		None => fname.with_extension("")
	};

	fs::create_dir_all(&outdir).unwrap();

	for station in xpln.stations.values() {
		let mut file = File::create(&outdir.join(format!("{}.bfo", &station.name))).unwrap();
		let mut tts  = Vec::new();

		for train in xpln.trains.values() {
			for timetable in train.timetables.iter() {
				if timetable.station == station.name {
					tts.push(timetable);
				}
			}
		}

		tts.sort_by(|ref t0, ref t1| t0.arrival.cmp(&t1.arrival));

		let mut data  = String::new();
		let     arrow = " -->";

		for timetable in tts {

			let arrival : &str = if timetable.arrival == timetable.departure {
				arrow
			} else {
				timetable.arrival.as_ref()
			};

			let departure : &str = &timetable.departure;
			let train     : &str = &xpln.trains[&timetable.train].name();
			let track     : &str = &timetable.track;
			let remark    : &str = &timetable.remark;

			let line = format!("{arrival}\t{departure}\t{train}\t\t\t{track}\t\t\t\t\t{remark}\n",
				arrival=arrival, departure=departure, track=track, train=train, remark=remark
			);

			data.push_str(&line);
		}

		file.write_all(data.as_bytes()).unwrap();
	}

	// println!("{:?}", outdir);
	// println!("{}", xpln);

	return 0;
}
