use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{BufReader, Lines};

use crate::errors::*;
use crate::parse;

pub struct Iter<R> {
    lines: Lines<BufReader<R>>,
    substitution_data: HashMap<String, Option<String>>,
}

impl<R: Read> Iter<R> {
    pub fn new(reader: R) -> Iter<R> {
        Iter {
            lines: BufReader::new(reader).lines(),
            substitution_data: HashMap::new(),
        }
    }
}

impl<R: Read> Iterator for Iter<R> {
    type Item = Result<(String, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = match self.lines.next() {
                Some(Ok(line)) => line,
                Some(Err(err)) => return Some(Err(Error::Io(err))),
                None => return None,
            };

            match parse::parse_line(&line, &mut self.substitution_data) {
                Ok(Some(result)) => return Some(Ok(result)),
                Ok(None) => {}
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
