use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::hash::{BuildHasherDefault, Hasher};
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;

use fnv::FnvHasher;
use structopt::StructOpt;

type Hash = u64;
type Count = u64;
type Fnv = BuildHasherDefault<FnvHasher>;
type WordCount = HashMap<Hash, Count, Fnv>;
type Clusters = HashMap<Vec<u8>, Count, Fnv>;

#[derive(StructOpt)]
#[structopt(author = "")]
struct Opts {
    /// Display only clusters with at least this many instances
    #[structopt(short = "c", long = "cluster-threshold", default_value = "1000")]
    cluster_threshold: Count,

    /// Only consider words with at least this many appearances for clustering
    #[structopt(short = "w", long = "word-threshold", default_value = "1000")]
    word_threshold: Count,

    /// Discard lines longer than this many bytes
    #[structopt(short = "l", long = "max-line-length", default_value = "1000")]
    max_line_length: usize,

    /// Display the clusters below the instance threshold rather than the common
    /// ones above it
    #[structopt(short = "r", long = "show-rare")]
    show_rare: bool,

    /// Consider lines with leading whitespace as continuation of the previous
    /// line for clustering purposes
    #[structopt(short = "m", long = "merge-lines")]
    merge_lines: bool,

    #[structopt()]
    input_files: Vec<PathBuf>,
}

fn main() -> Result<(), Box<Error>> {
    let opts = Opts::from_args();

    let word_freq = calc_word_freq(&opts.input_files, opts.max_line_length)?;
    println!("Found {} unique words", word_freq.len());

    let clusters = calc_clusters(
        &opts.input_files,
        &word_freq,
        opts.word_threshold,
        opts.max_line_length,
        opts.merge_lines,
    )?;
    println!("Found {} clusters", clusters.len());

    let sorted = {
        let mut v = clusters
            .into_iter()
            .filter(|&(_, count)| {
                if opts.show_rare {
                    count <= opts.cluster_threshold
                } else {
                    count >= opts.cluster_threshold
                }
            })
            .collect::<Vec<_>>();
        v.sort_by(|a, b| b.1.cmp(&(a.1)));
        v
    };

    for (cluster, count) in sorted {
        println!("{}\t{}", count, String::from_utf8(cluster)?,);
    }

    Ok(())
}

fn calc_word_freq(paths: &[PathBuf], max_line_length: usize) -> io::Result<WordCount> {
    let mut word_freq: WordCount = HashMap::default();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if max_line_length > 0 && line.len() > max_line_length {
                continue;
            }

            for w in line.split(char::is_whitespace).filter(|s| !s.is_empty()) {
                let hash = fnv1a(w.as_bytes());
                *word_freq.entry(hash).or_insert(0) += 1;
            }
        }
    }

    Ok(word_freq)
}

fn calc_clusters(
    paths: &[PathBuf],
    word_freq: &WordCount,
    word_threshold: Count,
    max_line_length: usize,
    merge_lines: bool,
) -> io::Result<Clusters> {
    let mut clusters: Clusters = HashMap::default();

    for path in paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut chunk: Vec<u8> = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let whitespace = match line.chars().next() {
                Some(ch) => ch.is_whitespace(),
                None => true,
            };

            if merge_lines && whitespace {
                chunk.extend_from_slice(line.as_bytes());
                continue;
            }

            if max_line_length > 0 && chunk.len() > max_line_length {
                chunk.clear();
                continue;
            }

            let cluster = clusterify(&chunk, word_freq, word_threshold);
            if !cluster.is_empty() {
                *clusters.entry(cluster).or_insert(0) += 1;
            }

            chunk.clear();
            chunk.extend_from_slice(line.as_bytes());
        }

        let cluster = clusterify(&chunk, word_freq, word_threshold);
        if !cluster.is_empty() {
            *clusters.entry(cluster).or_insert(0) += 1;
        }
    }

    Ok(clusters)
}

fn clusterify(chunk: &[u8], word_freq: &WordCount, word_threshold: Count) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut marker = get_whitespace(chunk).len();
    loop {
        let word = get_word(&chunk[marker..]);
        if word.is_empty() {
            break;
        }
        marker += word.len();

        if word_freq[&fnv1a(word)] < word_threshold {
            result.push(b'*');
        } else {
            result.extend_from_slice(word);
        }

        let whitespace = get_whitespace(&chunk[marker..]);
        result.extend_from_slice(whitespace);
        marker += whitespace.len();
    }

    result
}

#[inline]
fn get_word(bytes: &[u8]) -> &[u8] {
    for (i, b) in bytes.iter().enumerate() {
        if b.is_ascii_whitespace() {
            return &bytes[..i];
        }
    }
    bytes
}

#[inline]
fn get_whitespace(bytes: &[u8]) -> &[u8] {
    for (i, b) in bytes.iter().enumerate() {
        if !b.is_ascii_whitespace() {
            return &bytes[..i];
        }
    }
    bytes
}

#[inline]
fn fnv1a(bytes: &[u8]) -> Hash {
    let mut hasher = FnvHasher::default();
    hasher.write(bytes);
    hasher.finish()
}
