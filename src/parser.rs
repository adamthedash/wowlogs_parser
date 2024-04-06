use chrono::prelude::*;
use csv::StringRecord;
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::player_events::{Actor, AdvancedParams};
use crate::prefixes::{EventPrefix, Prefix};
use crate::special::{CombatLogVersion, EncounterEnd, EncounterStart, MapChange, WorldMarkerPlaced, ZoneChange};
use crate::suffixes::{EventSuffix, Suffix};
use crate::traits::FromRecord;

#[derive(Debug)]
pub struct Event {
    pub datetime: NaiveDateTime,
    pub event_type: EventTypes,
}

#[derive(Debug)]
pub struct OtherEvent {
    pub prefix: Prefix,
    pub suffix: Option<Suffix>,
    pub source: Option<Actor>,
    pub target: Option<Actor>,
    pub advanced: Option<AdvancedParams>,
}

#[derive(Debug)]
pub enum EventTypes {
    EncounterStart(EncounterStart),
    EncounterEnd(EncounterEnd),
    MapChange(MapChange),
    ZoneChange(ZoneChange),
    CombatLogVersion(CombatLogVersion),
    WorldMarkerPlaced(WorldMarkerPlaced),
    Other(OtherEvent),
}

pub fn parse_line(line: &StringRecord) -> Result<Event, String> {
    /// https://wowpedia.fandom.com/wiki/COMBAT_LOG_EVENT#Event_Descriptions

    let line = line.iter().collect::<Vec<_>>();
    // println!("{:?}", line);

    // Date & Event type string
    let date_event_type = line[0].splitn(2, "  ").collect_tuple();
    if date_event_type.is_none() {
        return Err(format!("Error splitting date & event type: {}", line[0]));
    }
    let (date, event_type) = date_event_type.unwrap();

    // todo: horrible hacky way of date parsing
    let date = vec!["2024/ ", date].join("");
    let datetime = NaiveDateTime::parse_from_str(date.as_str(), "%Y/%_m/%d %H:%M:%S%.3f")
        .expect("Failed to parse date.");

    // Event type parsing
    let event_type = match event_type {
        // Special events
        "ENCOUNTER_START" => {
            EventTypes::EncounterStart(EncounterStart::parse_record(&line[1..]))
        }
        "ENCOUNTER_END" => {
            EventTypes::EncounterEnd(EncounterEnd::parse_record(&line[1..]))
        }
        "COMBAT_LOG_VERSION" => {
            EventTypes::CombatLogVersion(CombatLogVersion::parse_record(&line[1..]))
        }
        "ZONE_CHANGE" => {
            EventTypes::ZoneChange(ZoneChange::parse_record(&line[1..]))
        }
        "MAP_CHANGE" => {
            EventTypes::MapChange(MapChange::parse_record(&line[1..]))
        }
        "WORLD_MARKER_PLACED" => {
            EventTypes::WorldMarkerPlaced(WorldMarkerPlaced::parse_record(&line[1..]))
        }
        // Combat event
        _ => {
            // Parse prefix & suffix
            let prefix = EventPrefix::parse(event_type);
            if prefix.is_none() {
                return Err(format!("Error parsing prefix: {}", event_type));
            }
            let prefix = prefix.unwrap();

            let suffix = EventSuffix::parse(event_type);
            if suffix.is_none() {
                return Err(format!("Error parsing suffix: {}", event_type));
            }
            let suffix = suffix.unwrap();
            let mut offset = 1;


            // Actors
            let source = Actor::parse_record(&line[offset..offset + 4]);
            offset += 4;

            let target = Actor::parse_record(&line[offset..offset + 4]);
            offset += 4;


            // Spell info on prefix
            let prefixes = match prefix {
                EventPrefix::SWING => Prefix::SWING,
                _ => {
                    let p = Prefix::parse_record(&line[offset..offset + 3], prefix);
                    offset += 3;

                    p
                }
            };

            // Advanced parameters
            let advanced_params = match suffix {
                // todo: Check if covers all
                EventSuffix::DAMAGE |
                EventSuffix::HEAL |
                EventSuffix::CAST_SUCCESS |
                EventSuffix::ENERGIZE |
                EventSuffix::SPLIT => {
                    Some(AdvancedParams::parse_record(&line[offset..offset + 17]))
                }
                _ => None
            };
            if advanced_params.is_some() {
                offset += 17;
            }


            // Suffixes
            let suffixes = Suffix::parse_record(&line[offset..], suffix);

            EventTypes::Other(OtherEvent {
                prefix: prefixes,
                suffix: suffixes,
                source,
                target,
                advanced: advanced_params,
            })
        }
    };

    return Ok(Event {
        datetime,
        event_type,
    });
}


