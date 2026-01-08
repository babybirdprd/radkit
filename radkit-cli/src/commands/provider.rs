use anyhow::Result;
use console::style;

pub fn add_provider(name: String) -> Result<()> {
    let (provider_struct, env_var) = match name.to_lowercase().as_str() {
        "gemini" => ("GeminiLlm", "GEMINI_API_KEY"),
        "openai" => ("OpenAILlm", "OPENAI_API_KEY"),
        "anthropic" => ("AnthropicLlm", "ANTHROPIC_API_KEY"),
        _ => anyhow::bail!("Unknown provider '{}'. Supported: gemini, openai, anthropic", name),
    };

    println!("{}", style("Provider Configuration Instructions").bold());
    println!("{}", style("-----------------------------------").dim());

    println!("1. Add the environment variable:");
    println!("   export {}=your_key_here", env_var);

    println!("\n2. Update your `src/main.rs` to use `{}`:", provider_struct);
    println!("\n   use radkit::models::providers::{};", provider_struct);
    println!("   let llm = {}::from_env(\"model-name\")?;", provider_struct);

    Ok(())
}
