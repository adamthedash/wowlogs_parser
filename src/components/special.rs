use anyhow::Result;

use crate::components::combatant;
use crate::components::common::Actor;
use crate::components::guid::GUID;
use crate::utils::{parse_bool, parse_num};

#[derive(Debug)]
pub enum Special {
    EnchantApplied {
        source: Option<Actor>,
        target: Option<Actor>,
        spell_name: String,
        item_id: u64,
        item_name: String,
    },
    EnchantRemoved {
        source: Option<Actor>,
        target: Option<Actor>,
        spell_name: String,
        item_id: u64,
        item_name: String,
    },
    PartyKill {
        source: Option<Actor>,
        target: Option<Actor>,
        unconscious_on_death: bool,
    },
    UnitDied {
        source: Option<Actor>,
        target: Option<Actor>,
        unconscious_on_death: bool,
    },
    UnitDestroyed {
        source: Option<Actor>,
        target: Option<Actor>,
        unconscious_on_death: bool,
    },
    UnitDissipates {
        source: Option<Actor>,
        target: Option<Actor>,
        unconscious_on_death: bool,
    },
    CombatLogInfo {
        log_version: u64,
        advanced_log_enabled: bool,
        build_version: String,
        project_id: u64,
    },
    ZoneChange {
        instance_id: u64,
        zone_name: String,
        id: u64,
    },
    MapChange {
        ui_map_id: u64,
        ui_map_name: String,
        x0: f32,
        x1: f32,
        y0: f32,
        y1: f32,
    },
    EncounterStart {
        encounter_id: u64,
        encounter_name: String,
        difficulty_id: u64,
        group_size: u64,
        instance_id: u64,
    },
    EncounterEnd {
        encounter_id: u64,
        encounter_name: String,
        difficulty_id: u64,
        group_size: u64,
        success: bool,
        fight_time: u64,
    },
    WorldMarkerPlaced {
        instance_id: u64,
        marker: u64,
        x: f32,
        y: f32,
    },
    WorldMarkerRemoved {
        marker: u64
    },
    EmoteStandard {
        actor: Option<Actor>,
        text: String,
    },
    EmoteEnvironmental {
        source_guid: Option<GUID>,
        source_name: String,
        target_guid: Option<GUID>,
        target_name: String,
        text: String,
    },
    CombatantInfo(combatant::CombatantInfo),
    ChallengeModeStart {
        zone_name: String,
        instance_id: u64,
        challenge_mode_id: u64,
        keystone_level: u64,
        affix_ids: Vec<u64>,
    },
    ChallengeModeEnd {
        instance_id: u64,
        success: bool,
        keystone_level: u64,
        total_time: u64,
    },
    NoneSentinel,
}

