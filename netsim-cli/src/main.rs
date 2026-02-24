use clap::Parser;

mod app_config;
mod argparser;

fn main() {
    let _cli = argparser::Cli::parse();
    let cfg = app_config::SystemConfig::new().expect("failed to load config");

    println!(
        "Netsim CLI: window={}x{}, debug={}",
        cfg.window.width_px, cfg.window.height_px, cfg.debug
    );
}
