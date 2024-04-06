use chrono::prelude::*;
use csv::StringRecord;
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::player_events::{Actor, AdvancedParams};
use crate::prefixes::{EventPrefix, Prefix};
use crate::special::{CombatLogVersion, MapChange, WorldMarkerPlaced, ZoneChange};
use crate::suffixes::{EventSuffix, Suffix};
use crate::traits::FromRecord;

#[derive(Debug)]
pub struct Event {
    pub datetime: NaiveDateTime,
    pub event_type: EventTypes,
}

#[derive(Debug)]
pub struct OtherEvent {
    pub prefix: Prefix,
    pub suffix: Option<Suffix>,
    pub source: Option<Actor>,
    pub target: Option<Actor>,
    pub advanced: Option<AdvancedParams>,
}

#[derive(Debug)]
pub enum EventTypes {
    MapChange(MapChange),
    ZoneChange(ZoneChange),
    CombatLogVersion(CombatLogVersion),
    WorldMarkerPlaced(WorldMarkerPlaced),
    Other(OtherEvent),
}

pub fn parse_line(line: &StringRecord) -> Result<Event, String> {
    /// https://wowpedia.fandom.com/wiki/COMBAT_LOG_EVENT#Event_Descriptions

    let line = line.iter().collect::<Vec<_>>();
    // println!("{:?}", line);

    // Date & Event type string
    let date_event_type = line[0].splitn(2, "  ").collect_tuple();
    if date_event_type.is_none() {
        return Err(format!("Error splitting date & event type: {}", line[0]));
    }
    let (date, event_type) = date_event_type.unwrap();

    // todo: horrible hacky way of date parsing
    let date = vec!["2024/ ", date].join("");
    let datetime = NaiveDateTime::parse_from_str(date.as_str(), "%Y/%_m/%d %H:%M:%S%.3f")
        .expect("Failed to parse date.");

    // Event type parsing
    let event_type = match event_type {
        // Special events
        "COMBAT_LOG_VERSION" => {
            EventTypes::CombatLogVersion(CombatLogVersion::parse_record(&line[1..]))
        }
        "ZONE_CHANGE" => {
            EventTypes::ZoneChange(ZoneChange::parse_record(&line[1..]))
        }
        "MAP_CHANGE" => {
            EventTypes::MapChange(MapChange::parse_record(&line[1..]))
        }
        "WORLD_MARKER_PLACED" => {
            EventTypes::WorldMarkerPlaced(WorldMarkerPlaced::parse_record(&line[1..]))
        }
        // Combat event
        _ => {
            // Parse prefix & suffix
            let prefix = EventPrefix::parse(event_type);
            if prefix.is_none() {
                return Err(format!("Error parsing prefix: {}", event_type));
            }
            let prefix = prefix.unwrap();

            let suffix = EventSuffix::parse(event_type);
            if suffix.is_none() {
                return Err(format!("Error parsing suffix: {}", event_type));
            }
            let suffix = suffix.unwrap();
            let mut offset = 1;


            // Actors
            let source = Actor::parse_record(&line[offset..offset + 4]);
            offset += 4;

            let target = Actor::parse_record(&line[offset..offset + 4]);
            offset += 4;


            // Spell info on prefix
            let prefixes = Prefix::parse_record(&line[offset..offset + 3], prefix);
            if prefixes != Prefix::SWING {
                offset += 3;
            }

            // Advanced parameters
            let advanced_params = match suffix {
                // todo: Check if covers all
                EventSuffix::DAMAGE |
                EventSuffix::HEAL |
                EventSuffix::CAST_SUCCESS |
                EventSuffix::ENERGIZE |
                EventSuffix::SPLIT => {
                    Some(AdvancedParams::parse_record(&line[offset..offset + 17]))
                }
                _ => { None }
            };
            if advanced_params.is_some() {
                offset += 17;
            }


            // Suffixes
            let suffixes = Suffix::parse_record(&line[offset..], suffix);

            EventTypes::Other(OtherEvent {
                prefix: prefixes,
                suffix: suffixes,
                source,
                target,
                advanced: advanced_params,
            })
        }
    };

    return Ok(Event {
        datetime,
        event_type,
    });
}