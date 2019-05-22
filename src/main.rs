use rust_htslib::bam::{
    record::{Cigar, Record},
    ReadError, Reader, ReaderPathError, ThreadingError,
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
    let mut bam = Reader::from_path(
        &"/home/slazic/Documents/Code/2019-05-21-bam_hist_rust/bam_histogram_rust/files/\
          SWID_13766177_GLCS_0001_Lv_R_PE_280_WG_DKT3-2_190305_M00146_0024_000000000-D5N29_\
          GCGAGTAA_L001_001.annotated.bam",
    )?;
    bam.set_threads(8)?;

    let mut qc = BamQC::new();
    let mut record = Record::new();
    let mut record_error = Ok(());

    loop {
        match bam.read(&mut record) {
            Err(ReadError::NoMoreRecord) => break,
            Err(e) => {
                record_error = Err(Error::ReadError(e));
                break;
            }
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

    match record_error {
        Ok(_) => {
            let serialized = serde_json::to_string(&qc)?;
            println!("{}", serialized);
            Ok(())
        }
        Err(e) => Err(e),
    }
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
