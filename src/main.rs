use clap::{App, Arg};
use rust_htslib::bam::{
    record::{Cigar, Record},
    BGZFError, ReadError, Reader, ReaderPathError, ThreadingError,
};
use rust_htslib::prelude::*;
use serde_derive::Serialize;
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct BamQC {
    insert_size: HashMap<i32, usize>,
    cigar: HashMap<u32, HashMap<char, usize>>,
}

impl BamQC {
    fn new() -> BamQC {
        BamQC {
            insert_size: HashMap::new(),
            cigar: HashMap::new(),
        }
    }
}

fn main() -> Result<(), Error> {
    let arg = App::new("Bam Histogram Calculator")
        .version("0.1")
        .author("Savo Lazic <savo.lazic@oicr.on.ca")
        .about(
            "Collects various statistics from a BAM file that allow creation of histograms for QC \
             purposes",
        )
        .arg(
            Arg::with_name("FILE")
                .help("File path to BAM file. Use '-' for STDIN")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("threads")
                .help("Number of threads to use")
                .short("t")
                .long("threads")
                .default_value("1"),
        )
        .get_matches();

    let path = arg.value_of("FILE").unwrap();
    let thread: usize = arg
        .value_of("threads")
        .unwrap()
        .parse()
        .map_err(|_| ThreadingError::Some)?;

    let mut bam = if path == "-" {
        Reader::from_stdin()?
    } else {
        Reader::from_path(path)?
    };
    bam.set_threads(thread)?;

    let mut qc = BamQC::new();
    let mut record = Record::new();

    loop {
        match bam.read(&mut record) {
            Err(ReadError::NoMoreRecord) => break,
            Err(e) => Err(Error::ReadError(e))?,
            Ok(_) => {
                let in_size = record.insert_size();
                if in_size >= 0 {
                    qc.insert_size
                        .entry(in_size)
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                };

                // https://stackoverflow.com/questions/
                // 29760668/conditionally-iterate-over-one-of-several-possible-iterators
                let mut a;
                let mut b;
                let cv = record.cigar();

                let cigar_iter: &mut Iterator<Item = &Cigar> = if record.is_reverse() {
                    a = cv.iter().rev();
                    &mut a
                } else {
                    b = cv.iter();
                    &mut b
                };

                let mut cycle = 1;
                for c in cigar_iter {
                    for _ in 0..c.len() {
                        let pos = qc.cigar.entry(cycle).or_insert(HashMap::new());
                        pos.entry(c.char()).and_modify(|e| *e += 1).or_insert(1);
                        cycle += 1;
                    }
                }
            }
        }
    }

    let serialized = serde_json::to_string(&qc)?;
    println!("{}", serialized);
    Ok(())
}

#[derive(Debug)]
enum Error {
    ReaderPathError(ReaderPathError),
    ThreadingError(ThreadingError),
    ReadError(ReadError),
    JSONError(serde_json::error::Error),
}

impl From<ReaderPathError> for Error {
    fn from(error: ReaderPathError) -> Self {
        Error::ReaderPathError(error)
    }
}

impl From<ThreadingError> for Error {
    fn from(error: ThreadingError) -> Self {
        Error::ThreadingError(error)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Self {
        Error::JSONError(error)
    }
}

impl From<BGZFError> for Error {
    fn from(error: BGZFError) -> Self {
        Error::ReaderPathError(ReaderPathError::BGZFError(error))
    }
}
