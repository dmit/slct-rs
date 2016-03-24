The code in this repository was written by Dmitri Melnikov
and released to the public domain, as explained at
http://creativecommons.org/publicdomain/zero/1.0/

slct-rs
=======

[![Build Status](https://travis-ci.org/dmit/slct-rs.svg?branch=master)](https://travis-ci.org/dmit/slct-rs)

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
