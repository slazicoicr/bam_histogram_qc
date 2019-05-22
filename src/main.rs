use rust_htslib::bam::{
    record::{Cigar, Record},
    ReadError, Reader,
};
use rust_htslib::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
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

fn main() {
    let mut bam = Reader::from_path(
        &"/home/slazic/Documents/Code/2019-05-21-bam_hist_rust/bam_histogram_rust/files/\
          SWID_13766177_GLCS_0001_Lv_R_PE_280_WG_DKT3-2_190305_M00146_0024_000000000-D5N29_\
          GCGAGTAA_L001_001.annotated.bam",
    )
    .unwrap();
    bam.set_threads(8).unwrap();

    let mut qc = BamQC::new();

    let mut record = Record::new();

    loop {
        match bam.read(&mut record) {
            Err(ReadError::NoMoreRecord) => break,
            Err(e) => panic!(e),
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

    //println!("{:?}", qc);
}
