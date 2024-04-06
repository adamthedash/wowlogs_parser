use std::str::FromStr;
use std::u64;

use strum::{AsRefStr, EnumIter, IntoEnumIterator};

use crate::traits::FromRecord;

#[derive(Debug, PartialEq)]
pub enum Prefix {
    SWING,
    RANGE(Spell),
    SPELL(Spell),
    SPELL_PERIODIC(Spell),
    SPELL_BUILDING(Spell),
    DAMAGE(Spell),
}

impl Prefix {
    pub fn parse_record(line: &[&str], prefix_type: EventPrefix) -> Self {
        match prefix_type {
            EventPrefix::SWING => { Self::SWING }
            EventPrefix::RANGE => { Self::RANGE(Spell::parse_record(line)) }
            EventPrefix::SPELL => { Self::SPELL(Spell::parse_record(line)) }
            EventPrefix::SPELL_PERIODIC => { Self::SPELL_PERIODIC(Spell::parse_record(line)) }
            EventPrefix::SPELL_BUILDING => { Self::SPELL_BUILDING(Spell::parse_record(line)) }
            EventPrefix::DAMAGE => { Self::DAMAGE(Spell::parse_record(line)) }
        }
    }
}

#[derive(Debug, AsRefStr, EnumIter, PartialEq)]
pub enum EventPrefix {
    SWING,
    RANGE,
    SPELL,
    SPELL_PERIODIC,
    SPELL_BUILDING,
    DAMAGE,
}

impl EventPrefix {
    pub fn parse(s: &str) -> Option<Self> {
        for e in Self::iter() {
            if s.starts_with(e.as_ref()) {
                return Some(e);
            }
        }

        return None;
    }
}

#[derive(Debug, PartialEq)]
pub struct Spell {
    id: u64,
    name: String,
    school: u64,
}

impl FromRecord for Spell {
    fn parse_record(line: &[&str]) -> Self {
        Self {
            id: u64::from_str(line[0]).expect("Error parsing spell ID"),
            name: line[1].to_string(),
            school: u64::from_str_radix(line[2].trim_start_matches("0x"), 16)
                .expect("Error parsing spell school"),
        }
    }
}
