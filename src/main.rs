extern crate zip;
extern crate xml;

use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

mod ods;
mod xpln;

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
	xpln.load(&document);

	//
	// Export BFO
	//

	let outdir = match args.get(2) {
		Some(dir) => PathBuf::from(dir),
		None => fname.with_extension("")
	};

	println!("Write BFOs...");

	fs::create_dir_all(&outdir).unwrap();

	for station in xpln.stations.values() {
		let path = outdir.join(format!("{}.txt", &station.name));

		println!("  {:?}", &path);

		let mut file = File::create(&path).unwrap();
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

		for timetable in tts {
			let arrival   : &str = &timetable.arrival;
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

	println!("Done.");

	return 0;
}
