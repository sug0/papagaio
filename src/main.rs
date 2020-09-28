use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};

use permutation::permutation;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug)]
struct Stats {
    of: HashMap<char, Stat>,
}

#[derive(Clone, Debug)]
struct Stat {
    next: HashMap<char, i32>,
}

#[derive(Clone, Debug)]
struct Usage<'a> {
    current: char,
    usage: &'a HashMap<char, Vec<char>>,
}

fn main() {
    let stats = read_stats()
        .expect("failed to read stats");

    // determine highest usage for each entry
    let usage = determine_highest_usage(&stats);

    // make up some random gibberish
    let sentence: String = Usage::new('x', &usage)
        .take(100)
        .collect();

    println!("{}", sentence);
}

fn determine_highest_usage(stats: &Stats) -> HashMap<char, Vec<char>> {
    let mut usage = HashMap::new();
    for (ch, neighbors) in stats.of.iter() {
        let mut numbers = Vec::new();
        let mut chars = Vec::new();
        for (neigh, number) in neighbors.next.iter() {
            numbers.push(*number);
            chars.push(*neigh);
        }
        let perm = permutation::sort(numbers);
        let ordered_chars = perm.apply_slice(chars);
        usage.insert(*ch, ordered_chars);
    }
    usage
}

fn read_stats() -> io::Result<Stats> {
    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    let reader = BufReader::new(stdin_lock);

    let mut stats = Stats::new();

    for line in reader.lines() {
        let transformed: String = line?
            .chars()
            .map(normalize)
            .collect();

        let ch_fst = transformed.chars();
        let ch_snd = transformed.chars();

        let zipped = ch_fst.zip(ch_snd.cycle().skip(1));

        for (fst, snd) in zipped {
            stats.update(fst, snd);
        }
    }

    Ok(stats)
}

fn normalize(ch: char) -> char {
    let mut buf = [0_u8; 4];
    let encoded = ch.encode_utf8(&mut buf[..]);
    encoded
        .nfkd()
        .flat_map(|ch| ch.to_lowercase())
        .nth(0)
        .unwrap()
}

impl Stat {
    fn new() -> Self {
        Stat { next: HashMap::new() }
    }
}

impl Stats {
    fn new() -> Self {
        Stats { of: HashMap::new() }
    }

    fn update(&mut self, ch: char, neigh: char) {
        let stat = self.of.entry(ch).or_insert_with(Stat::new);
        let number = stat.next.entry(neigh).or_insert(0);
        *number += 1;
    }
}

impl<'a> Usage<'a> {
    fn new(first: char, usage: &'a HashMap<char, Vec<char>>) -> Self {
        Usage { current: normalize(first), usage }
    }
}

impl Iterator for Usage<'_> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let percent: f32 = loop {
            let x = rand::random();
            if x >= 0.9 {
                break x;
            }
        };
        let candidates = self.usage.get(&self.current)?;
        let char_picked = (percent * (candidates.len() as f32)) as usize;
        let char_picked = candidates[char_picked];
        self.current = char_picked;
        Some(char_picked)
    }
}
