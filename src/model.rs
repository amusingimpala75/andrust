use crate::args::TokenType;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) degree: u8,
    pub(crate) token_type: TokenType,
    pub(crate) freq_table: HashMap<String, Vec<(String, f64)>>,
}

impl Model {
    pub(crate) fn write(&self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let mut w = BufWriter::new(File::create(path)?);

        write_u64(&mut w, self.degree as u64)?;
        write_string(&mut w, self.token_type.as_str())?;

        write_u64(&mut w, self.freq_table.len() as u64)?;
        for (prefix, table) in &self.freq_table {
            write_string(&mut w, prefix)?;
            write_u64(&mut w, table.len() as u64)?;
            for (next, threshold) in table {
                write_string(&mut w, next)?;
                write_f64(&mut w, *threshold)?;
            }
        }

        Ok(())
    }

    pub(crate) fn parse(path: PathBuf) -> Result<Model, Box<dyn Error>> {
        let mut r = BufReader::new(File::create(path)?);

        let degree = read_u64(&mut r)? as u8;
        let token_type: TokenType = TokenType::parse(&read_string(&mut r)?)
            .map_err(|_| std::io::Error::other("invalid token type"))?;

        let mut freq_table: HashMap<String, Vec<(String, f64)>> = HashMap::new();
        for _ in 0..read_u64(&mut r)? {
            let prefix = read_string(&mut r)?;
            let mut v: Vec<(String, f64)> = vec![];
            for _ in 0..read_u64(&mut r)? {
                v.push((read_string(&mut r)?, read_f64(&mut r)?));
            }
            freq_table.insert(prefix, v);
        }

        Ok(Model {
            degree,
            token_type,
            freq_table,
        })
    }

    pub(crate) fn generate(&self, max_generated: u64) -> Result<String, Box<dyn Error>> {
        let mut generated: String = "^".repeat(self.degree.into());
        let mut count_generated: u64 = 0;

        while !generated.ends_with("$") && count_generated < max_generated {
            let prob = random_f64()?;
            let thresholds = &self.freq_table[&generated
                .chars()
                .rev()
                .take(self.degree.into())
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>()];

            for (s, threshold) in thresholds {
                if *threshold <= prob {
                    generated.push_str(s);
                    break;
                }
            }

            count_generated += 1;
        }

        if !generated.ends_with("$") {
            generated.push_str("$");
        }

        Ok(generated[self.degree.into()..generated.len() - 1].to_string())
    }

    pub(crate) fn train(&mut self, files: Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
        // Map of prefix -> list of following and their frequencies
        let mut frequency_table: HashMap<String, HashMap<String, u64>> = HashMap::new();
        for file in files {
            for line in BufReader::new(File::open(file)?).lines() {
                // char token type only for now
                assert_eq!(self.token_type, TokenType::Char);
                for window in "^"
                    .repeat(self.degree.into())
                    .chars()
                    .map(|c| c.to_string())
                    .chain(line?.chars().map(|c| c.to_string()))
                    .chain(vec!["$".to_string()])
                    .collect::<Vec<_>>() // [TODO] avoid b/c runtime cost
                    .windows(usize::from(self.degree) + 1)
                {
                    let prefix = &window[..usize::from(self.degree)];
                    let next = &window[usize::from(self.degree)];
                    frequency_table
                        .entry(prefix.join(""))
                        .or_insert(HashMap::new())
                        .entry(next.to_string())
                        .and_modify(|i| *i += 1)
                        .or_insert(1);
                }
            }
        }

        // Convert frequency table into thresholds
        self.freq_table = HashMap::new();
        for (k, v) in frequency_table {
            let mut suffices: Vec<String> = vec![];
            let mut sums: Vec<u64> = vec![];
            let mut sum: u64 = 0;

            for (k, v) in v {
                suffices.push(k);
                sums.push(sum);
                sum += v;
            }

            let mut with_thresholds: Vec<(String, f64)> = vec![];
            for (s, c) in suffices.into_iter().zip(sums) {
                with_thresholds.push((s, c as f64 / sum as f64))
            }

            self.freq_table.insert(k, with_thresholds);
        }

        Ok(())
    }
}

fn write_u64(w: &mut impl Write, n: u64) -> std::io::Result<()> {
    w.write_all(&n.to_le_bytes())
}

fn write_string(w: &mut impl Write, s: &str) -> std::io::Result<()> {
    write_u64(w, s.len() as u64)?;
    w.write_all(s.as_bytes())
}

fn write_f64(w: &mut impl Write, f: f64) -> std::io::Result<()> {
    w.write_all(&f.to_le_bytes())
}

fn read_u64(r: &mut impl Read) -> std::io::Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_string(r: &mut impl Read) -> Result<String, Box<dyn Error>> {
    let length = read_u64(r)?;
    let mut buf = vec![0u8; length as usize];
    r.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

fn read_f64(r: &mut impl Read) -> std::io::Result<f64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

fn random_f64() -> std::io::Result<f64> {
    let mut bytes = [0u8; 8];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let n = u64::from_ne_bytes(bytes);
    Ok((n >> 11) as f64 / (1u64 << 53) as f64)
}
