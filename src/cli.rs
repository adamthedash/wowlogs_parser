use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, subcommand_value_name = "OUTPUT_MODE", subcommand_help_heading = "Output modes")]
pub struct Cli {
    /// Path to wow log file
    pub wowlog_path: PathBuf,

    #[arg(value_enum)]
    pub read_mode: ReadMode,

    /// Output mode
    #[command(subcommand)]
    pub output_mode: OutputMode,

}

#[derive(Debug, ValueEnum, Clone)]
pub enum ReadMode {
    /// Life-processes a file
    Watch,
    /// Process the entire file
    Process,
}

#[derive(Debug, Subcommand)]
pub enum OutputMode {
    /// Prints to stdin / stdout
    Std,

    /// Write to a file
    File {
        /// File to write correctly parsed events to
        good_path: PathBuf,
        /// File to write incorrectly parsed events to
        failed_path: PathBuf,
    },

    /// Do nothing
    None,
}


#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::Cli;

    #[test]
    fn test_help() {
        let args = Cli::parse_from(vec!["wowlogs.exe", "--help"]);
        println!("{:?}", args);
    }

    #[test]
    fn test_process_std() {
        let args = Cli::parse_from(vec!["wowlogs.exe", "logs.txt", "process", "std"]);
        println!("{:?}", args);
    }

    #[test]
    fn test_watch_file() {
        let args = Cli::parse_from(vec!["wowlogs.exe", "logs.txt", "watch", "file", "good.txt", "bad.txt"]);
        println!("{:?}", args);
    }
}