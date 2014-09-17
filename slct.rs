use std::collections::HashMap;
use std::io::{BufferedReader, File};
use std::os;

static WORD_THRESHOLD: u32 = 1000;
static CLUSTER_THRESHOLD: u32 = 1000;

fn main() {
    let args = os::args();
    let inputs: Vec<Path> = args.tail().iter().map(|f| Path::new(f.as_slice())).collect();

    let mut word_freq: HashMap<u64, u32> = HashMap::new();
    for path in inputs.iter() {
        let mut file = BufferedReader::new(File::open(path));
        for l in file.lines() {
            for w in l.ok().unwrap().as_slice().words() {
                let hash = fnv1a(w);
                word_freq.insert_or_update_with(hash, 1, |_k,v| *v+=1);
            }
        }
    }

    println!("words found: {}", word_freq.len());

    let mut clusters: HashMap<String, u32> = HashMap::new();
    for path in inputs.iter() {
        let mut file = BufferedReader::new(File::open(path));
        for l in file.lines() {
            let cluster = clusterify(&word_freq, l.ok().unwrap().as_slice());
            if !cluster.is_empty() {
                clusters.insert_or_update_with(cluster, 1, |_k,v| *v+=1);
            }
        }
    }

    println!("clusters found: {}", clusters.len());

    for (k,v) in clusters.iter() {
        if *v >= CLUSTER_THRESHOLD {
            println!("{}\t{}", v, k);
        }
    }
}

fn clusterify(word_freq: &HashMap<u64, u32>, line: &str) -> String {
    let words: Vec<&str> = line.words().map({|w|
        if word_freq[fnv1a(w)] < WORD_THRESHOLD {
            "*"
        } else {
            w
        }
    }).collect();

    words.connect(" ")
}

static FNV_PRIME_64: u64 = 1099511628211;
static FNV1_OFFSET_BASIS_64: u64 = 14695981039346656037;

fn fnv1a(s: &str) -> u64 {
    let mut hash = FNV1_OFFSET_BASIS_64;
    for b in s.as_bytes().iter() {
        hash = (hash ^ *b as u64) * FNV_PRIME_64;
    }
    hash
}
