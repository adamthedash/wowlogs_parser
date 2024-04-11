use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use itertools::Itertools;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use crate::cli::{Cli, OutputMode, ReadMode};
use crate::consumers::{DamageTracker, EventHandler, FileLogger, StdLogger};
use crate::parser::EventParser;

mod traits;
mod utils;
mod parser;
mod consumers;
mod components;
mod cli;


/// Parses the entire buffer
fn parse_file<R: Read>(buf_reader: R, handlers: &mut [Box<dyn EventHandler>]) {
    let reader = EventParser::new(buf_reader);

    reader
        .for_each(|e| {
            handlers.iter_mut()
                .for_each(|h| {
                    h.handle(&e);
                });
        });
}

/// Processes an entire file
fn process<P: AsRef<Path> + Debug>(path: P, handlers: &mut [Box<dyn EventHandler>]) -> Result<()> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open file: {:?}", path))?;

    let reader = EventParser::new(file);

    reader
        .for_each(|e| {
            handlers.iter_mut()
                .for_each(|h| {
                    h.handle(&e);
                });
        });

    Ok(())
}


/// Watches a logile and parses them as they stream in
fn watch<P: AsRef<Path>>(path: P, handlers: &mut [Box<dyn EventHandler>]) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    // Get the number of bytes currently in the file - we only want to tail it
    let mut prev_size = File::open(path)?.metadata()?.len();


    for event in rx.iter().filter_map(Result::ok) {
        let mut file = File::open(&event.paths[0])?;
        let new_size = file.metadata()?.len();

        file.seek(SeekFrom::Current(prev_size as i64))?;

        parse_file(BufReader::new(file), handlers);
        println!("{}", handlers.iter().filter_map(|h| h.display()).join("\n---\n"));

        prev_size = new_size;
    }

    Ok(())
}

fn execute(args: Cli) {
    // Handlers
    let mut handlers: Vec<Box<dyn EventHandler>> = vec![
        Box::new(DamageTracker::new()),
    ];

    // Output mode
    handlers.push(match args.output_mode {
        OutputMode::Std => Box::new(StdLogger::new()),
        OutputMode::File { good_path, failed_path } =>
            Box::new(FileLogger::new(&good_path, &failed_path).unwrap())
    });

    // Inputs
    match args.read_mode {
        ReadMode::Watch => watch(args.wowlog_path, &mut handlers).unwrap(),
        ReadMode::Process => process(args.wowlog_path, &mut handlers).unwrap(),
    }
}


fn main() {
    let args = Cli::parse();
    execute(args);
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::path::PathBuf;
    use std::str::FromStr;

    use clap::Parser;

    use crate::{execute, parse_file};
    use crate::cli::Cli;
    use crate::consumers::{EventHandler, StdLogger};
    use crate::parser::EventParser;

    #[test]
    fn test1() {
        let wowlog_path = PathBuf::from_str(r"E:\Games\Blizzard\World of Warcraft\_retail_\Logs\WoWCombatLog-040624_135724.txt").unwrap();

        let file = File::open(wowlog_path)
            .expect("Error loading wowlogs file.");

        let mut handlers: Vec<Box<dyn EventHandler>> = vec![
            // Box::new(StdLogger::new()),
            // Box::new(DamageTracker::new()),
        ];

        parse_file(file, &mut handlers);
    }

    #[test]
    fn test2() {
        let wowlog_path = PathBuf::from_str("test_data/WoWCombatLog-021524_201412.txt").unwrap();

        let file = File::open(wowlog_path)
            .expect("Error loading wowlogs file.");

        let mut handlers: Vec<Box<dyn EventHandler>> = vec![
            // Box::new(StdLogger::new()),
            // Box::new(DamageTracker::new()),
        ];

        parse_file(file, &mut handlers);
    }

    #[test]
    fn test3() {
        let file = "2/15 20:14:12.865  COMBAT_LOG_VERSION,20,ADVANCED_LOG_ENABLED,1,BUILD_VERSION,10.2.5,PROJECT_ID,1\n".as_bytes();

        let mut handlers: Vec<Box<dyn EventHandler>> = vec![
            Box::new(StdLogger::new()),
            // Box::new(DamageTracker::new()),
        ];

        parse_file(file, &mut handlers);
    }

    #[test]
    fn test_new_method() {
        let file = "2/15 20:14:12.865  COMBAT_LOG_VERSION,20,ADVANCED_LOG_ENABLED,1,BUILD_VERSION,10.2.5,PROJECT_ID,1\n2/15 20:14:12.865  COMBAT_LOG_VERSION,15,ADVANCED_LOG_ENABLED,1,BUILD_VERSION,10.2.5,PROJECT_ID,1\n".as_bytes();

        for event in EventParser::new(file) {
            println!("{:?}", event.unwrap());
        }
    }


    #[test]
    fn test_real() {
        let args = Cli::parse_from(["wow.exe", r"E:\Games\Blizzard\World of Warcraft\_retail_\Logs\WoWCombatLog-041124_213746.txt", "process", "file", "good2.txt", "bad2.txt"]);
        println!("{:?}", args);
        execute(args);
    }
}

