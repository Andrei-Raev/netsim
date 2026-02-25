use clap::Parser;

/// Параметры командной строки для запуска CLI.
#[derive(Parser)]
#[command(name = "NetsimCLI")]
#[command(author = "Раев Андрей Сергеевич")]
#[command(version = "0.1")]
pub struct Cli {
    /// Путь к файлу сценария (*.scenario.toml).
    #[arg(short, long)]
    pub scenario: Option<String>,
}
