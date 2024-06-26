use std::i8;
use std::str::FromStr;

use anyhow::{Context, Result};
use strum::{EnumIter, EnumString, IntoEnumIterator};

use crate::traits::ToCamel;
use crate::utils::parse_num;

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
    pub(crate) fn parse(s: &str) -> Result<Option<Vec<SpellSchool>>> {
        if s == "-1" { return Ok(None); }

        let s = if s.starts_with("0x") {
            u8::from_str_radix(s.trim_start_matches("0x"), 16)
        } else {
            u8::from_str(s)
        }.with_context(|| format!("Could not parse spell school as u8: {s}"))?;

        Ok(Some(Self::iter()
            .filter(|&e| (e as u8) & s != 0)
            .collect()))
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
    pub(crate) fn parse(s: &str) -> Result<Option<PowerType>> {
        if s == "-1" { return Ok(None); };

        let s = parse_num(s)?;

        let matched = Self::iter().find(|&e| e as i8 == s)
            .with_context(|| format!("Failed to find matching PowerType: {s}"))?;

        Ok(Some(matched))
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

impl MissType {
    pub fn parse(s: &str) -> Result<Self> {
        MissType::from_str(&s.to_camel_case())
            .with_context(|| format!("Failed to parse MissType: {}", s))
    }
}

/// https://warcraft.wiki.gg/wiki/COMBAT_LOG_EVENT#Aura_Type
#[derive(Debug, EnumString)]
pub enum AuraType {
    Buff,
    Debuff,
}

impl AuraType {
    pub fn parse(s: &str) -> Result<Self> {
        AuraType::from_str(&s.to_camel_case())
            .with_context(|| format!("Failed to parse AuraType: {}", s))
    }
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

impl EnvironmentalType {
    pub fn parse(s: &str) -> Result<Self> {
        EnvironmentalType::from_str(&s.to_camel_case())
            .with_context(|| format!("Error parsing Environmental prefix: {}", s))
    }
}


#[cfg(test)]
mod tests {
    use crate::components::enums::{MissType, PowerType, SpellSchool};
    use crate::components::enums::SpellSchool::{Arcane, Holy, Nature, Shadow};

    #[test]
    fn parse_spell_school() {
        assert_eq!(SpellSchool::parse("0x2").unwrap(), Some(vec![Holy]));
        assert_eq!(SpellSchool::parse("0x6A").unwrap(), Some(vec![Holy, Nature, Shadow, Arcane]));
        assert!(SpellSchool::parse("-1").unwrap().is_none());
    }

    #[test]
    fn parse_power_type() {
        assert_eq!(PowerType::parse("-2").unwrap(), Some(PowerType::Health));
        assert_eq!(PowerType::parse("-1").unwrap(), None);
        assert_eq!(PowerType::parse("22").unwrap(), Some(PowerType::RuneUnholy));
    }

    #[test]
    fn parse() {
        assert_eq!(MissType::parse("ABSORB").unwrap(), MissType::Absorb);
    }
}