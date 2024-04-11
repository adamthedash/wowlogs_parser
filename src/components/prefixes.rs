use anyhow::{bail, Result};

use crate::components::common::SpellInfo;
use crate::components::enums::EnvironmentalType;

#[derive(Debug)]
pub enum Prefix {
    Swing,
    Range(SpellInfo),
    Spell(Option<SpellInfo>),
    SpellPeriodic(SpellInfo),
    SpellBuilding(SpellInfo),
    Environmental(EnvironmentalType),
}

impl Prefix {
    pub(crate) fn parse(event_type: &str, line: &[&str]) -> Result<Self> {
        let matched = match event_type {
            x if x.starts_with("SWING") => Self::Swing,
            x if x.starts_with("RANGE") => Self::Range(SpellInfo::parse(&line[..3])?),
            x if x.starts_with("SPELL_PERIODIC") => Self::SpellPeriodic(SpellInfo::parse(&line[..3])?),
            x if x.starts_with("SPELL_BUILDING") => Self::SpellBuilding(SpellInfo::parse(&line[..3])?),
            x if x.starts_with("SPELL") => Self::Spell({
                match line.len() {
                    0 => None,
                    3 => Some(SpellInfo::parse(&line[..3])?),
                    _ => bail!("Bad number of entries for Spell")
                }
            }),
            x if x.starts_with("ENVIRONMENTAL") => Self::Environmental(
                EnvironmentalType::parse(line[0])?
            ),
            _ => bail!("Unknown prefix: {}", event_type)
        };

        Ok(matched)
    }

    pub(crate) fn entries_to_consume(event_type: &str) -> Result<usize> {
        let matched = match event_type {
            x if x.starts_with("SWING") => 0,
            x if x.starts_with("RANGE") |
                x.starts_with("SPELL_PERIODIC") |
                x.starts_with("SPELL_BUILDING") |
                x.starts_with("SPELL") => 3,
            x if x.starts_with("ENVIRONMENTAL") => 1,
            _ => bail!("Unknown prefix: {}", event_type)
        };

        Ok(matched)
    }
}

#[cfg(test)]
mod tests {
    use super::Prefix;

    #[test]
    fn parse() {
        let event_type = "SPELL_PERIODIC_HEAL";
        let lines = vec!["8936", "Regrowth", "0x8"];
        let _parsed = Prefix::parse(event_type, &lines);

        let event_type = "SWING_DAMAGE";
        let lines = vec![];
        let _parsed = Prefix::parse(event_type, &lines);

        let event_type = "SPELL_AURA_APPLIED";
        let lines = vec!["6673", "Battle Shout", "0x1"];
        let _parsed = Prefix::parse(event_type, &lines);
    }
}
