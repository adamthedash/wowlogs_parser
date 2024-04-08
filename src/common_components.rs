use std::u64;

use anyhow::{Context, Result};

use crate::enums::SpellSchool;
use crate::guid::GUID;
use crate::utils::{parse_hex, parse_num};

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
    pub fn parse_record(line: &[&str]) -> Result<Self> {
        assert_eq!(line.len(), 3);

        let spell_school = SpellSchool::parse(line[2])?
            .with_context(|| format!("Error parsing spell school: {}", line[2]))?;

        Ok(Self {
            spell_id: parse_num(line[0])?,
            spell_name: line[1].to_string(),
            spell_school,
        })
    }
}

impl Actor {
    pub fn parse(line: &[&str]) -> Result<Option<Self>> {
        let guid = GUID::parse(line[0])?;
        let guid = if let Some(g) = guid { g } else { return Ok(None); };

        let flags = parse_hex(line[2]).context("Error parsing target flags")?;

        let raid_flags = match line[3] {
            "nil" => None,
            x => Some(parse_hex(x).context("Error parsing target raid flags")?)
        };

        Ok(Some(Self {
            guid,
            name: line[1].to_string(),
            flags,
            raid_flags,

        }))
    }
}


#[cfg(test)]
mod tests {
    use crate::common_components::{Actor, SpellInfo};

    #[test]
    fn parse_spell_info() {
        let line = vec!["8936", "Regrowth", "0x8"];
        let _parsed = SpellInfo::parse_record(&line);
    }

    #[test]
    fn parse_actor() {
        let line = vec!["Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0"];
        let parsed = Actor::parse(&line);
        assert!(parsed.is_ok_and(|x| x.is_some()));

        let line = vec!["0000000000000000", "nil", "0x80000000", "0x80000000"];
        let parsed = Actor::parse(&line);
        assert!(parsed.is_ok_and(|x| x.is_none()));

        let line = vec!["Creature-0-4233-2549-14868-200927-00004E8C97", "Smolderon", "0000000000000000", "nil"];
        let parsed = Actor::parse(&line);
        assert!(parsed.is_ok_and(|a| a.is_some_and(|a| a.raid_flags.is_none())));
    }
}