extern crate getopts;

use std::collections::HashMap;
use std::io::{BufferedReader, File};
use std::os;
use std::rt::heap;

use getopts::{getopts,optflag,optopt};

type Hash = uint;
type Count = u32;
type Words = HashMap<Hash, Count>;
type Clusters = HashMap<String, Count>;

fn main() {
    let opts = [
        optopt("c", "cluster-threshold", "cluster appearance threshold for diplay", "CLUSTERTHRESHOLD"),
        optflag("m", "mem-stats", "print memory allocator statistics at the end"),
        optflag("r", "rare", "display only clusters below CLUSTERTHRESHOLD"),
        optopt("w", "word-threshold", "minimum frequency of a word to be considered for a cluster", "WORDTHRESHOLD"),
    ];
    let matches = getopts(os::args().tail(), opts).unwrap_or_else(|f| fail!(f.to_string()));

    let cluster_threshold: Count = match matches.opt_str("c") {
        Some(ct) => { from_str(ct.as_slice()).unwrap() }
        None => { 1000 }
    };

    let word_threshold: Count = match matches.opt_str("w") {
        Some(wt) => { from_str(wt.as_slice()).unwrap() }
        None => { 1000 }
    };

    let show_rare = matches.opt_present("r");
    let print_mem_stats = matches.opt_present("m");

    let inputs: Vec<Path> = matches.free.iter().map(|f| Path::new(f.as_slice())).collect();

    let word_freq = calc_word_freq(&inputs);
    println!("found {} unique words", word_freq.len());

    let clusters = calc_clusters(&inputs, &word_freq, word_threshold);
    println!("found {} clusters", clusters.len());

    if print_mem_stats {
        heap::stats_print();
    }

    for (k,v) in clusters.iter() {
        if show_rare == *v < cluster_threshold {
            println!("{}\t{}", v, k);
        }
    }
}

fn calc_word_freq(paths: &Vec<Path>) -> Words {
    let mut word_freq: HashMap<Hash, Count> = HashMap::new();

    for path in paths.iter() {
        let mut file = BufferedReader::new(File::open(path));
        for l in file.lines() {
            for w in l.ok().unwrap().as_slice().words() {
                let hash = fnv1a(w);
                word_freq.insert_or_update_with(hash, 1, |_k,v| *v+=1);
            }
        }
    }

    word_freq
}

fn calc_clusters(paths: &Vec<Path>, word_freq: &Words, word_threshold: Count) -> Clusters {
    let mut clusters: HashMap<String, Count> = HashMap::new();

    for path in paths.iter() {
        let mut file = BufferedReader::new(File::open(path));
        for l in file.lines() {
            let cluster = clusterify(l.unwrap().as_slice(), word_freq, word_threshold);
            if !cluster.is_empty() {
                clusters.insert_or_update_with(cluster, 1, |_k,v| *v+=1);
            }
        }
    }

    clusters
}

fn clusterify(line: &str, word_freq: &Words, word_threshold: Count) -> String {
    let words: Vec<&str> = line.words().map({|w|
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
