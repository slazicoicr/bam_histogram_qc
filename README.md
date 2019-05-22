# BAM Histogram QC

Takes a BAM file as input and returns a JSON file that summarizes:

* Template lengths
* CIGAR string per cycle

```
Bam Histogram Calculator 0.1
Savo Lazic <savo.lazic@oicr.on.ca
Collects various statistics from a BAM file that allow creation of histograms for QC purposes

USAGE:
    bam_histogram_qc [OPTIONS] <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -t, --threads <threads>    Number of threads to use [default: 1]

ARGS:
    <FILE>    File path to BAM file. Use '-' for STDIN
```

