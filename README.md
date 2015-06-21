# xpln2bfo

[![Build Status](https://travis-ci.org/jgraichen/xpln2bfo.svg?branch=master)](https://travis-ci.org/jgraichen/xpln2bfo) [![Build status](https://ci.appveyor.com/api/projects/status/hhxl44komt45clpb?svg=true)](https://ci.appveyor.com/project/jgraichen/xpln2bfo)

Small utility to convert XPLAN timetable spreadsheets (ods) to BFO text documents usable for RgZm.

## Usage

	$ ./xpln2bfo <spreadsheet> <outdir>

## Known limitations

* As of now only BFO text documents are emitted; no RgZm configuration
* Next and previous stations are empty for all timetable entries
* Untested code; only manual tested with single timetable
* Some dependencies seem to not compile on M$ Windows(R)(C)(TM) (See appveyor build status)
* Arrival/Departure times may be incorrectly compacted for fiddleyards

## Build yourself

*xpln2bfo* is written in [Rust](http://rust-lang.org) and can be compiled using `cargo`:

	$ cargo build --release

## License

Copyright (C) 2015 Jan Graichen

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
