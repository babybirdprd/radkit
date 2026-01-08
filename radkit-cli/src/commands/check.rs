use anyhow::{Context, Result};
use console::style;
use std::fs;
use std::path::Path;

pub fn check_environment() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let main_rs = current_dir.join("src").join("main.rs");

    if !main_rs.exists() {
        println!("{}", style("No src/main.rs found. Are you in a radkit agent project?").yellow());
        return Ok(());
    }

    println!("{}", style("Checking Agent Environment").bold());
    println!("{}", style("--------------------------").dim());

    let content = fs::read_to_string(&main_rs).context("Failed to read src/main.rs")?;

    // Check for provider usage
    let providers = vec![
        ("GeminiLlm", "GEMINI_API_KEY"),
        ("OpenAILlm", "OPENAI_API_KEY"),
        ("AnthropicLlm", "ANTHROPIC_API_KEY"),
        ("DeepSeekLlm", "DEEPSEEK_API_KEY"),
        ("GrokLlm", "XAI_API_KEY"),
        ("OpenRouterLlm", "OPENROUTER_API_KEY"),
    ];

    let mut found_provider = false;

    for (struct_name, env_var) in providers {
        if content.contains(struct_name) {
            found_provider = true;
            print!("Detected usage of {}: ", style(struct_name).cyan());

            if std::env::var(env_var).is_ok() {
                println!("{}", style("✔ OK (Environment variable set)").green());
            } else {
                println!("{}", style("✘ MISSING").red());
                println!("  ➜ Please set the {} environment variable.", style(env_var).bold());
            }
        }
    }

    if !found_provider {
        println!("{}", style("No specific LLM provider usage detected in main.rs.").yellow());
        println!("This might be a custom implementation or using a provider not yet tracked by this tool.");
    }

    // Check for .env file presence
    if Path::new(".env").exists() {
        println!("Found .env file: {}", style("✔ YES").green());
    } else {
        println!("Found .env file: {}", style("NO").dim());
    }

    Ok(())
}
