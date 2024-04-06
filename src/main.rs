use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;

use crate::parser::{EventTypes, OtherEvent, parse_line};
use crate::player_events::Actor;
use crate::prefixes::Prefix;

mod parser;
mod special;
mod traits;
mod player_events;
mod suffixes;
mod prefixes;
mod guid;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "FILE")]
    wowlog_path: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    out_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(args.wowlog_path)
        .expect("Error loading wowlogs file.");


    let mut events = vec![];
    for entry in reader.records() {
        let parsed = parse_line(&entry.expect("Error parsing entry."));
        if parsed.is_err() {
            println!("Error parsing entry, skipping. {:?}", parsed);
            continue;
        }
        // println!("{:?}\n", parsed);

        events.push(parsed.unwrap());
    }

    println!("{}", events.len());

    let actors = events.iter()
        .filter_map(|e| match &e.event_type {
            EventTypes::Other(OtherEvent { source: Some(a), .. }) => Some(&a.name),
            _ => None
        })
        .collect::<HashSet<_>>();

    println!("{:?}", actors);
    let actors = events.iter()
        .filter_map(|e| match &e.event_type {
            EventTypes::Other(OtherEvent { target: Some(a), .. }) => Some(&a.name),
            _ => None
        })
        .collect::<HashSet<_>>();

    println!("{:?}", actors);

    let adam = "Yildrisz-Ravencrest".to_string();
    let adam_spells = events.iter()
        .filter_map(|e| match &e.event_type {
            EventTypes::Other(OtherEvent {
                                  source: Some(Actor { name: s, .. }),
                                  target: Some(Actor { name: t, .. }),
                                  prefix: Prefix::SPELL(spell),
                                  ..
                              })
            if s == &adam && t == &adam => { Some(spell) }
            _ => None
        })
        .collect::<Vec<_>>();

    println!("{:?}", adam_spells);
}
