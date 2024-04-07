use std::i8;
use std::str::FromStr;

use strum::{EnumIter, EnumString, IntoEnumIterator};

use crate::traits::ToCamel;

/// https://warcraft.wiki.gg/wiki/COMBAT_LOG_EVENT#Spell_School
#[derive(Debug, EnumIter, PartialEq, Copy, Clone)]
pub enum SpellSchool {
    Physical = 1,
    Holy = 2,
    Fire = 4,
    Nature = 8,
    Frost = 16,
    Shadow = 32,
    Arcane = 64,
}

impl SpellSchool {
    /// Hex bitmask to vector of schools
    pub(crate) fn parse(s: &str) -> Option<Vec<SpellSchool>> {
        if s == "-1" { return None; }

        let s = if s.starts_with("0x") {
            u8::from_str_radix(s.trim_start_matches("0x"), 16)
        } else {
            u8::from_str(s)
        }.expect(&format!("Could not parse spell school as u8: {s}"));

        Some(Self::iter()
            .filter(|&e| (e as u8) & s != 0)
            .collect())
    }
}

/// https://warcraft.wiki.gg/wiki/COMBAT_LOG_EVENT#Power_Type
#[derive(Debug, Copy, Clone, EnumIter, PartialEq)]
pub enum PowerType {
    Health = -2,
    Mana = 0,
    Rage = 1,
    Focus = 2,
    Energy = 3,
    ComboPoints = 4,
    Runes = 5,
    RunicPower = 6,
    SoulShards = 7,
    LunarPower = 8,
    HolyPower = 9,
    Alternate = 10,
    Maelstrom = 11,
    Chi = 12,
    Insanity = 13,
    Obsolete = 14,
    Obsolete2 = 15,
    ArcaneCharges = 16,
    Fury = 17,
    Pain = 18,
    Essence = 19,
    RuneBlood = 20,
    RuneFrost = 21,
    RuneUnholy = 22,
    AlternateQuest = 23,
    AlternateEncounter = 24,
    AlternateMount = 25,
}

impl PowerType {
    pub(crate) fn parse(s: &str) -> Option<PowerType> {
        if s == "-1" { return None; };

        let s = i8::from_str(s)
            .expect(&format!("Failed to parse PowerType: {s}"));

        let matched = Self::iter().find(|&e| e as i8 == s)
            .expect(&format!("Failed to find matching PowerType: {s}"));

        Some(matched)
    }
}

/// https://warcraft.wiki.gg/wiki/COMBAT_LOG_EVENT#Miss_Type
#[derive(Debug, EnumString, PartialEq)]
pub enum MissType {
    Absorb,
    Block,
    Deflect,
    Dodge,
    Evade,
    Immune,
    Miss,
    Parry,
    Reflect,
    Resist,
}

/// https://warcraft.wiki.gg/wiki/COMBAT_LOG_EVENT#Aura_Type
#[derive(Debug, EnumString)]
pub enum AuraType {
    Buff,
    Debuff,
}

/// https://warcraft.wiki.gg/wiki/COMBAT_LOG_EVENT#Environmental_Type
#[derive(Debug, EnumString)]
pub enum EnvironmentalType {
    Drowning,
    Falling,
    Fatigue,
    Fire,
    Lava,
    Slime,
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::enums::{MissType, PowerType, SpellSchool};
    use crate::enums::SpellSchool::{Arcane, Holy, Nature, Shadow};
    use crate::traits::ToCamel;

    #[test]
    fn parse_spell_school() {
        assert_eq!(SpellSchool::parse("0x2".as_ref()), Some(vec![Holy]));
        assert_eq!(SpellSchool::parse("0x6A".as_ref()), Some(vec![Holy, Nature, Shadow, Arcane]));
        assert!(SpellSchool::parse("-1".as_ref()).is_none());
    }

    #[test]
    fn parse_power_type() {
        assert_eq!(PowerType::parse("-2"), Some(PowerType::Health));
        assert_eq!(PowerType::parse("-1"), None);
        assert_eq!(PowerType::parse("22"), Some(PowerType::RuneUnholy));
    }

    #[test]
    fn parse() {
        assert_eq!(MissType::from_str("Absorb"), Ok(MissType::Absorb));
        assert_eq!(MissType::from_str(&"ABSORB".to_camel_case()), Ok(MissType::Absorb));
    }
}