#[cfg(test)]
mod tests {
    use csv::StringRecord;

    use crate::parser::parse_line;

    #[test]
    fn zone_change() {
        let record = vec!["4/6 14:01:52.697  ZONE_CHANGE", "2549", "Amirdrassil, the Dream's Hope", "14"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn map_change() {
        let record = vec!["4/6 13:58:49.517  MAP_CHANGE", "2232", "Amirdrassil", "3800.000000", "3000.000000", "13725.000000", "12525.000000"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn combatant_info() {
        let record = vec!["4/6 14:02:02.856  COMBATANT_INFO", "Player-1587-0F81497D", "1", "1212", "1734", "31231", "10469", "0", "0", "0", "3067", "3067", "3067", "0", "0", "4759", "4759", "4759", "0", "3677", "1576", "1576", "1576", "2621", "256", "[(82710", "103678", "1)", "(82564", "103687", "1)", "(82566", "103690", "1)", "(82567", "103692", "1)", "(82568", "103693", "2)", "(82570", "103695", "1)", "(82572", "103697", "2)", "(82577", "103702", "1)", "(82578", "103703", "2)", "(82579", "103704", "1)", "(82581", "103706", "1)", "(82582", "103708", "1)", "(82583", "103709", "2)", "(82584", "103710", "1)", "(82586", "103712", "1)", "(82587", "103713", "1)", "(82588", "103714", "1)", "(82590", "103718", "1)", "(82591", "103720", "1)", "(82592", "103721", "1)", "(82593", "103722", "1)", "(82594", "103723", "1)", "(82595", "103724", "1)", "(82596", "103725", "1)", "(82598", "103727", "1)", "(82601", "103731", "1)", "(82602", "103732", "1)", "(82677", "103823", "2)", "(82678", "103825", "1)", "(82681", "103829", "2)", "(82682", "103830", "1)", "(82683", "103832", "1)", "(82684", "103833", "2)", "(82685", "103835", "1)", "(82690", "103840", "1)", "(82694", "103844", "1)", "(82696", "103846", "1)", "(82697", "103847", "2)", "(82699", "103849", "1)", "(82701", "103851", "1)", "(82703", "103853", "1)", "(82705", "103855", "1)", "(82707", "103858", "1)", "(82711", "103863", "1)", "(82712", "103864", "1)", "(82714", "103866", "1)", "(82715", "103867", "1)", "(82716", "103868", "1)", "(82720", "103872", "1)", "(82721", "103873", "1)", "(82691", "114735", "1)", "(82674", "115883", "1)", "(82676", "115884", "1)", "(82713", "103865", "1)", "(82717", "103869", "1)]", "(0", "0", "0", "0)", "[(207281", "467", "()", "(6652", "9599", "7979", "9513", "9564", "1498", "8767)", "())", "(207163", "463", "()", "(6652", "8784", "10244", "7979", "9563", "1494", "8767)", "())", "(207117", "467", "()", "(6652", "9508", "7980", "9568", "1498", "8767)", "())", "(0", "0", "()", "()", "())", "(207121", "460", "()", "(6652", "9505", "7979", "9562", "1491", "8767)", "())", "(109824", "476", "()", "(9639", "6652", "9599", "9506", "9144", "9571", "9875", "8767)", "())", "(207280", "463", "()", "(6652", "9512", "9563", "9639", "1494", "8767)", "())", "(207122", "457", "()", "(6652", "9505", "7979", "9561", "1488", "8767)", "())", "(209907", "431", "()", "(9537", "6652", "9599", "1488", "8766)", "())", "(159272", "441", "()", "(9552", "9636", "6652", "9506", "9144", "3311", "8767)", "())", "(193804", "428", "()", "(9536", "9636", "6652", "9600", "1650", "8766)", "())", "(153928", "424", "()", "(3303", "8767)", "())", "(137306", "457", "()", "(9561", "9639", "6652", "9144", "9461", "8767)", "())", "(207171", "470", "()", "(6652", "7979", "9569", "1501", "8767)", "())", "(158375", "450", "()", "(9555", "9639", "6652", "9506", "9144", "3320", "8767)", "())", "(208321", "447", "()", "(9554", "9639", "6652", "9147", "9529", "1517", "8767)", "())", "(0", "0", "()", "()", "())", "(0", "0", "()", "()", "())]", "[Player-1587-0F81497D", "373456", "Player-1400-0482DEDD", "389684", "Player-1400-0482DEDD", "389685", "Player-1587-0F81497D", "21562", "Player-1403-0A5506C6", "381753", "Player-1329-09AF0ACF", "1126", "Player-3682-0B4DD6DD", "6673]", "1", "0", "0", "0"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn encounter_start() {
        let record = vec!["4/6 14:02:02.856  ENCOUNTER_START", "2820", "Gnarlroot", "14", "19", "2549"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn encounter_end() {
        let record = vec!["4/6 14:04:45.580  ENCOUNTER_END", "2820", "Gnarlroot", "14", "19", "1", "162742"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn combat_log_version() {
        let record = vec!["4/6 13:57:24.313  COMBAT_LOG_VERSION", "20", "ADVANCED_LOG_ENABLED", "1", "BUILD_VERSION", "10.2.6", "PROJECT_ID", "1"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record);

        let record = vec!["COMBAT_LOG_VERSION", "20", "ADVANCED_LOG_ENABLED", "1", "BUILD_VERSION", "10.2.6", "PROJECT_ID", "1"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_periodic_heal() {
        let record = vec!["4/6 13:57:24.741  SPELL_PERIODIC_HEAL", "Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "Creature-0-1469-2549-12530-210177-000011428F", "Tormented Ancient", "0xa18", "0x0", "8936", "Regrowth", "0x8", "Creature-0-1469-2549-12530-210177-000011428F", "0000000000000000", "5927873", "7468728", "0", "0", "5043", "0", "1", "0", "0", "0", "3295.44", "13209.11", "2232", "3.4506", "72", "2557", "2557", "0", "0", "nil"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_aura_applied() {
        let record = vec!["4/6 13:57:25.222  SPELL_AURA_APPLIED", "Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "768", "Cat Form", "0x1", "BUFF"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_cast_success() {
        let record = vec!["4/6 13:57:25.222  SPELL_CAST_SUCCESS", "Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "0000000000000000", "nil", "0x80000000", "0x80000000", "768", "Cat Form", "0x1", "Player-1393-077C088C", "0000000000000000", "696560", "696560", "14262", "2190", "4869", "0", "3", "160", "160", "0", "3316.10", "13199.07", "2232", "5.3044", "470"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_aura_removed() {
        let record = vec!["4/6 13:57:27.495  SPELL_AURA_REMOVED", "0000000000000000", "nil", "0x511", "0x0", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "421639", "Burning Heat", "0x4", "DEBUFF"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_aura_refresh() {
        let record = vec!["4/6 13:57:32.547  SPELL_AURA_REFRESH", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "425479", "Dream's Blessing", "0x8", "DEBUFF"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_cast_start() {
        let record = vec!["4/6 13:59:25.117  SPELL_CAST_START", "Player-1084-0AA1EF84", "Ohlga-TarrenMill", "0x514", "0x0", "0000000000000000", "nil", "0x80000000", "0x80000000", "556", "Astral Recall", "0x8"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_heal() {
        let record = vec!["4/6 13:59:50.642  SPELL_HEAL", "Player-1316-0CC289C3", "Ferrarello-Nemesis", "0x514", "0x0", "Player-1316-0CC289C3", "Ferrarello-Nemesis", "0x514", "0x0", "73685", "Unleash Life", "0x8", "Player-1316-0CC289C3", "0000000000000000", "700790", "700790", "3937", "13311", "6359", "0", "0", "250000", "250000", "0", "3411.52", "13122.82", "2232", "6.1016", "464", "132235", "132235", "132235", "0", "1"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_energise() {
        let record = vec!["4/6 13:59:50.643  SPELL_ENERGIZE", "Player-1316-0CC289C3", "Ferrarello-Nemesis", "0x514", "0x0", "Player-1316-0CC289C3", "Ferrarello-Nemesis", "0x514", "0x0", "101033", "Resurgence", "0x8", "Player-1316-0CC289C3", "0000000000000000", "700790", "700790", "3937", "13311", "6359", "0", "0", "249202", "250000", "0", "3411.52", "13122.82", "2232", "6.1016", "464", "1200.0000", "0.0000", "0", "250000"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_summon() {
        let record = vec!["4/6 14:02:02.773  SPELL_SUMMON", "Player-1390-0B058D1A", "Nistî-Hyjal", "0x40514", "0x20", "Creature-0-1469-2549-12530-27893-00001147CA", "Unknown", "0xa28", "0x0", "377671", "Everlasting Bond", "0x1"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn swing_damage() {
        let record = vec!["4/6 14:02:02.797  SWING_DAMAGE", "Creature-0-1469-2549-12530-27893-00009147CA", "Unknown", "0x2114", "0x0", "Creature-0-1469-2549-12530-209333-000011428A", "Gnarlroot", "0xa48", "0x0", "Creature-0-1469-2549-12530-27893-00009147CA", "Player-1390-0B058D1A", "675350", "675350", "5761", "0", "23324", "0", "1", "0", "0", "0", "3475.76", "13116.73", "2232", "6.2641", "476", "3575", "5106", "-1", "1", "0", "0", "0", "nil", "nil", "nil"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_absorbed() {
        let record = vec!["4/6 14:02:03.252  SPELL_ABSORBED", "Player-1390-0B058D1A", "Nistî-Hyjal", "0x40514", "0x20", "Player-1390-0B058D1A", "Nistî-Hyjal", "0x40514", "0x20", "425461", "Tainted Heart", "0x24", "Player-1587-0F81497D", "Huisarts-Arathor", "0x514", "0x0", "17", "Power Word: Shield", "0x2", "1629", "1910", "nil"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_missed() {
        let record = vec!["4/6 14:02:03.253  SPELL_MISSED", "Player-1390-0B058D1A", "Nistî-Hyjal", "0x40514", "0x20", "Player-1390-0B058D1A", "Nistî-Hyjal", "0x40514", "0x20", "425461", "Tainted Heart", "0x24", "ABSORB", "nil", "1629", "1910", "nil"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_damage() {
        let record = vec!["4/6 14:02:03.255  SPELL_DAMAGE", "Player-1390-0B058D1A", "Nistî-Hyjal", "0x40514", "0x20", "Creature-0-1469-2549-12530-209333-000011428A", "Gnarlroot", "0xa48", "0x0", "425461", "Tainted Heart", "0x24", "Creature-0-1469-2549-12530-209333-000011428A", "0000000000000000", "268607404", "268624895", "0", "0", "5043", "0", "3", "0", "100", "0", "3475.30", "13117.91", "2232", "2.7460", "73", "3820", "1910", "-1", "36", "0", "0", "0", "1", "nil", "nil"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn swing_missed() {
        let record = vec!["4/6 14:02:03.352  SWING_MISSED", "Player-1335-0A264B4C", "Sønike-Ysondre", "0x514", "0x0", "Creature-0-1469-2549-12530-209333-000011428A", "Gnarlroot", "0xa48", "0x0", "MISS", "nil"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_periodic_energize() {
        let record = vec!["4/6 14:02:03.700  SPELL_PERIODIC_ENERGIZE", "Player-1390-0B058D1A", "Nistî-Hyjal", "0x40514", "0x20", "Player-1390-0B058D1A", "Nistî-Hyjal", "0x40514", "0x20", "47568", "Empower Rune Weapon", "0x1", "Player-1390-0B058D1A", "0000000000000000", "1755909", "1789354", "17300", "1186", "18659", "174164", "6", "50", "1250", "0", "3465.71", "13122.42", "2232", "5.9743", "476", "5.0000", "0.0000", "6", "1250"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_cast_failed() {
        let record = vec!["4/6 14:02:04.424  SPELL_CAST_FAILED", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "0000000000000000", "nil", "0x80000000", "0x80000000", "8921", "Moonfire", "0x40", "Not yet recovered"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_periodic_damage() {
        let record = vec!["4/6 14:02:05.129  SPELL_PERIODIC_DAMAGE", "Player-3692-0A2A043C", "Hypersus-Eredar", "0x514", "0x0", "Creature-0-1469-2549-12530-209333-000011428A", "Gnarlroot", "0x10a48", "0x0", "12654", "Ignite", "0x4", "Creature-0-1469-2549-12530-209333-000011428A", "0000000000000000", "267910173", "268624895", "0", "0", "5043", "0", "3", "2", "100", "0", "3475.30", "13117.91", "2232", "2.7460", "73", "6374", "6374", "-1", "4", "0", "0", "0", "nil", "nil", "nil"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_empower_start() {
        let record = vec!["4/6 14:02:05.375  SPELL_EMPOWER_START", "Player-1403-0A5506C6", "Smoczyslawa-Draenor", "0x512", "0x0", "Creature-0-1469-2549-12530-209333-000011428A", "Gnarlroot", "0x10a48", "0x0", "359073", "Eternity Surge", "0x50"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_empower_end() {
        let record = vec!["4/6 14:02:06.333  SPELL_EMPOWER_END", "Player-1403-0A5506C6", "Smoczyslawa-Draenor", "0x512", "0x0", "0000000000000000", "nil", "0x80000000", "0x80000000", "359073", "Eternity Surge", "0x50", "1"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_periodic_missed() {
        let record = vec!["4/6 14:02:09.676  SPELL_PERIODIC_MISSED", "Creature-0-1469-2549-12530-209333-000011428A", "Gnarlroot", "0x10a48", "0x0", "Player-1390-0B058D1A", "Nistî-Hyjal", "0x40514", "0x20", "422026", "Tortured Scream", "0x24", "ABSORB", "nil", "20403", "23920", "nil"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_drain() {
        let record = vec!["4/6 14:02:15.736  SPELL_DRAIN", "Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "22568", "Ferocious Bite", "0x1", "Player-1393-077C088C", "0000000000000000", "718494", "731380", "14975", "2190", "4869", "4658", "3", "160", "160", "0", "3469.27", "13114.87", "2232", "0.4522", "470", "25", "3", "0", "160"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn unit_died() {
        let record = vec!["4/6 14:02:28.373  UNIT_DIED", "0000000000000000", "nil", "0x80000000", "0x80000000", "Creature-0-1469-2549-12530-78001-00001147D1", "Cloudburst Totem", "0x2114", "0x0", "0"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn party_kill() {
        let record = vec!["4/6 14:02:34.769  PARTY_KILL", "Player-1403-0A5506C6", "Smoczyslawa-Draenor", "0x512", "0x0", "Creature-0-1469-2549-12530-210231-00011147DE", "Tainted Lasher", "0xa48", "0x0", "0"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_empower_interrupt() {
        let record = vec!["4/6 14:03:02.965  SPELL_EMPOWER_INTERRUPT", "Player-1396-0AD1A529", "Scalynfatal-AzjolNerub", "0x514", "0x0", "0000000000000000", "nil", "0x80000000", "0x80000000", "382266", "Fire Breath", "0x4", "0"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn unit_destroyed() {
        let record = vec!["4/6 14:03:04.036  UNIT_DESTROYED", "0000000000000000", "nil", "0x80000000", "0x80000000", "Creature-0-1469-2549-12530-26125-00001147CB", "Risen Ghoul", "0x2114", "0x0", "0"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }


    #[test]
    fn spell_aura_broken_spell() {
        let record = vec!["4/6 14:06:31.727  SPELL_AURA_BROKEN_SPELL", "Player-1335-0A264B4C", "Sønike-Ysondre", "0x514", "0x0", "Player-1335-0A264B4C", "Sønike-Ysondre", "0x514", "0x0", "115191", "Stealth", "0x1", "360194", "Deathmark", "1", "DEBUFF"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn spell_resurrect() {
        let record = vec!["4/6 14:09:35.703  SPELL_RESURRECT", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "Player-1335-0A264B4C", "Sønike-Ysondre", "0x514", "0x0", "20484", "Rebirth", "0x8"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }

    #[test]
    fn enchant_removed() {
        let record = vec!["4/6 14:10:40.183  ENCHANT_REMOVED", "0000000000000000", "nil", "0x80000000", "0x80000000", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "Howling Rune", "207782", "Sickle of the White Stag"];
        let record = StringRecord::from(record);

        let parsed = parse_line(&record);
        assert!(parsed.is_ok(), "Error parsing line: {:?} {:?}", parsed.err(), record)
    }
}