use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use itertools::Itertools;

use crate::components::{
    advanced::AdvancedParams,
    common::Actor,
    prefixes::Prefix,
    special,
    suffixes::Suffix,
};

#[derive(Debug)]
pub enum EventType {
    Special {
        name: String,
        details: special::Special,
    },
    Standard {
        name: String,
        source: Option<Actor>,
        target: Option<Actor>,
        prefix: Prefix,
        advanced_params: Option<AdvancedParams>,
        suffix: Suffix,
    },
}

impl EventType {
    fn parse(event_type: &str, line: &[&str]) -> Result<Self> {
        // Match against any special events
        let special = special::Special::parse(event_type, line)?;
        match special {
            // No match
            special::Special::NoneSentinel => {}
            // Valid match
            s => return Ok(Self::Special {
                name: event_type.to_string(),
                details: s,
            }),
        }


        // match against standard but specially named events
        let specially_named_events = HashMap::from([
            ("DAMAGE_SPLIT", "SPELL_DAMAGE"),
            ("DAMAGE_SHIELD", "SPELL_DAMAGE"),
            ("DAMAGE_SHIELD_MISSED", "SPELL_MISSED"),
            ("SWING_DAMAGE_LANDED_SUPPORT", "SPELL_DAMAGE_SUPPORT"),
        ]);

        let (name, event_type) = match specially_named_events.get(&event_type) {
            None => (event_type, event_type),
            Some(&val) => (event_type, val)
        };

        // Fallback to standard one
        let source = Actor::parse(&line[..4])?;
        let target = Actor::parse(&line[4..8])?;

        let (prefix, advanced, offset) = if name == "ENVIRONMENTAL_DAMAGE" {
            // ENVIRONMENTAL_DAMAGE has spellinfo & advanced params flipped order /facepalm/
            let prefix = Prefix::parse(event_type, &line[25..26])?;
            let advanced = Some(AdvancedParams::parse(&line[8..25])?);

            (prefix, advanced, 26)
        } else {
            let to_consume = match event_type {
                // Special case: ABSORB may or may not contain spell info
                // we have no way to tell without attempting to parse and catching fails
                e if e == "SPELL_ABSORBED" || e == "SPELL_ABSORBED_SUPPORT"
                    && u64::from_str(line[8]).is_err() => 0,
                _ => Prefix::entries_to_consume(event_type)?
            };

            let prefix = Prefix::parse(event_type, &line[8..8 + to_consume])?;
            let mut offset = 8 + to_consume;

            let advanced = if Suffix::has_advanced_params(event_type)? {
                let a = AdvancedParams::parse(&line[offset..offset + 17])?;
                offset += 17;
                Some(a)
            } else {
                None
            };

            (prefix, advanced, offset)
        };


        let suffixes = Suffix::parse(event_type, &line[offset..])?;

        Ok(Self::Standard {
            name: name.to_string(),
            source,
            target,
            prefix,
            advanced_params: advanced,
            suffix: suffixes,
        })
    }
}


#[derive(Debug)]
pub struct Event {
    pub timestamp: NaiveDateTime,
    pub event_type: EventType,
}

impl Event {
    pub(crate) fn parse(line: &[&str]) -> Result<Self> {
        // Split timestamp & event type
        let (timestamp, event_type) = if line[0] == "COMBAT_LOG_VERSION" {
            (
                NaiveDateTime::parse_from_str("2024/01/01 00:00:00.000", "%Y/%_m/%d %H:%M:%S%.3f").unwrap(),
                line[0]
            )
        } else {
            let (date, event_type) = line[0].splitn(2, "  ")
                .collect_tuple()
                .with_context(|| format!("Error splitting date & event type: {}", line[0]))?;

            // todo: horrible hacky way of date parsing
            let date = ["2024/ ", date].join("");
            let datetime = NaiveDateTime::parse_from_str(date.as_str(), "%Y/%_m/%d %H:%M:%S%.3f")
                .with_context(|| "Failed to parse date.")?;

            (datetime, event_type)
        };

        Ok(Self {
            timestamp,
            event_type: EventType::parse(event_type, &line[1..])
                .with_context(|| format!("Error parsing line: {:?}", line))?,
        })
    }
}


#[cfg(test)]
mod tests {
    use crate::components::events::{Event, EventType};

