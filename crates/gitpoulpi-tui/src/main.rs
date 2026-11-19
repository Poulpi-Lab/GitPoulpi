//! `cargo run -p gitpoulpi-tui`: see `gitpoulpi_tui::standalone`.

fn main() -> std::process::ExitCode {
    gitpoulpi_tui::standalone::main(std::env::args().skip(1).collect())
}
