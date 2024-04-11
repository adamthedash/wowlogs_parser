use std::str::FromStr;

use anyhow::{bail, Context, Result};

use crate::components::common::{Actor, SpellInfo};
use crate::components::enums::{AuraType, MissType, PowerType, SpellSchool};
use crate::traits::ToCamel;
use crate::utils::{parse_bool, parse_num};

#[derive(Debug)]
pub enum Suffix {
    Damage {
        amount: u64,
        base_amount: u64,
        overkill: Option<u64>,
        school: Option<Vec<SpellSchool>>,
        resisted: u64,
        blocked: u64,
        absorbed: i64,
        critical: bool,
        glancing: bool,
        crushing: bool,
    },
    DamageLanded {
        amount: u64,
        base_amount: u64,
        overkill: Option<u64>,
        school: Option<Vec<SpellSchool>>,
        resisted: u64,
        blocked: u64,
        absorbed: u64,
        critical: bool,
        glancing: bool,
        crushing: bool,
    },
    Missed {
        miss_type: MissType,
        offhand: bool,
        amount_missed: u64,
        base_amount: u64,
        critical: bool,
    },
    Heal {
        amount: u64,
        base_amount: u64,
        overhealing: u64,
        absorbed: u64,
        critical: bool,
    },
    HealAbsorbed {
        actor: Option<Actor>,
        spell_info: SpellInfo,
        absorbed_amount: u64,
        total_amount: u64,
    },
    Absorbed {
        absorb_caster: Actor,
        absorb_spell_info: SpellInfo,
        absorbed_amount: i64,
        base_amount: u64,
        critical: bool,
    },
    Energize {
        amount: f32,
        over_energize: f32,
        power_type: PowerType,
        max_power: u64,
    },
    Drain {
        amount: u64,
        power_type: PowerType,
        extra_amount: u64,
        max_power: u64,
    },
    Leech {
        amount: u64,
        power_type: PowerType,
        extra_amount: u64,
    },
    Interrupt { spell_info: SpellInfo },
    Dispel {
        spell_info: SpellInfo,
        aura_type: AuraType,
    },
    DispelFailed { spell_info: SpellInfo },
    Stolen {
        spell_info: SpellInfo,
        aura_type: AuraType,
    },
    ExtraAttacks { amount: u64 },
    AuraApplied {
        aura_type: AuraType,
        amount: Option<u64>,
    },
    AuraRemoved {
        aura_type: AuraType,
        amount: Option<u64>,
    },
    AuraAppliedDose {
        aura_type: AuraType,
        amount: u64,
    },
    AuraRemovedDose {
        aura_type: AuraType,
        amount: u64,
    },
    AuraRefresh { aura_type: AuraType },
    AuraBroken { aura_type: AuraType },
    AuraBrokenSpell {
        spell_info: SpellInfo,
        aura_type: AuraType,
    },
    CastStart,
    CastSuccess,
    CastFailed { failed_type: String },
    Instakill { unconscious_on_death: bool },
    DurabilityDamage,
    DurabilityDamageAll,
    Create,
    Summon,
    Resurrect,
    EmpowerStart,
    EmpowerEnd { empowered_rank: u64 },
    EmpowerInterrupt { empowered_rank: u64 },
}

