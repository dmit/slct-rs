slct-rs
=======

[![Build Status](https://github.com/dmit/slct-rs/workflows/Test/badge.svg)](https://github.com/dmit/slct-rs/actions)

SLCT (simple logfile clustering tool) in Rust. Based on ideas
by [Risto Vaarandi](https://github.com/ristov).

## Usage
```
slct [<input_files...>] [-c <cluster-threshold>] [-w <word-threshold>] [-l <max-line-length>] [-r] [-m]

Options:
  -c, --cluster-threshold
                    display only clusters with at least this many instances
  -w, --word-threshold
                    only consider words with at least this many appearances for
                    clustering
  -l, --max-line-length
                    discard lines longer than this many bytes
  -r, --show-rare   display the clusters below the instance threshold rather
                    than the common ones above it
  -m, --merge-lines consider lines with leading whitespace as continuation of
                    the previous line for clustering purposes
  --help            display usage information
```

## Build
```sh
cargo build --release
```
or
```sh
cargo install --path .
```
to automatically place the binary in `$PATH`.

## License
The [Unlicense](LICENSE).
