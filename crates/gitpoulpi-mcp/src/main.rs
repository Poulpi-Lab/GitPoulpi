//! `gitpoulpi-mcp`: see `gitpoulpi_mcp::standalone`.

fn main() -> std::process::ExitCode {
    gitpoulpi_mcp::standalone::main(std::env::args().skip(1).collect())
}
