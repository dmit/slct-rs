#![crate_type = "bin"]

extern crate getopts;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::env;
use std::fs::File;
use std::io::{BufRead,BufReader};
use std::path::Path;

use getopts::Options;

type Hash = u64;
type Count = u32;
type Words = HashMap<Hash, Count>;
type Clusters = HashMap<String, Count>;

fn main() {
    let mut opts = Options::new();
    opts.optopt("c", "cluster-threshold", "cluster appearance threshold for diplay", "CLUSTERTHRESHOLD");
    opts.optflag("r", "rare", "display only clusters below CLUSTERTHRESHOLD");
    opts.optopt("w", "word-threshold", "minimum frequency of a word to be considered for a cluster", "WORDTHRESHOLD");
    opts.optopt("", "max-line-length", "discard lines longer than this many characters", "MAX_LINE_LENGTH");

    let matches = opts.parse(env::args().skip(1)).unwrap_or_else(|f| panic!(f.to_string()));

    let cluster_threshold: Count = match matches.opt_str("c") {
        Some(ct) => { ct.parse().unwrap() }
        None => { 1000 }
    };

    let word_threshold: Count = match matches.opt_str("w") {
        Some(wt) => { wt.parse().unwrap() }
        None => { 1000 }
    };

    let max_line_length: usize = match matches.opt_str("max-line-length") {
        Some(len) => len.parse().unwrap(),
        None => 1000,
    };

    let show_rare = matches.opt_present("r");

    let inputs: Vec<&Path> = matches.free.iter().map(|f| Path::new(f)).collect();

    let word_freq = calc_word_freq(&inputs, max_line_length);
    println!("found {} unique words", word_freq.len());

    let clusters = calc_clusters(&inputs, &word_freq, word_threshold, max_line_length);
    println!("found {} clusters", clusters.len());

    for (k,v) in clusters.iter() {
        if show_rare == (*v < cluster_threshold) {
            println!("{}\t{}", v, k);
        }
    }
}

fn calc_word_freq(paths: &Vec<&Path>, max_line_length: usize) -> Words {
    let mut word_freq: HashMap<Hash, Count> = HashMap::new();

    for path in paths.iter() {
        let file = File::open(path);
        let reader = BufReader::new(file.unwrap());

        for l in reader.lines() {
            let line = l.ok().unwrap();
            if max_line_length > 0 && line.len() > max_line_length {
                continue;
            }

            for w in line.split(char::is_whitespace).filter(|s| !s.is_empty()) {
                let hash = fnv1a(w);
                match word_freq.entry(hash) {
                    Vacant(wf) => { wf.insert(1); },
                    Occupied(mut wf) => { *wf.get_mut() += 1; },
                };
            }
        }
    }

    word_freq
}

fn calc_clusters(paths: &Vec<&Path>, word_freq: &Words, word_threshold: Count, max_line_length: usize) -> Clusters {
    let mut clusters: HashMap<String, Count> = HashMap::new();

    for path in paths.iter() {
        let file = File::open(path);
        let reader = BufReader::new(file.unwrap());

        for l in reader.lines() {
            let line = l.ok().unwrap();
            if max_line_length > 0 && line.len() > max_line_length {
                continue;
            }

            let cluster = clusterify(line, word_freq, word_threshold);
            if !cluster.is_empty() {
                match clusters.entry(cluster) {
                    Vacant(c) => { c.insert(1); },
                    Occupied(mut c) => { *c.get_mut() += 1; },
                };
            }
        }
    }

    clusters
}

fn clusterify(line: String, word_freq: &Words, word_threshold: Count) -> String {
    let words: Vec<&str> = line.split(char::is_whitespace).filter(|s| !s.is_empty()).map({ |w|
        if word_freq[&fnv1a(w)] < word_threshold {
            "*"
        } else {
            w
        }
    }).collect();

    words.join(" ")
}

const FNV_PRIME_64: Hash = 1099511628211;
const FNV1_OFFSET_BASIS_64: Hash = 14695981039346656037;

fn fnv1a(s: &str) -> Hash {
    let mut hash = FNV1_OFFSET_BASIS_64;
    for b in s.as_bytes().iter() {
        hash = (hash ^ *b as Hash) * FNV_PRIME_64;
    }
    hash
}
