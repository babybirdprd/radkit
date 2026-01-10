use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
pub mod rewriter;
mod utils;
mod tui;

use cli::{Cli, Commands, ProviderCommands, SkillCommands, ToolCommands};
use commands::{create, provider, run, skill, tool};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ui => {
            let action = tui::run()?;
            match action {
                tui::TuiAction::CreateAgent { name, template, provider } => {
                    create::create_agent(Some(name), Some(template), Some(provider), None)?;
                },
                tui::TuiAction::AddTool { name, template } => {
                     // We need to implement a way to pass the template choice to `add_tool`.
                     // Currently `add_tool` uses `Select` interactively.
                     // I will update `commands::tool::add_tool` to take an optional template name/index
                     // or creates a new function `add_tool_with_template`.
                     // For now, I'll modify `add_tool` to check if arguments are passed?
                     // No, `add_tool` signature is `fn add_tool(name: String)`.
                     // I should overload it or change it.

                     // I'll call a new function I will add: `tool::add_tool_non_interactive`.
                     tool::add_tool_with_template(name, template)?;
                },
                tui::TuiAction::None => {}
            }
        },
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
            ProviderCommands::List => {
                provider::list_providers()?;
            }
            ProviderCommands::Remove => {
                provider::remove_provider()?;
            }
        },
    }

    Ok(())
}
