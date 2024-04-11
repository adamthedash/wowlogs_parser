use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use itertools::Itertools;

use crate::components::{
    advanced::AdvancedParams,
    common::Actor,
    prefixes::Prefix,
    special,
    suffixes::Suffix,
};

#[derive(Debug)]
pub enum EventType {
    Special {
        name: String,
        details: special::Special,
    },
    Standard {
        name: String,
        source: Option<Actor>,
        target: Option<Actor>,
        prefix: Prefix,
        advanced_params: Option<AdvancedParams>,
        suffix: Suffix,
    },
}

impl EventType {
    fn parse(event_type: &str, line: &[&str]) -> Result<Self> {
        // Match against any special events
        let special = special::Special::parse(event_type, line);
        if let Ok(s) = special {
            return Ok(Self::Special {
                name: event_type.to_string(),
                details: s,
            });
        }

        // match against standard but specially named events
        let specially_named_events = HashMap::from([
            ("DAMAGE_SPLIT", "SPELL_DAMAGE"),
            ("DAMAGE_SHIELD", "SPELL_DAMAGE"),
            ("DAMAGE_SHIELD_MISSED", "SPELL_MISSED"),
        ]);


        let (name, event_type) = match specially_named_events.get(&event_type) {
            None => (event_type, event_type),
            Some(&val) => (event_type, val)
        };


        // Fallback to standard one
        let source = Actor::parse(&line[..4])?;
        let target = Actor::parse(&line[4..8])?;

        let to_consume = match event_type {
            // Special case: ABSORB may or may not contain spell info
            // we have no way to tell without attempting to parse and catching fails
            e if e == "SPELL_ABSORBED"
                && u64::from_str(line[8]).is_err() => 0,
            _ => Prefix::entries_to_consume(event_type)?
        };

        let prefix = Prefix::parse(event_type, &line[8..8 + to_consume])?;
        let mut offset = 8 + to_consume;

        let advanced = if Suffix::has_advanced_params(event_type)? {
            let a = AdvancedParams::parse(&line[offset..offset + 17])?;
            offset += 17;
            Some(a)
        } else {
            None
        };

        let suffixes = Suffix::parse(event_type, &line[offset..])?;

        Ok(Self::Standard {
            name: name.to_string(),
            source,
            target,
            prefix,
            advanced_params: advanced,
            suffix: suffixes,
        })
    }
}


#[derive(Debug)]
pub struct Event {
    pub timestamp: NaiveDateTime,
    pub event_type: EventType,
}

impl Event {
    pub(crate) fn parse(line: &[&str]) -> Result<Self> {
        // Split timestamp & event type
        let (timestamp, event_type) = if line[0] == "COMBAT_LOG_VERSION" {
            (
                NaiveDateTime::parse_from_str("2024/01/01 00:00:00.000", "%Y/%_m/%d %H:%M:%S%.3f").unwrap(),
                line[0]
            )
        } else {
            let (date, event_type) = line[0].splitn(2, "  ")
                .collect_tuple()
                .with_context(|| format!("Error splitting date & event type: {}", line[0]))?;

            // todo: horrible hacky way of date parsing
            let date = ["2024/ ", date].join("");
            let datetime = NaiveDateTime::parse_from_str(date.as_str(), "%Y/%_m/%d %H:%M:%S%.3f")
                .with_context(|| "Failed to parse date.")?;

            (datetime, event_type)
        };

        Ok(Self {
            timestamp,
            event_type: EventType::parse(event_type, &line[1..])
                .with_context(|| format!("Error parsing line: {:?}", line))?,
        })
    }
}


#[cfg(test)]
mod tests {
    use crate::components::events::{Event, EventType};

