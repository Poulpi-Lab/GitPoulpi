//! `gitpoulpi config get|set|path`: prints what
//! `gitpoulpi_commands::config_cmd::execute` computes, with the CLI's own
//! `--backend` as the top layer.

use gitpoulpi_commands::cli::ConfigAction;
use gitpoulpi_config::Overrides;
use serde_json::json;

pub fn run(action: ConfigAction, overrides: &Overrides, json_output: bool) {
    match gitpoulpi_commands::config_cmd::execute(action, overrides) {
        Ok(emitted) => {
            if json_output {
                println!("{}", emitted.json);
            } else {
                println!("{}", emitted.human);
            }
        }
        Err(message) => {
            if json_output {
                println!("{}", json!({ "error": message }));
            } else {
                eprintln!("gitpoulpi: {message}");
            }
            std::process::exit(1);
        }
    }
}
