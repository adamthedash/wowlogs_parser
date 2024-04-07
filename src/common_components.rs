use std::str::FromStr;
use std::u64;

use crate::enums::SpellSchool;
use crate::guid::GUID;

#[derive(Debug)]
pub struct SpellInfo {
    spell_id: u64,
    spell_name: String,
    spell_school: Vec<SpellSchool>,
}

#[derive(Debug)]
pub struct Actor {
    guid: GUID,
    pub name: String,
    flags: u64,
    raid_flags: Option<u64>,
}

impl SpellInfo {
    pub fn parse_record(line: &[&str]) -> Self {
        assert_eq!(line.len(), 3);

        Self {
            spell_id: u64::from_str(line[0])
                .expect(&format!("Error parsing u64: {}", line[0])),
            spell_name: line[1].to_string(),
            spell_school: SpellSchool::parse(line[2])
                .expect(&format!("Error parsing spell school: {}", line[2])),
        }
    }
}

impl Actor {
    pub fn parse(line: &[&str]) -> Option<Self> {
        let guid = GUID::parse(line[0])?;

        let raid_flags = match line[3] {
            x if x == "nil" => None,
            x => Some(u64::from_str_radix(x.trim_start_matches("0x"), 16)
                .expect("Error parsing target raid flags"))
        };

        Some(Self {
            guid,
            name: line[1].to_string(),
            flags: u64::from_str_radix(line[2].trim_start_matches("0x"), 16)
                .expect("Error parsing target flags"),
            raid_flags,

        })
    }
}


#[cfg(test)]
mod tests {
    use crate::common_components::{Actor, SpellInfo};

    #[test]
    fn parse_spell_info() {
        let line = vec!["8936", "Regrowth", "0x8"];
        let parsed = SpellInfo::parse_record(&line);
    }

    #[test]
    fn parse_actor() {
        let line = vec!["Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0"];
        let parsed = Actor::parse(&line);
        assert!(parsed.is_some());

        let line = vec!["0000000000000000", "nil", "0x80000000", "0x80000000"];
        let parsed = Actor::parse(&line);
        assert!(parsed.is_none());

        let line = vec!["Creature-0-4233-2549-14868-200927-00004E8C97", "Smolderon", "0000000000000000", "nil"];
        let parsed = Actor::parse(&line);
        assert!(parsed.is_some_and(|a| a.raid_flags.is_none()));
    }
}