    #[test]
    fn parse_event_type() {
        let event_type = "COMBAT_LOG_VERSION";
        let line = vec!["20", "ADVANCED_LOG_ENABLED", "1", "BUILD_VERSION", "10.2.6", "PROJECT_ID", "1"];
        let parsed = EventType::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_PERIODIC_HEAL";
        let line = vec!["Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "Creature-0-1469-2549-12530-210177-000011428F", "Tormented Ancient", "0xa18", "0x0", "8936", "Regrowth", "0x8", "Creature-0-1469-2549-12530-210177-000011428F", "0000000000000000", "5927873", "7468728", "0", "0", "5043", "0", "1", "0", "0", "0", "3295.44", "13209.11", "2232", "3.4506", "72", "2557", "2557", "0", "0", "nil"];
        let parsed = EventType::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_CAST_SUCCESS";
        let line = vec!["Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "0000000000000000", "nil", "0x80000000", "0x80000000", "1850", "Dash", "0x1", "Player-1329-09AF0ACF", "0000000000000000", "846460", "846460", "16429", "15797", "5313", "94077", "3", "100", "100", "0", "3110.69", "13146.01", "2232", "0.7478", "486"];
        let parsed = EventType::parse(event_type, &line);
        println!("{:?}", parsed);

        let event_type = "SPELL_AURA_REMOVED";
        let line = vec!["Player-1084-0934CD1D", "Neversman-TarrenMill", "0x514", "0x0", "Player-1379-0814BAB7", "Kuro-Zul'jin", "0x40512", "0x4", "6673", "Battle Shout", "0x1", "BUFF"];
        let parsed = EventType::parse(event_type, &line);
        println!("{:?}", parsed);
    }

    #[test]
    fn parse_event() {
        let line = vec!["4/6 14:09:44.867  SPELL_PERIODIC_HEAL", "Player-1393-077C088C", "Mubaku-BronzeDragonflight", "0x514", "0x0", "Creature-0-1469-2549-12530-210177-000011428F", "Tormented Ancient", "0xa18", "0x0", "8936", "Regrowth", "0x8", "Creature-0-1469-2549-12530-210177-000011428F", "0000000000000000", "5927873", "7468728", "0", "0", "5043", "0", "1", "0", "0", "0", "3295.44", "13209.11", "2232", "3.4506", "72", "2557", "2557", "0", "0", "nil"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());

        let line = vec!["COMBAT_LOG_VERSION", "20", "ADVANCED_LOG_ENABLED", "1", "BUILD_VERSION", "10.2.6", "PROJECT_ID", "1"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());

        let line = vec!["4/6 14:09:44.867  COMBAT_LOG_VERSION", "20", "ADVANCED_LOG_ENABLED", "1", "BUILD_VERSION", "10.2.6", "PROJECT_ID", "1"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());

        let line = vec!["4/6 14:02:07.362  SWING_MISSED", "Player-1335-0A264B4C", "Sønike-Ysondre", "0x514", "0x0", "Creature-0-1469-2549-12530-209333-000011428A", "Gnarlroot", "0x10a48", "0x0", "MISS", "1"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_emote_player() {
        let line = vec!["4/11 22:19:57.499  EMOTE", "Creature-0-1465-2444-137-194909-00009853CD", "Feather-Ruffling Duck", "0000000000000000", "nil", "Take control of the Feather Ruffling Duck!"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_emote_env() {
        let line = vec!["4/11 22:47:58.605  EMOTE", "Player-1329-09AF0ACF", "Adamthebash", "Player-1329-09AF0ACF", "Adamthebash", "Turn back! The Emerald Dream is clouding your mind..."];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_env_damage() {
        let line = vec!["4/11 22:42:01.100  ENVIRONMENTAL_DAMAGE", "0000000000000000", "nil", "0x80000000", "0x80000000", "Player-1329-070EBCFC", "Naladrem-Ravencrest", "0x518", "0x0", "Player-1329-070EBCFC", "0000000000000000", "815216", "866544", "14879", "1421", "5217", "0", "17", "109", "120", "0", "-931.46", "2546.12", "2133", "4.8479", "484", "Falling", "51328", "51328", "0", "1", "0", "0", "0", "nil", "nil", "nil"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_bres() {
        let line = vec!["4/11 22:38:54.708  SPELL_CAST_SUCCESS", "Player-1329-09AF0ACF", "Adamthebash-Ravencrest", "0x511", "0x0", "Corpse-0-1465-2454-103-0-000018584E", "Unknown", "0x4228", "0x0", "20484", "Rebirth", "0x8", "Player-1329-09AF0ACF", "0000000000000000", "732698", "846460", "16347", "15718", "5632", "0", "0", "250000", "250000", "5000", "66.53", "3330.43", "2133", "4.7368", "486"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_spell_negative() {
        let line = vec!["4/11 23:46:16.867  SPELL_DAMAGE", "Player-604-0A77B54A", "Sangrenar-Thrall", "0x514", "0x0", "Creature-0-1469-2549-12091-204931-0000186743", "Fyrakk", "0x10a48", "0x0", "203796", "Demon Blades", "0x20", "Creature-0-1469-2549-12091-204931-0000186743", "0000000000000000", "758517319", "770131200", "0", "-2435", "5043", "0", "3", "11", "100", "0", "-2161.04", "7142.32", "2238", "0.5034", "73", "16857", "6079", "-1", "127", "0", "0", "0", "1", "nil", "nil"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_spell_negative2() {
        let line = vec!["4/11 23:52:57.070  SPELL_DAMAGE", "Creature-0-1469-2549-12091-204931-0000186743", "Fyrakk", "0x10a48", "0x0", "Player-1390-0C4E032E", "Stillnixx-Hyjal", "0x514", "0x0", "423720", "Blazing Seed", "0x24", "Player-1390-0C4E032E", "0000000000000000", "306419", "834740", "2104", "22733", "3088", "0", "0", "196960", "250000", "0", "-2159.06", "7174.82", "2238", "4.5667", "481", "-14260", "144372", "-1", "36", "0", "0", "85562", "nil", "nil", "nil"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_combatant_info() {
        let line = vec!["4/11 23:57:17.207  COMBATANT_INFO", "Player-1098-0500B8C6", "1", "12648", "1734", "52761", "1128", "0", "0", "0", "3511", "3511", "3511", "900", "0", "4692", "4692", "4692", "443", "6741", "533", "533", "533", "11302", "251", "[(76034", "96162", "1)", "(76036", "96164", "1)", "(76044", "96172", "1)", "(76046", "96174", "1)", "(76050", "96178", "1)", "(76051", "96179", "2)", "(76052", "96180", "1)", "(76055", "96183", "2)", "(76056", "96184", "1)", "(76058", "96187", "1)", "(76059", "96188", "1)", "(76061", "96190", "1)", "(76063", "96192", "1)", "(76067", "96196", "1)", "(76068", "96197", "2)", "(76070", "96199", "1)", "(76071", "96200", "1)", "(76072", "96201", "1)", "(76073", "96202", "1)", "(76076", "96205", "1)", "(76079", "96208", "2)", "(76080", "96209", "2)", "(76082", "96211", "1)", "(76083", "96212", "1)", "(76084", "96213", "1)", "(76085", "96214", "1)", "(76087", "96216", "1)", "(76089", "96218", "1)", "(76091", "96220", "1)", "(76092", "96221", "1)", "(76097", "96226", "1)", "(76098", "96228", "1)", "(76100", "96230", "1)", "(76103", "96233", "2)", "(76104", "96234", "1)", "(76105", "96235", "1)", "(76106", "96236", "2)", "(76109", "96239", "1)", "(76111", "96241", "1)", "(76112", "96242", "1)", "(76113", "96243", "1)", "(76114", "96244", "1)", "(76115", "96245", "1)", "(76116", "96246", "1)", "(76117", "96247", "1)", "(76118", "96248", "1)", "(76119", "96249", "1)", "(76120", "96251", "1)", "(76121", "96252", "1)", "(76122", "96253", "2)", "(76123", "96254", "2)", "(76081", "96210", "1)", "(76049", "96177", "1)]", "(1", "204080", "199719", "233396)", "[(207200", "489", "(7052", "0", "0)", "(40", "9513", "9639", "9576", "1520", "8767", "9516)", "(192961", "415))", "(137311", "483", "()", "(9639", "6652", "9144", "9477", "8782", "9581", "9876", "8767)", "(192945", "415", "192945", "415", "192945", "415))", "(207198", "489", "()", "(6652", "9511", "9639", "9576", "1520", "8767)", "())", "(0", "0", "()", "()", "())", "(207203", "489", "(6625", "0", "0)", "(6652", "9515", "9639", "9576", "1520", "8767)", "())", "(109841", "489", "()", "(9639", "6652", "9516", "9506", "9144", "9576", "9888", "8767)", "(192919", "415))", "(190523", "486", "(6490", "0", "0)", "(8836", "8840", "8902", "8960)", "())", "(190496", "486", "(6607", "0", "0)", "(8836", "8840", "8902)", "())", "(207150", "483", "(6586", "0", "0)", "(6652", "9516", "9508", "7980", "9581", "1514", "8767)", "(192945", "415))", "(207201", "489", "()", "(6652", "9514", "9639", "9576", "1520", "8767)", "())", "(192999", "486", "(6556", "0", "0)", "(8836", "8840", "8902", "8780)", "(192988", "415))", "(134487", "489", "(6556", "0", "0)", "(9639", "6652", "9144", "9576", "9882", "8767", "9516)", "(192945", "415))", "(207168", "483", "()", "(42", "7980", "9581", "1514", "8767)", "())", "(207566", "483", "()", "(9639", "6652", "9144", "9581", "1534", "8767)", "())", "(207195", "483", "(6604", "0", "0)", "(6652", "9639", "9581", "1514)", "())", "(208193", "483", "(3368", "6518", "0)", "(9524", "9639", "6652", "9147", "9581", "1605", "8767)", "())", "(0", "0", "()", "()", "())", "(210501", "1", "()", "()", "())]", "[Player-1098-0500B8C6", "396092", "Player-1098-0500B8C6", "393438", "Player-1098-0500B8C6", "391571", "Player-1098-0500B8C6", "377073", "Player-1098-0500B8C6", "377098", "Player-1303-0B0DF865", "389684", "Player-1303-0B0DF865", "389685", "Player-1084-086A5186", "1126", "Player-1303-0C124AD2", "6673", "Player-1403-0A82B49D", "21562]", "145", "0", "0", "0"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_spell_dam_support() {
        let line = vec!["2/15 20:32:16.706  SPELL_DAMAGE_SUPPORT", "Player-1329-0A00AB32", "Twigsneak-Ravencrest", "0x514", "0x0", "Creature-0-4233-2549-14868-200927-00004E626C", "Smolderon", "0x10a48", "0x0", "410089", "Prescience", "0x40", "Creature-0-4233-2549-14868-200927-00004E626C", "0000000000000000", "1439613911", "1442829510", "0", "0", "5043", "0", "3", "3", "100", "0", "4043.26", "13109.35", "2233", "2.9862", "73", "163", "73", "-1", "8", "0", "0", "0", "1", "nil", "nil", "Player-1329-09E79FE9"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_swing_dam_support() {
        let line = vec!["2/15 23:32:08.602  SWING_DAMAGE_LANDED_SUPPORT", "Player-1329-0A00AB32", "Twigsneak-Ravencrest", "0x514", "0x0", "Creature-0-4233-2549-14868-200927-00004E8F62", "Smolderon", "0x10a48", "0x0", "410089", "Prescience", "0x40", "Creature-0-4233-2549-14868-200927-00004E8F62", "0000000000000000", "255970276", "1442829510", "0", "0", "5043", "0", "3", "81", "100", "0", "4076.52", "13078.54", "2233", "0.3173", "73", "0", "0", "-1", "1", "0", "0", "0", "1", "nil", "nil", "Player-1329-09E79FE9"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_swing_dam_support_neg() {
        let line = vec!["2/15 23:23:01.449  SWING_DAMAGE_LANDED_SUPPORT", "Creature-0-4233-2549-14868-98035-00004E8EBA", "Dreadstalker", "0x2112", "0x0", "Creature-0-4233-2549-14868-200927-00004E8DDC", "Smolderon", "0x10a48", "0x0", "413984", "Shifting Sands", "0x40", "Creature-0-4233-2549-14868-200927-00004E8DDC", "0000000000000000", "791093865", "1442829510", "0", "0", "5043", "0", "3", "100", "100", "0", "4065.42", "13115.50", "2233", "3.1067", "73", "-908", "-617", "-1", "1", "0", "0", "0", "1", "nil", "nil", "Player-1329-09E79FE9"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_spell_absorbed_support() {
        let line = vec!["2/15 20:33:05.904  SPELL_ABSORBED_SUPPORT", "Creature-0-4233-2549-14868-200927-00004E626C", "Smolderon", "0x10a48", "0x0", "Player-1329-0A0800FA", "Foxgates-Ravencrest", "0x512", "0x0", "422578", "Searing Aftermath", "0x4", "Player-1329-0A0800FA", "Foxgates-Ravencrest", "0x512", "0x0", "413984", "Shifting Sands", "0x40", "1284", "37144", "nil", "Player-1329-09E79FE9"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_spell_absorbed_support2() {
        let line = vec!["1/31 23:32:26.312  SPELL_ABSORBED_SUPPORT", "Creature-0-1467-1501-22700-98542-00003AC9B3", "Amalgam of Souls", "0x10a48", "0x0", "Player-1329-0A17341B", "Oscaruwu-Ravencrest", "0x512", "0x0", "Player-1329-0A17341B", "Oscaruwu-Ravencrest", "0x512", "0x0", "395152", "Ebon Might", "0xc", "7839", "55203", "nil", "Player-1379-0AD1D733"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_challenge_mode_start() {
        let line = vec!["1/31 23:26:12.705  CHALLENGE_MODE_START", "Black Rook Hold", "1501", "199", "18", "[9", "134", "11]"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_challenge_mode_end() {
        let line = vec!["1/31 23:26:12.693  CHALLENGE_MODE_END", "1501", "0", "0", "0", "0.000000", "0.000000"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }

    #[test]
    fn parse_combatant_info2() {
        let line = vec!["3/19 18:44:05.261  COMBATANT_INFO", "Player-1329-09E71507", "1", "918", "1422", "45581", "15599", "0", "0", "0", "5520", "5520", "5520", "250", "0", "3363", "3363", "3363", "325", "1847", "4230", "4230", "4230", "13398", "1467", "[(93271", "115577", "1)", "(93272", "115578", "2)", "(93274", "115580", "2)", "(93275", "115581", "1)", "(93276", "115582", "1)", "(93280", "115587", "1)", "(93281", "115588", "2)", "(93282", "115589", "1)", "(93284", "115592", "1)", "(93285", "115593", "1)", "(93288", "115596", "1)", "(93289", "115597", "1)", "(93295", "115603", "2)", "(93300", "115609", "1)", "(93302", "115611", "2)", "(93303", "115612", "1)", "(93304", "115613", "1)", "(93306", "115615", "1)", "(93307", "115616", "1)", "(93309", "115618", "1)", "(93310", "115619", "2)", "(93311", "115620", "1)", "(93314", "115624", "1)", "(93315", "115625", "1)", "(93316", "115627", "1)", "(93318", "115629", "1)", "(93319", "115631", "1)", "(93321", "115633", "1)", "(93322", "115634", "1)", "(93323", "115635", "1)", "(93324", "115636", "1)", "(93325", "115637", "1)", "(93328", "115640", "1)", "(93330", "115642", "1)", "(93331", "115643", "1)", "(93332", "115644", "1)", "(93333", "115646", "1)", "(93334", "115647", "1)", "(93340", "115654", "1)", "(93341", "115655", "1)", "(93343", "115657", "1)", "(93344", "115658", "1)", "(93345", "115659", "1)", "(93348", "115663", "1)", "(93349", "115664", "2)", "(93350", "115665", "1)", "(93352", "115667", "1)", "(93353", "115668", "1)", "(93354", "115669", "1)", "(93355", "115670", "2)", "(93366", "115683", "1)", "(93715", "116103", "1)", "(93305", "115614", "1)", "(93312", "115621", "1)", "(93320", "115632", "1)]", "(0", "378437", "384660", "378444)", "[(207227", "489", "(7052", "0", "0)", "(6652", "7981", "8095", "9513", "9576", "1520", "8767", "9516)", "(192932", "415))", "(201759", "486", "()", "(8836", "8840", "8902", "9477", "8782)", "(192982", "415", "192932", "415", "192932", "415))", "(207225", "489", "()", "(6652", "7981", "8095", "9511", "9576", "1520", "8767)", "())", "(0", "0", "()", "()", "())", "(193422", "486", "(6625", "0", "0)", "(8836", "8840", "8902)", "())", "(207144", "489", "(6904", "0", "0)", "(6652", "9509", "7981", "9576", "1520", "8767", "9516)", "(192932", "415))", "(207226", "489", "(6830", "0", "0)", "(6652", "9639", "9512", "9576", "1520", "8767)", "())", "(193466", "486", "(6607", "0", "0)", "(8836", "8840", "8902", "8960)", "())", "(204704", "486", "(6574", "0", "0)", "(8836", "8840", "8902", "8960)", "(192932", "415))", "(207228", "489", "()", "(6652", "9639", "9514", "9576", "1520", "8767)", "())", "(193000", "486", "(6556", "0", "0)", "(8836", "8840", "8902", "8780)", "(192932", "415))", "(192999", "486", "(6556", "0", "0)", "(8836", "8840", "8902", "8780)", "(192932", "415))", "(207172", "483", "()", "(6652", "7980", "9581", "1514", "8767)", "())", "(208615", "489", "()", "(6652", "7981", "9576", "1520", "8767)", "())", "(207222", "489", "(6592", "0", "0)", "(6652", "9639", "9576", "1520", "8767)", "())", "(207788", "483", "(6655", "6514", "0)", "(6652", "7980", "9584", "9581", "1514", "8767)", "())", "(158322", "489", "()", "(9639", "6652", "9144", "9576", "9853", "8767)", "())", "(194675", "1", "()", "()", "())]", "[]", "11", "0", "0", "0"];
        let parsed = Event::parse(&line);
        println!("{:?}", parsed.unwrap());
    }
}