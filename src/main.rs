extern crate fnv;
extern crate getopts;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::env;
use std::fs::File;
use std::hash::{BuildHasherDefault, Hasher};
use std::io;
use std::io::{BufRead, BufReader};
use std::iter::Iterator;
use std::path::Path;

use fnv::FnvHasher;
use getopts::Options;

type Hash = u64;
type Count = u32;
type Fnv = BuildHasherDefault<FnvHasher>;
type Words = HashMap<Hash, Count, Fnv>;
type Clusters = HashMap<String, Count, Fnv>;

fn main() {
    let mut opts = Options::new();
    opts.optopt("c",
                "cluster-threshold",
                "cluster appearance threshold for diplay (1000)",
                "CLUSTER_THRESHOLD");
    opts.optflag("h", "help", "print this help message");
    opts.optflag("m",
                 "merge-lines",
                 "consider lines with leading whitespace part of the previous message");
    opts.optflag("r", "rare", "display only clusters below CLUSTER_THRESHOLD");
    opts.optopt("w",
                "word-threshold",
                "minimum frequency of a word to be considered for a cluster (1000)",
                "WORD_THRESHOLD");
    opts.optopt("",
                "max-line-length",
                "discard lines longer than this many characters (1000)",
                "MAX_LINE_LENGTH");

    let matches = opts.parse(env::args().skip(1)).expect("Parsing args");

    let cluster_threshold: Count = match matches.opt_str("c") {
        Some(ct) => ct.parse().expect("Invalid cluster-threshold"),
        None => 1000,
    };

    let word_threshold: Count = match matches.opt_str("w") {
        Some(wt) => wt.parse().expect("Invalid word-threshold"),
        None => 1000,
    };

    let max_line_length: usize = match matches.opt_str("max-line-length") {
        Some(len) => len.parse().expect("Invalid max-line-length"),
        None => 1000,
    };

    let print_help = matches.opt_present("h");
    let merge_lines = matches.opt_present("m");
    let show_rare = matches.opt_present("r");

    let inputs: Vec<&Path> = matches.free
                                    .iter()
                                    .map(|p| Path::new(p))
                                    .collect();

    if print_help || inputs.is_empty() {
        println!("{}", opts.usage("Usage: slct-rs [options] [<files>...]"));
        return;
    }

    let word_freq = calc_word_freq(&inputs, max_line_length)
                        .expect("Failed to calculate word frequency");
    println!("found {} unique words", word_freq.len());

    let clusters = calc_clusters(&inputs,
                                 &word_freq,
                                 word_threshold,
                                 max_line_length,
                                 merge_lines)
                       .expect("Failed to calculate clusters");
    println!("found {} clusters", clusters.len());

    for (k, v) in clusters.iter() {
        if show_rare == (*v < cluster_threshold) {
            println!("{}\t{}", v, k);
        }
    }
}

fn calc_word_freq(paths: &Vec<&Path>, max_line_length: usize) -> io::Result<Words> {
    let mut word_freq: Words = HashMap::default();

    for path in paths {
        let file = try!(File::open(path));
        let reader = BufReader::new(file);

        for l in reader.lines() {
            let line = try!(l);
            if max_line_length > 0 && line.len() > max_line_length {
                continue;
            }

            for w in line.split(char::is_whitespace).filter(|s| !s.is_empty()) {
                let hash = fnv1a(w);
                match word_freq.entry(hash) {
                    Vacant(wf) => {
                        wf.insert(1);
                    }
                    Occupied(mut wf) => {
                        *wf.get_mut() += 1;
                    }
                };
            }
        }
    }

    Result::Ok(word_freq)
}

fn calc_clusters(paths: &Vec<&Path>,
                 word_freq: &Words,
                 word_threshold: Count,
                 max_line_length: usize,
                 merge_lines: bool)
                 -> io::Result<Clusters> {
    let mut clusters: Clusters = HashMap::default();

    for path in paths {
        let file = try!(File::open(path));
        let reader = BufReader::new(file);

        let mut chunk = String::new();
        for l in reader.lines() {
            let line = try!(l);
            let whitespace = match line.chars().next() {
                Some(ch) => ch.is_whitespace(),
                None => true,
            };

            if merge_lines && whitespace {
                chunk.push_str(&line);
                continue;
            }

            if max_line_length > 0 && chunk.len() > max_line_length {
                chunk.clear();
                continue;
            }

            let cluster = clusterify(&chunk, word_freq, word_threshold);
            if !cluster.is_empty() {
                match clusters.entry(cluster) {
                    Vacant(c) => {
                        c.insert(1);
                    }
                    Occupied(mut c) => {
                        *c.get_mut() += 1;
                    }
                };
            }

            chunk.clear();
            chunk.push_str(&line);
        }

        let cluster = clusterify(&chunk, word_freq, word_threshold);
        if !cluster.is_empty() {
            match clusters.entry(cluster) {
                Vacant(c) => {
                    c.insert(1);
                }
                Occupied(mut c) => {
                    *c.get_mut() += 1;
                }
            };
        }
    }

    Result::Ok(clusters)
}

fn clusterify(line: &str, word_freq: &Words, word_threshold: Count) -> String {
    let words: Vec<&str> = line.split(char::is_whitespace)
                               .filter(|s| !s.is_empty())
                               .map({
                                   |w| {
                                       if word_freq[&fnv1a(w)] < word_threshold {
                                           "*"
                                       } else {
                                           w
                                       }
                                   }
                               })
                               .collect();

    words.join(" ")
}

#[inline]
fn fnv1a(s: &str) -> Hash {
    let mut hasher = FnvHasher::default();
    hasher.write(s.as_bytes());
    hasher.finish()
}
