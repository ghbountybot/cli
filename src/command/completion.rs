use crate::Cli;
use clap::{CommandFactory, ValueEnum};
use clap_complete::Generator;
use eyre::Result;
use owo_colors::OwoColorize;
use std::fs;

pub fn handle(shell: impl ValueEnum + Generator) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    if shell.to_possible_value().unwrap().get_name() == "fish" {
        // Create fish completions directory if it doesn't exist
        let fish_dir = dirs::home_dir()
            .ok_or_else(|| eyre::eyre!("Could not determine home directory"))?
            .join(".config")
            .join("fish")
            .join("completions");

        fs::create_dir_all(&fish_dir)?;

        let completion_path = fish_dir.join(format!("{name}.fish"));
        let mut file = fs::File::create(&completion_path)?;

        clap_complete::generate(shell, &mut cmd, name, &mut file);

        println!("  {}", completion_path.display().to_string().dimmed());
    } else {
        // For other shells, just print to stdout
        clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
    }

    Ok(())
}
