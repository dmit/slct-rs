#![crate_type = "bin"]

#![feature(collections)]
#![feature(core)]
#![feature(env)]
#![feature(io)]
#![feature(os)]
#![feature(path)]

extern crate getopts;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied,Vacant};
use std::env;
use std::old_io::{BufferedReader, File};
use std::rt::heap;

use getopts::Options;

type Hash = u64;
type Count = u32;
type Words = HashMap<Hash, Count>;
type Clusters = HashMap<String, Count>;

fn main() {
    let mut opts = Options::new();
    opts.optopt("c", "cluster-threshold", "cluster appearance threshold for diplay", "CLUSTERTHRESHOLD");
    opts.optflag("m", "mem-stats", "print memory allocator statistics at the end");
    opts.optflag("r", "rare", "display only clusters below CLUSTERTHRESHOLD");
    opts.optopt("w", "word-threshold", "minimum frequency of a word to be considered for a cluster", "WORDTHRESHOLD");
    opts.optopt("", "max-line-length", "discard lines longer than this many characters", "MAX_LINE_LENGTH");

    let args: Vec<String> = env::args().map(|s| s.into_string().unwrap()).collect();
    let matches = opts.parse(args.tail()).unwrap_or_else(|f| panic!(f.to_string()));

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
    let print_mem_stats = matches.opt_present("m");

    let inputs: Vec<Path> = matches.free.iter().map(|f| Path::new(f)).collect();

    let word_freq = calc_word_freq(&inputs, max_line_length);
    println!("found {} unique words", word_freq.len());

    let clusters = calc_clusters(&inputs, &word_freq, word_threshold, max_line_length);
    println!("found {} clusters", clusters.len());

    if print_mem_stats {
        heap::stats_print();
    }

    for (k,v) in clusters.iter() {
        if show_rare == (*v < cluster_threshold) {
            println!("{}\t{}", v, k);
        }
    }
}

fn calc_word_freq(paths: &Vec<Path>, max_line_length: usize) -> Words {
    let mut word_freq: HashMap<Hash, Count> = HashMap::new();

    for path in paths.iter() {
        let file = File::open(path);
        let mut reader = BufferedReader::new(file);

        let mut i = 0;
        for l in reader.lines() {
            let line = l.ok().unwrap();
            if max_line_length > 0 && line.len() > max_line_length {
                continue;
            }

            for w in line.words() {
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

fn calc_clusters(paths: &Vec<Path>, word_freq: &Words, word_threshold: Count, max_line_length: usize) -> Clusters {
    let mut clusters: HashMap<String, Count> = HashMap::new();

    for path in paths.iter() {
        let mut file = BufferedReader::new(File::open(path));
        for l in file.lines() {
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
    let words: Vec<&str> = line.words().map({ |w|
        if word_freq[fnv1a(w)] < word_threshold {
            "*"
        } else {
            w
        }
    }).collect();

    words.connect(" ")
}

static FNV_PRIME_64: Hash = 1099511628211;
static FNV1_OFFSET_BASIS_64: Hash = 14695981039346656037;

fn fnv1a(s: &str) -> Hash {
    let mut hash = FNV1_OFFSET_BASIS_64;
    for b in s.as_bytes().iter() {
        hash = (hash ^ *b as Hash) * FNV_PRIME_64;
    }
    hash
}
