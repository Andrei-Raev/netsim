use netsim_core;
use clap::Parser;

mod argparser;
mod app_config;

fn main() {
    let cli = argparser::Cli::parse();
    println!("Hello, world!");
    println!("{}", netsim_core::add(1, cli.data as u64));
    let cfg = app_config::SystemConfig::new();
    print!("{}", cfg.unwrap().debug)
}
