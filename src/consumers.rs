use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use itertools::Itertools;

use crate::components::common::Actor;
use crate::components::events::{Event, EventType};
use crate::components::guid::GUID;
use crate::components::special;
use crate::components::suffixes::Suffix;

pub trait EventHandler {
    fn handle(&mut self, event: &Result<Event>);

    fn display(&self) -> Option<String>;
}


/// Logs out successfully & failed parsed events to stdout & stderr.
pub struct StdLogger;

impl StdLogger {
    pub fn new() -> Self { Self {} }
}

impl EventHandler for StdLogger {
    fn handle(&mut self, event: &Result<Event>) {
        match event {
            Ok(x) => println!("{:?}", x),
            Err(x) => eprintln!("{}", x)
        }
    }

    fn display(&self) -> Option<String> {
        None
    }
}

pub struct FileLogger {
    good_file: File,
    bad_file: File,
}

/// Logs out successfully & failed parsed events to files.
impl FileLogger {
    pub(crate) fn new(good_path: &PathBuf, error_path: &PathBuf) -> Result<Self> {
        Ok(Self {
            good_file: File::options().create(true).append(true).open(good_path)
                .with_context(|| format!("Failed to open file: {:?}", good_path))?,
            bad_file: File::options().create(true).append(true).open(error_path)
                .with_context(|| format!("Failed to open file: {:?}", error_path))?,
        })
    }
}

impl EventHandler for FileLogger {
    fn handle(&mut self, event: &Result<Event>) {
        match event {
            Ok(x) => {
                let _ = self.good_file.write(format!("{:?}\n", x).as_bytes());
            },
            Err(x) => {
                let _ = self.bad_file.write(format!("{:?}\n", x).as_bytes());
            }
        };
    }

    fn display(&self) -> Option<String> {
        None
    }
}

/// A simple damage tracker
#[derive(Debug)]
pub struct DamageTracker {
    accumulated: HashMap<String, i64>,
    start_time: Option<NaiveDateTime>,
    latest_time: Option<NaiveDateTime>,
}

impl DamageTracker {
    pub(crate) fn new() -> Self {
        Self { accumulated: HashMap::new(), start_time: None, latest_time: None }
    }

    fn reset(&mut self) {
        self.accumulated.clear();
        self.start_time = None;
        self.latest_time = None;
    }
}


impl EventHandler for DamageTracker {
    fn handle(&mut self, event: &Result<Event>) {
        match event {
            Ok(Event {
                   timestamp: time,
                   event_type: EventType::Standard {
                       source: Some(Actor { name, guid: GUID::Player { .. }, .. }),
                       suffix: Suffix::Damage { amount: dmg, .. },
                       ..
                   },
                   ..
               }) => {
                if self.accumulated.is_empty() { self.start_time = Some(*time) }
                self.latest_time = Some(*time);

                if let Some(total) = self.accumulated.get_mut(name) {
                    *total += dmg;
                } else {
                    self.accumulated.insert(name.clone(), *dmg);
                }
            }

            // Reset on encounter start
            Ok(Event {
                   event_type: EventType::Special {
                       details: special::Special::EncounterStart { .. }, ..
                   }, ..
               }) => {
                self.reset();
            }
            _ => {}
        }
    }

    fn display(&self) -> Option<String> {
        let duration = if let (Some(start), Some(end)) = (self.start_time, self.latest_time) {
            (end - start).num_seconds() + 1
        } else { 1 };

        let s = self.accumulated.iter()
            .sorted_by_key(|(_, &v)| v).rev()
            .map(|(k, v)| format!("{:>30}:{:>10}|{:>10.0}{:>10}", k, v, (*v as f64) / (duration as f64), "💯"))
            .join("\n");

        Some(format!("8=================D~~~~~{:~>0}~{:~>10}~{:~>10}~{:~>10}\n{}", "Player", "Damage", "DPS", "Parse", s))
    }
}