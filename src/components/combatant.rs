use anyhow::{bail, Context, ensure, Result};
use regex::Regex;

use crate::components::guid::GUID;
use crate::utils::{match_replace_all, parse_num};

#[derive(Debug)]
pub struct CharacterStats {
    strength: u64,
    agility: u64,
    stamina: u64,
    intelligence: u64,
    dodge: u64,
    parry: u64,
    block: u64,
    crit_melee: u64,
    crit_ranged: u64,
    crit_spell: u64,
    speed: u64,
    leech: u64,
    haste_melee: u64,
    haste_range: u64,
    haste_spell: u64,
    avoidance: u64,
    mastery: u64,
    versatility_damage_done: u64,
    versatility_healing_done: u64,
    versatility_damage_taken: u64,
    armor: u64,
}

impl CharacterStats {
    pub fn parse(line: &[&str]) -> Result<Self> {
        Ok(Self {
            strength: parse_num(line[0])?,
            agility: parse_num(line[1])?,
            stamina: parse_num(line[2])?,
            intelligence: parse_num(line[3])?,
            dodge: parse_num(line[4])?,
            parry: parse_num(line[5])?,
            block: parse_num(line[6])?,
            crit_melee: parse_num(line[7])?,
            crit_ranged: parse_num(line[8])?,
            crit_spell: parse_num(line[9])?,
            speed: parse_num(line[10])?,
            leech: parse_num(line[11])?,
            haste_melee: parse_num(line[12])?,
            haste_range: parse_num(line[13])?,
            haste_spell: parse_num(line[14])?,
            avoidance: parse_num(line[15])?,
            mastery: parse_num(line[16])?,
            versatility_damage_done: parse_num(line[17])?,
            versatility_healing_done: parse_num(line[18])?,
            versatility_damage_taken: parse_num(line[19])?,
            armor: parse_num(line[20])?,
        })
    }
}

#[derive(Debug)]
pub struct PVPStats {
    honor_level: u64,
    season: u64,
    rating: u64,
    tier: u64,
}

impl PVPStats {
    pub fn parse(line: &[&str]) -> Result<Self> {
        Ok(Self {
            honor_level: parse_num(line[0])?,
            season: parse_num(line[1])?,
            rating: parse_num(line[2])?,
            tier: parse_num(line[3])?,
        })
    }
}

#[derive(Debug)]
pub enum Faction {
    Horde,
    Alliance,
    // Neutral?
}

impl Faction {
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "0" => Ok(Self::Horde),
            "1" => Ok(Self::Alliance),
            _ => bail!(format!("Failed to parse Faction: {:?}", s))
        }
    }
}

pub type PVPTalents = [u64; 4];

trait PrimitiveParse<T> {
    fn parse(s: &str) -> Result<T>;
}

impl PrimitiveParse<PVPTalents> for PVPTalents {
    fn parse(s: &str) -> Result<Self> {
        // s: "(a,b,c,d),"
        let ids: PVPTalents = s[1..s.len() - 2]
            .split(',')
            .map(parse_num)
            .collect::<Result<Vec<u64>>>()?
            // Vec -> [u64]
            .as_slice()
            .try_into()
            .with_context(|| format!("Incorrect number of ids: {}", s))?;

        Ok(ids)
    }
}

pub struct ClassTalents {}

#[derive(Debug)]
pub struct CombatantInfo {
    guid: GUID,
    faction: Faction,
    stats: CharacterStats,
    // class_talents: todo!(),
    pvp_talents: PVPTalents,
    // artifact_traits: todo!(),
    // equipped_items: todo!(),
    // interesting_auras: todo!(),
    pvp_stats: PVPStats,
}

impl CombatantInfo {
    pub fn parse(line: &[&str]) -> Result<Self> {
        let line2 = line.join(",");

        // Pull out square brackets (class talents, equipped items, interesting auras
        let re = Regex::new(r"(\[.+?]),").unwrap();
        let (matches, line3) = match_replace_all(&re, &line2);
        ensure!(matches.len() == 3, "incorrect number of [...] sections found. Expected 3, found {}", matches.len());

        // Pull out remaining round brackets (pvp talents)
        let re = Regex::new(r"\([\d,?]+\),").unwrap();
        let (matches, line4) = match_replace_all(&re, &line3);
        ensure!(matches.len() == 1, "incorrect number of (...) sections found. Expected 1, found {}", matches.len());
        let pvp_talents = matches[0].as_str();

        // Re-split todo: use csv to make sure we escape properly
        let line5 = line4.split(',').collect::<Vec<_>>();


        Ok(Self {
            guid: GUID::parse(line5[0])?.unwrap(),
            faction: Faction::parse(line5[1])?,
            stats: CharacterStats::parse(&line5[2..23])?,
            pvp_talents: PVPTalents::parse(pvp_talents)?,
            pvp_stats: PVPStats::parse(&line5[23..])?,
        })
    }
}