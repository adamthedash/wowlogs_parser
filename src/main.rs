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


#[cfg(test)]
mod tests {
    use itertools::Itertools;

    #[test]
    fn test1() {
        let wowlog_path = r"E:\Games\Blizzard\World of Warcraft\_retail_\Logs\WoWCombatLog-040624_135724.txt";

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_path(wowlog_path)
            .expect("Error loading wowlogs file.");


        let mut seen = vec![
            "ZONE_CHANGE", "MAP_CHANGE", "COMBATANT_INFO", "ENCOUNTER_START", "COMBAT_LOG_VERSION",
            "SPELL_AURA_APPLIED", "SPELL_PERIODIC_HEAL", "SPELL_CAST_SUCCESS", "SPELL_AURA_REMOVED",
            "SPELL_AURA_REFRESH", "SPELL_CAST_START", "SPELL_HEAL", "SPELL_ENERGIZE", "SPELL_SUMMON",
            "SWING_DAMAGE", "SPELL_ABSORBED", "SPELL_MISSED", "SPELL_DAMAGE", "SWING_MISSED",
            "SPELL_PERIODIC_ENERGIZE", "SPELL_CAST_FAILED", "SPELL_PERIODIC_DAMAGE",
            "SPELL_EMPOWER_START", "SPELL_EMPOWER_END", "SPELL_PERIODIC_MISSED", "SPELL_DRAIN",
            "UNIT_DIED", "PARTY_KILL", "SPELL_EMPOWER_INTERRUPT", "UNIT_DESTROYED", "ENCOUNTER_END",
            "SPELL_AURA_BROKEN_SPELL", "SPELL_RESURRECT", "ENCHANT_REMOVED",
        ];

        for entry in reader.records()
            .filter_map(|e| match e {
                Ok(x) => { Some(x) }
                Err(_) => { None }
            })
            .filter(|e| !seen.iter().any(|&s| e[0].contains(s)))
            .take(100) {
            println!("{:?}", entry)
        }


        // for entry in reader.records() {
        //     let parsed = parse_line(&entry.expect("Error parsing entry."));
        //     if parsed.is_err() {
        //         println!("Error parsing entry, skipping. {:?}", parsed);
        //         continue;
        //     }
        //
        // }
    }
}