    #[test]
    fn parse_event_type() {
        let event_type = "COMBAT_LOG_VERSION";
        let line = vec!["20", "ADVANCED_LOG_ENABLED", "1", "BUILD_VERSION", "10.2.6", "PROJECT_ID", "1"];
        let parsed = EventType::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_PERIODIC_HEAL";
        let line = vec!["Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "Creature-0-1469-2549-12530-210177-000011428F", "Tormented Ancient", "0xa18", "0x0", "8936", "Regrowth", "0x8", "Creature-0-1469-2549-12530-210177-000011428F", "0000000000000000", "5927873", "7468728", "0", "0", "5043", "0", "1", "0", "0", "0", "3295.44", "13209.11", "2232", "3.4506", "72", "2557", "2557", "0", "0", "nil"];
        let parsed = EventType::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_CAST_SUCCESS";
        let line = vec!["Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "0000000000000000", "nil", "0x80000000", "0x80000000", "1850", "Dash", "0x1", "Player-1329-09AF0ACF", "0000000000000000", "846460", "846460", "16429", "15797", "5313", "94077", "3", "100", "100", "0", "3110.69", "13146.01", "2232", "0.7478", "486"];
        let parsed = EventType::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_AURA_REMOVED";
        let line = vec!["Player-1084-0934CD1D", "Neversman-TarrenMill", "0x514", "0x0", "Player-1379-0814BAB7", "Kuro-Zul'jin", "0x40512", "0x4", "6673", "Battle Shout", "0x1", "BUFF"];
        let parsed = EventType::parse(event_type, &line);
        println!("{:?}", parsed);
    }

    #[test]
    fn parse_event() {
        let line = vec!["4/6 14:09:44.867  SPELL_PERIODIC_HEAL", "Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "Creature-0-1469-2549-12530-210177-000011428F", "Tormented Ancient", "0xa18", "0x0", "8936", "Regrowth", "0x8", "Creature-0-1469-2549-12530-210177-000011428F", "0000000000000000", "5927873", "7468728", "0", "0", "5043", "0", "1", "0", "0", "0", "3295.44", "13209.11", "2232", "3.4506", "72", "2557", "2557", "0", "0", "nil"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());

        let line = vec!["COMBAT_LOG_VERSION", "20", "ADVANCED_LOG_ENABLED", "1", "BUILD_VERSION", "10.2.6", "PROJECT_ID", "1"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());

        let line = vec!["4/6 14:09:44.867  COMBAT_LOG_VERSION", "20", "ADVANCED_LOG_ENABLED", "1", "BUILD_VERSION", "10.2.6", "PROJECT_ID", "1"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());

        let line = vec!["4/6 14:02:07.362  SWING_MISSED", "Player-1335-0A264B4C", "Sønike-Ysondre", "0x514", "0x0", "Creature-0-1469-2549-12530-209333-000011428A", "Gnarlroot", "0x10a48", "0x0", "MISS", "1"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_emote() {
        let line = vec!["4/11 22:47:58.605  EMOTE", "Player-1329-09AF0ACF", "Adamthebash", "Player-1329-09AF0ACF", "Adamthebash", "Turn back! The Emerald Dream is clouding your mind..."];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_env_damage() {
        let line = vec!["4/11 22:42:01.100  ENVIRONMENTAL_DAMAGE", "0000000000000000", "nil", "0x80000000", "0x80000000", "Player-1329-070EBCFC", "Naladrem-Ravencrest", "0x518", "0x0", "Player-1329-070EBCFC", "0000000000000000", "815216", "866544", "14879", "1421", "5217", "0", "17", "109", "120", "0", "-931.46", "2546.12", "2133", "4.8479", "484", "Falling", "51328", "51328", "0", "1", "0", "0", "0", "nil", "nil", "nil"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_bres() {
        let line = vec!["4/11 22:38:54.708  SPELL_CAST_SUCCESS", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "Corpse-0-1465-2454-103-0-000018584E", "Unknown", "0x4228", "0x0", "20484", "Rebirth", "0x8", "Player-1329-09AF0ACF", "0000000000000000", "732698", "846460", "16347", "15718", "5632", "0", "0", "250000", "250000", "5000", "66.53", "3330.43", "2133", "4.7368", "486"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }
}