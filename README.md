slct-rs
=======

[![Build Status](https://github.com/dmit/slct-rs/workflows/Test/badge.svg)](https://github.com/dmit/slct-rs/actions)

SLCT (simple logfile clustering tool) in Rust. Based on ideas
by [Risto Vaarandi](https://github.com/ristov).

## Usage
```
slct-rs [options] [<files>...]

Options:
    -c, --cluster-threshold CLUSTER_THRESHOLD
                        cluster appearance threshold for diplay (1000)
    -h, --help          print help message
    -r, --rare          display only clusters below CLUSTER_THRESHOLD
    -w, --word-threshold WORD_THRESHOLD
                        minimum frequency of a word to be considered for a
                        cluster (1000)
        --max-line-length MAX_LINE_LENGTH
                        discard lines longer than this many characters (1000)
```

## License
The [Unlicense](LICENSE).
