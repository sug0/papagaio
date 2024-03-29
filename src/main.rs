use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write, BufWriter};

use permutation::permutation;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug)]
struct Stats {
    of: HashMap<String, Stat>,
}

#[derive(Clone, Debug)]
struct Stat {
    next: HashMap<String, i32>,
}

#[derive(Clone, Debug)]
struct Usage<'a> {
    threshold: f32,
    current: String,
    usage: &'a HashMap<String, Vec<String>>,
}

struct Flags {
    thres: f32,
    words: usize,
}

enum Arguments {
    None,
    Print,
    Values(Flags),
}

enum ArgumentKind {
    Flag,
    Words,
    Threshold,
}

fn main() {
    // fetch arguments
    let mut args = std::env::args();
    let prog_name = args.next().unwrap();

    // parse arguments
    let args = match parse_arguments(args) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("error: {}", e);
            usage(&prog_name);
            return;
        },
    };

    // determine highest usage for each entry
    let stats = read_stats()
        .expect("failed to read stats");
    let usage = determine_highest_usage(&stats);

    // handle args...
    let (thres, words) = match args {
        Arguments::None => (0.75, 100),
        Arguments::Values(Flags { thres, words }) => (thres, words),
        Arguments::Print => {
            println!("{:#?}", usage);
            return;
        },
    };

    // make up some random gibberish
    let sentence = Usage::new(thres, &usage)
        .take(words);

    write_sentence(sentence)
        .expect("failed to write sentence")
}

fn usage(prog_name: &str) {
    println!("usage: {} [-p <print-words-graph>] [-t <threshold>] [-w <words>]", prog_name);
}

fn parse_arguments(it: impl Iterator<Item = String>) -> Result<Arguments, Box<dyn std::error::Error>> {
    let mut kind = ArgumentKind::Flag;
    let mut args = None;
    for arg in it {
        match kind {
            ArgumentKind::Flag => match arg.as_ref() {
                "-p" => return Ok(Arguments::Print),
                "-t" => kind = ArgumentKind::Threshold,
                "-w" => kind = ArgumentKind::Words,
                _ => return Err(format!("invalid flag: {}", arg).into())
            },
            ArgumentKind::Threshold => {
                match args {
                    None => args = Some(Flags { thres: arg.parse()?, words: 100 }),
                    Some(ref mut f) => f.thres = arg.parse()?,
                };
                kind = ArgumentKind::Flag;
            },
            ArgumentKind::Words => {
                match args {
                    None => args = Some(Flags { thres: 0.75, words: arg.parse()? }),
                    Some(ref mut f) => f.words = arg.parse()?,
                };
                kind = ArgumentKind::Flag;
            },
        }
    }
    Ok(match args {
        None => Arguments::None,
        Some(flags) => Arguments::Values(flags),
    })
}

fn write_sentence<I>(sentence: I) -> io::Result<()>
where
    I: Iterator,
    <I as Iterator>::Item: AsRef<[u8]>,
{
    let stdout = io::stdout();
    let stdout_lock = stdout.lock();
    let mut writer = BufWriter::new(stdout_lock);

    for word in sentence {
        writer.write(word.as_ref())?;
        writer.write(b" ")?;
    }

    writer.write(b"\n")?;
    writer.flush()
}

fn determine_highest_usage(stats: &Stats) -> HashMap<String, Vec<String>> {
    let mut usage = HashMap::new();
    for (word, neighbors) in stats.of.iter() {
        let mut numbers = Vec::new();
        let mut words = Vec::new();
        for (neigh, number) in neighbors.next.iter() {
            numbers.push(*number);
            words.push(neigh.clone());
        }
        let perm = permutation::sort(numbers);
        let ordered_words = perm.apply_slice(words);
        usage.insert(word.clone(), ordered_words);
    }
    usage
}

fn read_stats() -> io::Result<Stats> {
    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    let reader = BufReader::new(stdin_lock);

    let mut stats = Stats::new();

    for line in reader.lines() {
        let line = line?;

        let w_fst = line.split_whitespace();
        let w_snd = line.split_whitespace().cycle().skip(1);

        for (fst, snd) in w_fst.zip(w_snd) {
            let fst: String = fst
                .chars()
                .map(normalize)
                .collect();
            let snd: String = snd
                .chars()
                .map(normalize)
                .collect();
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

    fn update(&mut self, word: String, neigh: String) {
        let stat = self.of.entry(word).or_insert_with(Stat::new);
        let number = stat.next.entry(neigh).or_insert(0);
        *number += 1;
    }
}

impl<'a> Usage<'a> {
    fn new(threshold: f32, usage: &'a HashMap<String, Vec<String>>) -> Self {
        let threshold = if threshold < 0.0 || threshold > 1.0 {
            0.75
        } else {
            threshold
        };
        let mut keys = usage.keys();
        let mut rounds = (threshold * 10.0) as i32;
        let mut first = String::from("A");
        loop {
            match keys.next() {
                Some(_) if rounds == 0 => break,
                Some(k) => {
                    first.clear();
                    first.push_str(k);
                },
                None => break,
            }
            rounds -= 1;
        }
        Usage {
            usage,
            threshold,
            current: first
                .chars()
                .map(normalize)
                .collect(),
        }
    }
}

impl Iterator for Usage<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut it_word = 0;
        loop {
            let mut it_percent = 0;
            let percent: f32 = loop {
                let x = rand::random();
                if x >= self.threshold || it_percent >= 30 {
                    break x;
                }
                it_percent += 1;
            };
            let candidates = self.usage.get(&self.current)?;
            let char_picked = (percent * (candidates.len() as f32)) as usize;
            let char_picked = &candidates[char_picked];
            if char_picked == &self.current || it_word < 30 {
                it_word += 1;
                continue;
            }
            self.current.clear();
            self.current.push_str(&char_picked);
            break Some(char_picked.clone());
        }
    }
}
