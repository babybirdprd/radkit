use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod utils;

use cli::{Cli, Commands, ToolCommands, SkillCommands, ProviderCommands};
use commands::{create, run, tool, skill, provider, check};

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
        Commands::Check => {
            check::check_environment()?;
        }
        Commands::Tool { command } => match command {
            ToolCommands::Add { name } => {
                tool::add_tool(name)?;
            }
            ToolCommands::List => {
                tool::list_tools()?;
            }
            ToolCommands::Remove { name } => {
                tool::remove_tool(name)?;
            }
        },
        Commands::Skill { command } => match command {
            SkillCommands::Add { name } => {
                skill::add_skill(name)?;
            }
            SkillCommands::List => {
                skill::list_skills()?;
            }
            SkillCommands::Remove { name } => {
                skill::remove_skill(name)?;
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