impl Suffix {
    pub fn parse(event_type: &str, line: &[&str]) -> Result<Self> {
        let matched = match event_type {
            x if x.ends_with("DAMAGE") => Self::Damage {
                amount: parse_num(line[0])?,
                base_amount: parse_num(line[1])?,
                overkill: match line[2] {
                    "-1" => None,
                    x => Some(parse_num(x)?)
                },
                school: SpellSchool::parse(line[3])?,
                resisted: parse_num(line[4])?,
                blocked: parse_num(line[5])?,
                absorbed: parse_num(line[6])?,
                critical: parse_bool(line[7])?,
                glancing: parse_bool(line[8])?,
                crushing: parse_bool(line[9])?,
            },

            x if x.ends_with("DAMAGE_LANDED") => Self::DamageLanded {
                amount: parse_num(line[0])?,
                base_amount: parse_num(line[1])?,
                overkill: match line[2] {
                    "-1" => None,
                    x => Some(parse_num(x)?)
                },
                school: SpellSchool::parse(line[3])?,
                resisted: parse_num(line[4])?,
                blocked: parse_num(line[5])?,
                absorbed: parse_num(line[6])?,
                critical: parse_bool(line[7])?,
                glancing: parse_bool(line[8])?,
                crushing: parse_bool(line[9])?,
            },

            x if x.ends_with("MISSED") => {
                let miss_type = MissType::parse(line[0])?;

                let (amount_missed, base_amount, critical) = match miss_type {
                    MissType::Absorb => (
                        parse_num(line[2])?,
                        parse_num(line[3])?,
                        parse_bool(line[4])?
                    ),
                    _ => (0, 0, false)
                };

                Self::Missed {
                    miss_type,
                    offhand: parse_bool(line[1])?,
                    amount_missed,
                    base_amount,
                    critical,
                }
            }

            x if x.ends_with("HEAL") => Self::Heal {
                amount: parse_num(line[0])?,
                base_amount: parse_num(line[1])?,
                overhealing: parse_num(line[2])?,
                absorbed: parse_num(line[3])?,
                critical: parse_bool(line[4])?,
            },

            x if x.ends_with("HEAL_ABSORBED") => Self::HealAbsorbed {
                actor: Actor::parse(&line[..4])?,
                spell_info: SpellInfo::parse(&line[4..7])?,
                absorbed_amount: parse_num(line[7])?,
                total_amount: parse_num(line[8])?,
            },

            x if x.ends_with("ABSORBED") => Self::Absorbed {
                absorb_caster: Actor::parse(&line[..4])?.unwrap(),
                absorb_spell_info: SpellInfo::parse(&line[4..7])?,
                absorbed_amount: parse_num(line[7])?,
                base_amount: parse_num(line[8])?,
                critical: parse_bool(line[9])?,
            },

            x if x.ends_with("ENERGIZE") => Self::Energize {
                amount: parse_num(line[0])?,
                over_energize: parse_num(line[1])?,
                power_type: PowerType::parse(line[2])?
                    .with_context(|| format!("Invalid power type: {}", line[2]))?,
                max_power: parse_num(line[3])?,
            },

            x if x.ends_with("DRAIN") => Self::Drain {
                amount: parse_num(line[0])?,
                power_type: PowerType::parse(line[1])?
                    .with_context(|| format!("Invalid power type: {}", line[1]))?,
                extra_amount: parse_num(line[2])?,
                max_power: parse_num(line[3])?,
            },

            x if x.ends_with("LEECH") => Self::Leech {
                amount: parse_num(line[0])?,
                power_type: PowerType::parse(line[1])?
                    .with_context(|| format!("Invalid power type: {}", line[1]))?,
                extra_amount: parse_num(line[2])?,
            },

            x if x.ends_with("EMPOWER_INTERRUPT") => Self::EmpowerInterrupt {
                empowered_rank: parse_num(line[0])?
            },

            x if x.ends_with("INTERRUPT") => Self::Interrupt {
                spell_info: SpellInfo::parse(&line[..3])?,
            },

            x if x.ends_with("DISPEL") => Self::Dispel {
                spell_info: SpellInfo::parse(&line[..3])?,
                aura_type: AuraType::from_str(&line[3].to_camel_case())
                    .with_context(|| format!("Failed to parse AuraType: {}", line[3]))?,
            },

            x if x.ends_with("DISPEL_FAILED") => Self::DispelFailed {
                spell_info: SpellInfo::parse(&line[..3])?,
            },

            x if x.ends_with("STOLEN") => Self::Stolen {
                spell_info: SpellInfo::parse(&line[..3])?,
                aura_type: AuraType::from_str(&line[3].to_camel_case())
                    .with_context(|| format!("Failed to parse AuraType: {}", line[3]))?,
            },

            x if x.ends_with("EXTRA_ATTACKS") => Self::ExtraAttacks {
                amount: parse_num(line[0])?
            },

            x if x.ends_with("AURA_APPLIED") => {
                let amount = if line.len() < 2 { None } else { Some(parse_num(line[1])?) };

                Self::AuraApplied {
                    aura_type: AuraType::from_str(&line[0].to_camel_case())
                        .with_context(|| format!("Failed to parse AuraType: {}", line[0]))?,
                    amount,
                }
            }

            x if x.ends_with("AURA_REMOVED") => {
                let amount = if line.len() < 2 { None } else { Some(parse_num(line[1])?) };

                Self::AuraRemoved {
                    aura_type: AuraType::from_str(&line[0].to_camel_case())
                        .with_context(|| format!("Failed to parse AuraType: {}", line[0]))?,
                    amount,
                }
            }

            x if x.ends_with("AURA_APPLIED_DOSE") => Self::AuraAppliedDose {
                aura_type: AuraType::from_str(&line[0].to_camel_case())
                    .with_context(|| format!("Failed to parse AuraType: {}", line[0]))?,
                amount: parse_num(line[1])?,
            },

            x if x.ends_with("AURA_REMOVED_DOSE") => Self::AuraRemovedDose {
                aura_type: AuraType::from_str(&line[0].to_camel_case())
                    .with_context(|| format!("Failed to parse AuraType: {}", line[0]))?,
                amount: parse_num(line[1])?,
            },

            x if x.ends_with("AURA_REFRESH") => Self::AuraRefresh {
                aura_type: AuraType::from_str(&line[0].to_camel_case())
                    .with_context(|| format!("Failed to parse AuraType: {}", line[0]))?,
            },

            x if x.ends_with("AURA_BROKEN") => Self::AuraBroken {
                aura_type: AuraType::from_str(&line[0].to_camel_case())
                    .with_context(|| format!("Failed to parse AuraType: {}", line[0]))?,
            },

            x if x.ends_with("AURA_BROKEN_SPELL") => Self::AuraBrokenSpell {
                spell_info: SpellInfo::parse(&line[..3])?,
                aura_type: AuraType::from_str(&line[3].to_camel_case())
                    .with_context(|| format!("Failed to parse AuraType: {}", line[3]))?,
            },

            x if x.ends_with("CAST_START") => Self::CastStart,

            x if x.ends_with("CAST_SUCCESS") => Self::CastSuccess,

            x if x.ends_with("CAST_FAILED") => Self::CastFailed {
                failed_type: line[0].to_string(),
            },

            x if x.ends_with("INSTAKILL") => Self::Instakill {
                unconscious_on_death: parse_bool(line[0])?,
            },

            x if x.ends_with("DURABILITY_DAMAGE") => Self::DurabilityDamage,

            x if x.ends_with("DURABILITY_DAMAGE_ALL") => Self::DurabilityDamageAll,

            x if x.ends_with("CREATE") => Self::Create,

            x if x.ends_with("SUMMON") => Self::Summon,

            x if x.ends_with("RESURRECT") => Self::Resurrect,

            x if x.ends_with("EMPOWER_START") => Self::EmpowerStart,

            x if x.ends_with("EMPOWER_END") => Self::EmpowerEnd {
                empowered_rank: parse_num(line[0])?,
            },

            _ => bail!("Unknown suffix: {}", event_type)
        };

        Ok(matched)
    }

