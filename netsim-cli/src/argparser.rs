use clap::Parser;

/// Параметры командной строки для запуска CLI.
#[derive(Parser)]
#[command(name = "NetsimCLI")]
#[command(author = "Раев Андрей Сергеевич")]
#[command(version = "0.1")]
pub struct Cli {
    /// Зарезервированный параметр (временный, будет удалён).
    #[arg(short, long, default_value_t = 0)]
    pub data: i8,
}
