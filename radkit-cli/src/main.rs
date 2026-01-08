use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod utils;

use cli::{Cli, Commands, ToolCommands, SkillCommands, ProviderCommands};
use commands::{create, run, tool, skill, provider};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            name,
            template,
            provider,
            path,
        } => {
            create::create_agent(name, template, provider, path)?;
        }
        Commands::List => {
            create::list_templates()?;
        }
        Commands::Run { args } => {
            run::run_agent(args)?;
        }
        Commands::Build { args } => {
            run::build_agent(args)?;
        }
        Commands::Tool { command } => match command {
            ToolCommands::Add { name } => {
                tool::add_tool(name)?;
            }
        },
        Commands::Skill { command } => match command {
            SkillCommands::Add { name } => {
                skill::add_skill(name)?;
            }
        },
        Commands::Provider { command } => match command {
            ProviderCommands::Add { name } => {
                provider::add_provider(name)?;
            }
        },
    }

    Ok(())
}