    pub fn has_advanced_params(event_type: &str) -> Result<bool> {
        // todo: fill these in - surely a better way to do this
        let advanced_suffixes = [
            "DAMAGE",
            "DAMAGE_LANDED",
            "HEAL",
            "CAST_SUCCESS",
            "ENERGIZE",
            "DRAIN",
            "LEECH",
            "STOLEN",
            "CAST_SUCCESS",
        ];
        let non_advanced_suffixes = [
            "AURA_APPLIED",
            "AURA_REMOVED",
            "MISSED",
            "HEAL_ABSORBED",
            "ABSORBED",
            "EMPOWER_INTERRUPT",
            "INTERRUPT",
            "DISPEL_FAILED",
            "EXTRA_ATTACKS",
            "AURA_APPLIED_DOSE",
            "AURA_REMOVED_DOSE",
            "AURA_REFRESH",
            "AURA_BROKEN",
            "AURA_BROKEN_SPELL",
            "CAST_START",
            "CAST_FAILED",
            "INSTAKILL",
            "DURABILITY_DAMAGE",
            "DURABILITY_DAMAGE_ALL",
            "CREATE",
            "SUMMON",
            "RESURRECT",
            "EMPOWER_START",
            "EMPOWER_END",
            "DISPEL",
        ];

        let matched = match event_type {
            x if advanced_suffixes.iter().any(|s| x.ends_with(s)) => true,
            x if non_advanced_suffixes.iter().any(|s| x.ends_with(s)) => false,
            _ => bail!("Unknown suffix: {}", event_type)
        };

        Ok(matched)
    }
}