impl Special {
    pub fn parse(event_type: &str, line: &[&str]) -> Result<Self> {
        let matched = match event_type {
            "ENCHANT_APPLIED" => Self::EnchantApplied {
                source: Actor::parse(&line[0..4])?,
                target: Actor::parse(&line[4..8])?,
                spell_name: line[8].to_string(),
                item_id: parse_num(line[9])?,
                item_name: line[10].to_string(),
            },

            "ENCHANT_REMOVED" => Self::EnchantRemoved {
                source: Actor::parse(&line[0..4])?,
                target: Actor::parse(&line[4..8])?,
                spell_name: line[8].to_string(),
                item_id: parse_num(line[9])?,
                item_name: line[10].to_string(),
            },

            "PARTY_KILL" => Self::PartyKill {
                source: Actor::parse(&line[0..4])?,
                target: Actor::parse(&line[4..8])?,
                unconscious_on_death: parse_bool(line[8])?,
            },

            "UNIT_DIED" => Self::UnitDied {
                source: Actor::parse(&line[0..4])?,
                target: Actor::parse(&line[4..8])?,
                unconscious_on_death: parse_bool(line[8])?,
            },

            "UNIT_DESTROYED" => Self::UnitDestroyed {
                source: Actor::parse(&line[0..4])?,
                target: Actor::parse(&line[4..8])?,
                unconscious_on_death: parse_bool(line[8])?,
            },

            "UNIT_DISSIPATES" => Self::UnitDissipates {
                source: Actor::parse(&line[0..4])?,
                target: Actor::parse(&line[4..8])?,
                unconscious_on_death: parse_bool(line[8])?,
            },

            "COMBAT_LOG_VERSION" => Self::CombatLogInfo {
                log_version: parse_num(line[0])?,
                advanced_log_enabled: parse_bool(line[2])?,
                build_version: line[4].to_string(),
                project_id: parse_num(line[6])?,
            },

            "ZONE_CHANGE" => Self::ZoneChange {
                instance_id: parse_num(line[0])?,
                zone_name: line[1].to_string(),
                id: parse_num(line[2])?,
            },

            "MAP_CHANGE" => Self::MapChange {
                ui_map_id: parse_num(line[0])?,
                ui_map_name: line[1].to_string(),
                x0: parse_num(line[2])?,
                x1: parse_num(line[3])?,
                y0: parse_num(line[4])?,
                y1: parse_num(line[5])?,
            },

            "ENCOUNTER_START" => Self::EncounterStart {
                encounter_id: parse_num(line[0])?,
                encounter_name: line[1].to_string(),
                difficulty_id: parse_num(line[2])?,
                group_size: parse_num(line[3])?,
                instance_id: parse_num(line[4])?,
            },
            "ENCOUNTER_END" => Self::EncounterEnd {
                encounter_id: parse_num(line[0])?,
                encounter_name: line[1].to_string(),
                difficulty_id: parse_num(line[2])?,
                group_size: parse_num(line[3])?,
                success: parse_bool(line[4])?,
                fight_time: parse_num(line[5])?,
            },
            "WORLD_MARKER_PLACED" => Self::WorldMarkerPlaced {
                instance_id: parse_num(line[0])?,
                marker: parse_num(line[1])?,
                x: parse_num(line[2])?,
                y: parse_num(line[3])?,
            },
            "WORLD_MARKER_REMOVED" => Self::WorldMarkerRemoved {
                marker: parse_num(line[0])?,
            },
            "EMOTE" => {
                match GUID::parse(line[2]) {
                    Ok(g) => Self::EmoteEnvironmental {
                        source_guid: GUID::parse(line[0])?,
                        source_name: line[1].to_string(),
                        target_guid: g,
                        target_name: line[3].to_string(),
                        text: line[4].to_string(),
                    },
                    Err(_) => Self::EmoteStandard {
                        actor: Actor::parse(&line[..4])?,
                        text: line[4].to_string(),
                    }
                }
            }
            "COMBATANT_INFO" => Self::CombatantInfo(combatant::CombatantInfo::parse(line)?),
            "CHALLENGE_MODE_START" => Self::ChallengeModeStart {
                zone_name: line[0].to_string(),
                instance_id: parse_num(line[1])?,
                challenge_mode_id: parse_num(line[2])?,
                keystone_level: parse_num(line[3])?,
                affix_ids: {
                    let joined = line[4..].join(",");

                    joined[1..joined.len() - 1]
                        .split(',')
                        .map(parse_num)
                        .collect::<Result<Vec<u64>>>()?
                },
            },
            "CHALLENGE_MODE_END" => Self::ChallengeModeEnd {
                instance_id: parse_num(line[0])?,
                success: parse_bool(line[1])?,
                keystone_level: parse_num(line[2])?,
                total_time: parse_num(line[3])?,
            },

            _ => Self::NoneSentinel
        };

        Ok(matched)
    }
}


#[cfg(test)]
mod tests {
    use super::Special;

    #[test]
    fn parse() {
        let event_type = "ENCHANT_APPLIED";
        let line = vec!["0000000000000000", "nil", "0x80000000", "0x80000000", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "Howling Rune", "207782", "Sickle of the White Stag"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "ENCHANT_REMOVED";
        let line = vec!["0000000000000000", "nil", "0x80000000", "0x80000000", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "Howling Rune", "207782", "Sickle of the White Stag"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "PARTY_KILL";
        let line = vec!["0000000000000000", "nil", "0x80000000", "0x80000000", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "0"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "UNIT_DIED";
        let line = vec!["0000000000000000", "nil", "0x80000000", "0x80000000", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "0"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "UNIT_DESTROYED";
        let line = vec!["0000000000000000", "nil", "0x80000000", "0x80000000", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "0"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "UNIT_DISSIPATES";
        let line = vec!["0000000000000000", "nil", "0x80000000", "0x80000000", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "0"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "COMBAT_LOG_VERSION";
        let line = vec!["20", "ADVANCED_LOG_ENABLED", "1", "BUILD_VERSION", "10.2.6", "PROJECT_ID", "1"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "ZONE_CHANGE";
        let line = vec!["2549", "Amirdrassil, the Dream's Hope", "14"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "MAP_CHANGE";
        let line = vec!["2232", "Amirdrassil", "3800.000000", "3000.000000", "13725.000000", "12525.000000"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "ENCOUNTER_START";
        let line = vec!["2820", "Gnarlroot", "14", "19", "2549"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "ENCOUNTER_END";
        let line = vec!["2820", "Gnarlroot", "14", "19", "1", "162742"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "WORLD_MARKER_PLACED";
        let line = vec!["2549", "7", "4010.06", "13115.27"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "WORLD_MARKER_REMOVED";
        let line = vec!["7"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "EMOTE";
        let line = vec!["Creature-0-4233-2549-14868-200927-00004E8C97", "Smolderon", "0000000000000000", "nil", r"|TInterface\Icons\SPELL_FIRE_RAGNAROS_MOLTENINFERNO.BLP:20|tEmberscar attempts to |cFFFF0000|Hspell:422277|h[Devour Your Essence]|h|r!"];
        let parsed = Special::parse(event_type, &line);
        println!("{:?}", parsed);
    }
}