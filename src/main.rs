use clap::{App, Arg};
use rust_htslib::bam::{
    record::{Cigar, Record},
    Reader, Error as BamError, Read
};
use serde_derive::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct BamQC {
    insert_size: HashMap<i64, usize>,
    r1_cigar: HashMap<u32, HashMap<char, usize>>,
    r2_cigar: HashMap<u32, HashMap<char, usize>>,
}

impl BamQC {
    fn new() -> BamQC {
        BamQC {
            insert_size: HashMap::new(),
            r1_cigar: HashMap::new(),
            r2_cigar: HashMap::new(),
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
        .map_err(|_| BamError::SetThreads)?;

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
            Err(e) => return Err(Error::BamError(e)),
            Ok(true) => {
                let in_size = record.insert_size();
                if in_size > 0 {
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

                let cigar_iter: &mut dyn Iterator<Item = &Cigar> = if record.is_reverse() {
                    a = cv.iter().rev();
                    &mut a
                } else {
                    b = cv.iter();
                    &mut b
                };

                let mut cycle = 1;
                let count = if record.is_first_in_template() {
                    &mut qc.r1_cigar
                } else {
                    &mut qc.r2_cigar
                };

                for c in cigar_iter {
                    for _ in 0..c.len() {
                        let pos = count.entry(cycle).or_insert_with(HashMap::new);
                        pos.entry(c.char()).and_modify(|e| *e += 1).or_insert(1);

                        // Deletions are not a machine cycle
                        // If deletions happen, they are counted in the cycle that happened before
                        if c.char() != 'D' {
                            cycle += 1;
                        }
                    }
                }
            }
            Ok(false) => break
        }
    }

    let serialized = serde_json::to_string(&qc)?;
    println!("{}", serialized);
    Ok(())
}

#[derive(Debug)]
enum Error {
    BamError(BamError),
    JSONError(serde_json::error::Error),
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Self {
        Error::JSONError(error)
    }
}

impl From<BamError> for Error {
    fn from(error: BamError) -> Self {
        Error::BamError(error)
    }
}