#[cfg(test)]
mod tests {
    use super::Suffix;

    #[test]
    fn parse() {
        let event_type = "SPELL_DAMAGE";
        let line = vec!["23134", "23133", "-1", "2", "0", "0", "0", "nil", "nil", "nil"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_DAMAGE";
        let line = vec!["22844", "26082", "-1", "4", "0", "0", "-2025", "nil", "nil", "nil"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_PERIODIC_MISSED";
        let line = vec!["ABSORB", "nil", "9478", "11175", "nil"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_HEAL";
        let line = vec!["2621", "2621", "0", "0", "1"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_ABSORBED";
        let line = vec!["Player-1587-0F81497D", "Huisarts-Arathor", "0x514", "0x0", "47753", "Divine Aegis", "0x2", "983", "56699", "nil"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_ABSORBED";
        let line = vec!["Player-1329-0A0800FA", "Foxgates-Ravencrest", "0x512", "0x0", "386124", "Fel Armor", "0x20", "-2900", "48673", "nil"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_PERIODIC_ENERGIZE";
        let line = vec!["1.0000", "0.0000", "5", "6"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_DRAIN";
        let line = vec!["25", "3", "0", "160"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_EMPOWER_INTERRUPT";
        let line = vec!["0"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_AURA_APPLIED";
        let line = vec!["DEBUFF"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let line = vec!["DEBUFF", "123"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_AURA_REMOVED";
        let line = vec!["DEBUFF"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let line = vec!["DEBUFF", "123"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_AURA_APPLIED_DOSE";
        let line = vec!["DEBUFF", "123"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_AURA_REMOVED_DOSE";
        let line = vec!["DEBUFF", "123"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_AURA_REFRESH";
        let line = vec!["DEBUFF"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_AURA_BROKEN";
        let line = vec!["DEBUFF"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_AURA_BROKEN_SPELL";
        let line = vec!["360194", "Deathmark", "1", "DEBUFF"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_CAST_START";
        let line = vec![];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_CAST_SUCCESS";
        let line = vec![];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_CAST_FAILED";
        let line = vec!["Not yet recovered"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_SUMMON";
        let line = vec![];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_RESURRECT";
        let line = vec![];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_EMPOWER_START";
        let line = vec![];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_EMPOWER_END";
        let line = vec!["1"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SWING_DAMAGE_LANDED";
        let line = vec!["16898", "12070", "-1", "1", "0", "0", "0", "1", "nil", "nil"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_HEAL_ABSORBED";
        let line = vec!["Creature-0-4233-2549-14868-54983-00004E66CB", "Treant", "0x2114", "0x0", "422382", "Wild Growth", "0x8", "2585", "2585"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_HEAL_ABSORBED";
        let line = vec!["0000000000000000", "Unknown", "0x80000000", "0x80000000", "422382", "Wild Growth", "0x8", "2438", "2438"];
        let parsed = Suffix::parse(event_type, &line);
        println!("{:?}", parsed);
    }
}