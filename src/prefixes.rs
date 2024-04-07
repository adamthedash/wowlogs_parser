use std::str::FromStr;

use crate::common_components::SpellInfo;
use crate::enums::EnvironmentalType;
use crate::traits::ToCamel;

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
    pub(crate) fn parse(event_type: &str, line: &[&str]) -> Self {
        match event_type {
            x if x.starts_with("SWING") => Self::Swing,
            x if x.starts_with("RANGE") => Self::Range(SpellInfo::parse_record(&line[..3])),
            x if x.starts_with("SPELL_PERIODIC") => Self::SpellPeriodic(SpellInfo::parse_record(&line[..3])),
            x if x.starts_with("SPELL_BUILDING") => Self::SpellBuilding(SpellInfo::parse_record(&line[..3])),
            x if x.starts_with("SPELL") => Self::Spell({
                match line.len() {
                    0 => None,
                    3 => Some(SpellInfo::parse_record(&line[..3])),
                    _ => panic!("Bad number of entries for Spell")
                }
            }),
            x if x.starts_with("ENVIRONMENTAL") => Self::Environmental(EnvironmentalType::from_str(&line[0].to_camel_case())
                .expect(&format!("Error parsing Environmental prefix: {}", line[0]))),
            _ => panic!("Unknown prefix: {}", event_type)
        }
    }

    pub(crate) fn entries_to_consume(event_type: &str) -> usize {
        match event_type {
            x if x.starts_with("SWING") => 0,
            x if x.starts_with("RANGE") |
                x.starts_with("SPELL_PERIODIC") |
                x.starts_with("SPELL_BUILDING") |
                x.starts_with("SPELL") => 3,
            x if x.starts_with("ENVIRONMENTAL") => 1,
            _ => panic!("Unknown prefix: {}", event_type)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prefixes::Prefix;

    #[test]
    fn parse() {
        let event_type = "SPELL_PERIODIC_HEAL";
        let lines = vec!["8936", "Regrowth", "0x8"];
        let parsed = Prefix::parse(event_type, &lines);

        let event_type = "SWING_DAMAGE";
        let lines = vec![];
        let parsed = Prefix::parse(event_type, &lines);

        let event_type = "SPELL_AURA_APPLIED";
        let lines = vec!["6673", "Battle Shout", "0x1"];
        let parsed = Prefix::parse(event_type, &lines);
    }
}
