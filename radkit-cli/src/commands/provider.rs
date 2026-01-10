use crate::rewriter::Rewriter;
use anyhow::Result;
use colored::*;
use console::style;
use std::fs;

fn get_provider_info(name: &str) -> Result<(&'static str, &'static str)> {
    match name.to_lowercase().as_str() {
        "gemini" => Ok(("GeminiLlm", "GEMINI_API_KEY")),
        "openai" => Ok(("OpenAILlm", "OPENAI_API_KEY")),
        "anthropic" => Ok(("AnthropicLlm", "ANTHROPIC_API_KEY")),
        "deepseek" => Ok(("DeepSeekLlm", "DEEPSEEK_API_KEY")),
        "grok" => Ok(("GrokLlm", "XAI_API_KEY")),
        "openrouter" => Ok(("OpenRouterLlm", "OPENROUTER_API_KEY")),
        _ => anyhow::bail!(
            "Unknown provider '{}'. Supported: gemini, openai, anthropic, deepseek, grok, openrouter",
            name
        ),
    }
}

pub fn add_provider(name: String) -> Result<()> {
    let (provider_struct, env_var) = get_provider_info(&name)?;

    let current_dir = std::env::current_dir()?;
    let main_rs = current_dir.join("src").join("main.rs");

    if !main_rs.exists() {
        anyhow::bail!("src/main.rs not found. Are you in a radkit agent project root?");
    }

    println!("{}", "Configuring Provider".bold().cyan());
    println!("{}", "-----------------------------------".dimmed());
    println!("1. Ensure you have the environment variable set:");
    println!("   export {}=your_key_here", env_var);

    // Automate wiring
    let content = fs::read_to_string(&main_rs)?;
    let mut rewriter = Rewriter::new(&content)?;

    match rewriter.update_provider(provider_struct, env_var) {
        Ok(_) => {
            fs::write(main_rs, rewriter.to_string())?;
            println!(
                "{}",
                format!(
                    "✔ Automatically updated src/main.rs to use {}",
                    provider_struct
                )
                .green()
            );
        }
        Err(e) => {
            println!(
                "{}",
                format!(
                    "Warning: Could not automatically update provider: {}",
                    e
                )
                .yellow()
            );
            println!("\nPlease manually update your `src/main.rs`:");
            println!("   use radkit::models::providers::{};", provider_struct);
            println!(
                "   let llm = {}::from_env(\"model-name\")?;",
                provider_struct
            );
        }
    }

    Ok(())
}

pub fn list_providers() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let main_rs = current_dir.join("src").join("main.rs");

    if !main_rs.exists() {
        println!("No src/main.rs found.");
        return Ok(());
    }

    let content = fs::read_to_string(&main_rs)?;
    let rewriter = Rewriter::new(&content)?;

    if let Some(provider) = rewriter.get_current_provider() {
        println!("Current provider in main.rs: {}", style(provider).cyan());
    } else {
        println!("Could not detect a configured provider in main.rs.");
    }

    Ok(())
}

pub fn remove_provider() -> Result<()> {
    // There isn't an easy "remove" that leaves code compiling without replacing it.
    // We'll just print a message or maybe comment it out?
    // User requested remove command.
    // I'll interpret "remove" as checking if there is one and telling user how to unset,
    // or maybe replacing with a placeholder.

    println!("To remove/change the provider, simply `add` a different one.");
    println!("Example: `radkit provider add openai`");
    println!("There is no specific 'remove' action that leaves the agent in a valid state without a provider.");

    Ok(())
}
