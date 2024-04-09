use std::io::{Read, stdin};

use clap::Parser;
use itertools::Itertools;
use rayon::prelude::*;

use crate::events::Event;

mod enums;
mod traits;
mod prefixes;
mod common_components;
mod guid;
mod suffixes;
mod utils;
mod special;
mod advanced;
mod events;


fn parse_file<R: Read + Send>(reader: R) {
    let reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(reader);

    reader.into_records()
        .par_bridge()
        .filter_map(Result::ok)
        .map(|line| Event::parse(&line.iter().collect_vec()))
        .for_each(|e| {
            match e {
                Ok(x) => println!("{:?}", x),
                Err(x) => eprintln!("{}", x)
            }
        });
}


fn main() {
    parse_file(stdin());
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::PathBuf;
    use std::str::FromStr;

    use crate::parse_file;

    #[test]
    fn test1() {
        let wowlog_path = PathBuf::from_str(r"E:\Games\Blizzard\World of Warcraft\_retail_\Logs\WoWCombatLog-040624_135724.txt").unwrap();

        let file = File::open(wowlog_path)
            .expect("Error loading wowlogs file.");

        parse_file(file);
    }

    #[test]
    fn test2() {
        let wowlog_path = PathBuf::from_str("/test_data/WoWCombatLog-021524_201412.txt").unwrap();

        let file = File::open(wowlog_path)
            .expect("Error loading wowlogs file.");

        parse_file(file);
    }

    #[test]
    fn test3() {
        let file = "2/15 20:14:12.865  COMBAT_LOG_VERSION,20,ADVANCED_LOG_ENABLED,1,BUILD_VERSION,10.2.5,PROJECT_ID,1\n".as_bytes();

        parse_file(file);
    }
}

