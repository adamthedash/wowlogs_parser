use std::path::PathBuf;

use clap::Parser;
use itertools::Itertools;
use rayon::prelude::*;

use crate::events::Event;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    wowlog_path: PathBuf,
}


// fn main() {
//     let args = Args::parse();
//
//     let mut reader = csv::ReaderBuilder::new()
//         .has_headers(false)
//         .flexible(true)
//         .from_path(args.wowlog_path)
//         .expect("Error loading wowlogs file.");
//
//
//     let mut events = vec![];
//     for entry in reader.records() {
//         let parsed = parse_line(&entry.expect("Error parsing entry."));
//         if parsed.is_err() {
//             println!("Error parsing entry, skipping. {:?}", parsed);
//             continue;
//         }
//         // println!("{:?}\n", parsed);
//
//         events.push(parsed.unwrap());
//     }
//
//     println!("{}", events.len());
//
//     let actors = events.iter()
//         .filter_map(|e| match &e.event_type {
//             EventTypes::Other(OtherEvent { source: Some(a), .. }) => Some(&a.name),
//             _ => None
//         })
//         .collect::<HashSet<_>>();
//
//     println!("{:?}", actors);
//     let actors = events.iter()
//         .filter_map(|e| match &e.event_type {
//             EventTypes::Other(OtherEvent { target: Some(a), .. }) => Some(&a.name),
//             _ => None
//         })
//         .collect::<HashSet<_>>();
//
//     println!("{:?}", actors);
//
//     let adam = "Yildrisz-Ravencrest".to_string();
//     let adam_spells = events.iter()
//         .filter_map(|e| match &e.event_type {
//             EventTypes::Other(OtherEvent {
//                                   source: Some(Actor { name: s, .. }),
//                                   target: Some(Actor { name: t, .. }),
//                                   prefix: Prefix::SPELL(spell),
//                                   ..
//                               })
//             if s == &adam && t == &adam => { Some(spell) }
//             _ => None
//         })
//         .collect::<Vec<_>>();
//
//     println!("{:?}", adam_spells);
// }
//
//

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


fn parse_file(log_path: PathBuf) -> Vec<Event> {
    let reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(log_path)
        .expect("Error loading wowlogs file.");


    let events = reader.into_records()
        .par_bridge()
        .filter_map(Result::ok)
        .map(|line| Event::parse(&line.iter().collect_vec()))
        .filter_map(|e| {
            match e {
                Ok(x) => Some(x),
                Err(x) => {
                    eprintln!("{}", x);
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    events
}


fn main() {
    let args = Args::parse();

    let events = parse_file(args.wowlog_path);
    println!("Events parsed: {}", events.len())
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::str::FromStr;

    use crate::parse_file;

    #[test]
    fn test1() {
        let wowlog_path = PathBuf::from_str(r"E:\Games\Blizzard\World of Warcraft\_retail_\Logs\WoWCombatLog-040624_135724.txt").unwrap();

        let events = parse_file(wowlog_path);
        println!("num events: {}", events.len())
    }

    #[test]
    fn test2() {
        let wowlog_path = PathBuf::from_str("/test_data/WoWCombatLog-021524_201412.txt").unwrap();

        let events = parse_file(wowlog_path);
        println!("num events: {}", events.len())
    }
}

