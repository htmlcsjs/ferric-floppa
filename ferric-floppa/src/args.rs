use std::path::PathBuf;

use clap::Parser;
use tracing::Level;

#[derive(Parser, Debug)]
#[command(author = "htmlcsjs", version, about = "funny CEu discord bot", long_about = None)]
pub struct FlopArgs {
    #[cfg(debug_assertions)]
    #[arg(
        short,
        long,
        help = "Path of run directory to use",
        default_value = "run"
    )]
    pub run: PathBuf,
    #[cfg(not(debug_assertions))]
    #[arg(
        short,
        long,
        help = "Path of run directory to use",
        default_value = "."
    )]
    pub run: PathBuf,
    #[cfg(debug_assertions)]
    #[arg(
        long,
        help = "The level of severity of log outputs",
        default_value_t = Level::DEBUG,
    )]
    pub log_level: Level,
    #[cfg(not(debug_assertions))]
    #[arg(
        long,
        help = "The level of severity of log outputs",
        default_value_t = Level::DEBUG,
    )]
    pub log_level: Level,
}
