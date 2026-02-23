use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "NetsimCLI")]
#[command(author = "Раев Андрей Сергеевич")]
#[command(version = "0.1")]
pub struct Cli {
    #[arg(short, long)]
    pub data: i8,
}
