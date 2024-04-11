use std::io::Read;

use anyhow::Result;
use itertools::Itertools;

use crate::components::events::Event;

pub struct EventParser<R> {
    reader: csv::Reader<R>,
}

impl<R: Read> EventParser<R> {
    pub fn new(reader: R) -> Self {
        let mut binding = csv::ReaderBuilder::new();
        let reader = binding
            .has_headers(false)
            .flexible(true)
            .from_reader(reader);


        Self { reader }
    }
}

impl<R: Read> Iterator for EventParser<R> {
    type Item = Result<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.reader
            .records()
            .filter_map(Result::ok)
            .map(|line| Event::parse(&line.iter().collect_vec()))
            .next();

        val
    }
}