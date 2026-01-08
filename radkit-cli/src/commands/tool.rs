use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs;
use crate::utils::{self, to_camel_case, validate_rust_identifier};

#[allow(dead_code)] // For now, until we wire it up fully if needed elsewhere
mod templates {
    pub const CALCULATOR_TEMPLATE: &str = r#"use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
struct CalculatorArgs {
    expression: String,
}

#[tool(description = "Calculate the result of a mathematical expression")]
async fn calculator(args: CalculatorArgs) -> ToolResult {
    // In a real implementation, consider using a crate like `meval`
    let result = match args.expression.trim() {
        "1 + 1" => 2.0,
        _ => 0.0, // Placeholder
    };

    ToolResult::success(json!({
        "expression": args.expression,
        "result": result
    }))
}
"#;

    pub const WEB_SEARCH_TEMPLATE: &str = r#"use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
struct WebSearchArgs {
    query: String,
}

#[tool(description = "Search the web for information")]
async fn web_search(args: WebSearchArgs) -> ToolResult {
    // In a real implementation, you would call a search API (Google, Bing, Tavily)
    println!("Searching web for: {}", args.query);

    ToolResult::success(json!({
        "results": [
            { "title": "Result 1", "snippet": "Snippet for result 1" },
            { "title": "Result 2", "snippet": "Snippet for result 2" }
        ]
    }))
}
"#;
}

pub fn add_tool(name: String) -> Result<()> {
    // Sanitize name for Rust module/file (snake_case)
    let safe_name = name.replace('-', "_");

    // Validate after sanitization
    validate_rust_identifier(&safe_name)?;

    let current_dir = std::env::current_dir()?;
    let cargo_toml = current_dir.join("Cargo.toml");

    if !cargo_toml.exists() {
        anyhow::bail!("Cargo.toml not found. Are you in a radkit agent project root?");
    }

    let tools_dir = current_dir.join("src").join("tools");
    if !tools_dir.exists() {
        fs::create_dir_all(&tools_dir)?;
    }

    // Camel case for Structs
    let camel_name = to_camel_case(&safe_name);

    // Ask for template type
    let template_options = vec!["Blank Tool", "Calculator (Standard)", "Web Search (Standard)"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a tool template")
        .items(&template_options)
        .default(0)
        .interact()?;

    let tool_content = match selection {
        1 => templates::CALCULATOR_TEMPLATE.replace("calculator", &safe_name).replace("Calculator", &camel_name), // Naive replace
        2 => templates::WEB_SEARCH_TEMPLATE.replace("web_search", &safe_name).replace("WebSearch", &camel_name),
        _ => {
             // Interactive prompts for blank tool
             let description: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Description of the tool")
                .default(format!("Description for {}", safe_name))
                .interact_text()?;

             format!(
                r#"use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
struct {camel_name}Args {{
    // Add arguments here
    // location: String,
}}

#[tool(description = "{description}")]
async fn {name}(_args: {camel_name}Args) -> ToolResult {{
    // Implement tool logic here
    ToolResult::success(json!({{
        "status": "success",
        "message": "Tool executed"
    }}))
}}
"#,
                name = safe_name,
                camel_name = camel_name,
                description = description
            )
        }
    };

    let tool_path = tools_dir.join(format!("{}.rs", safe_name));
    if tool_path.exists() {
        anyhow::bail!("Tool file 'src/tools/{}.rs' already exists", safe_name);
    }

    fs::write(&tool_path, tool_content)?;

    println!(
        "{} Tool created at {}",
        style("✔").green(),
        style(tool_path.display()).bold()
    );

    println!("\nNext steps:");

    // Use utils to ensure mod declaration in main.rs
    utils::ensure_mod_decl(&current_dir, "tools")?;

    // Use utils to register module in tools/mod.rs
    utils::register_child_module(&tools_dir, &safe_name)?;

    println!("1. Ensure `pub mod tools;` is in `src/main.rs` or `src/lib.rs` (attempted automatically).");
    println!("2. Register the tool in `src/main.rs`:");
    println!(
        "   .with_tool(crate::tools::{}::{})",
        safe_name, safe_name
    );

    // Automatic wiring using utils
    let tool_call = format!("\n        .with_tool(crate::tools::{}::{})", safe_name, safe_name);
    if let Err(e) = utils::wire_in_main(&current_dir, &tool_call) {
        println!("{}", style(format!("Warning: Automatic wiring failed: {}", e)).yellow());
    }

    Ok(())
}

pub fn list_tools() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let tools_dir = current_dir.join("src").join("tools");

    if !tools_dir.exists() {
        println!("No tools found (src/tools directory does not exist).");
        return Ok(());
    }

    println!("Available tools:");
    let mut found = false;
    for entry in fs::read_dir(tools_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let file_name = path.file_stem().unwrap().to_string_lossy();
            if file_name != "mod" {
                println!("  - {}", style(file_name).cyan());
                found = true;
            }
        }
    }

    if !found {
        println!("  (No tools found)");
    }

    Ok(())
}

pub fn remove_tool(name: String) -> Result<()> {
    // Basic validation to prevent path traversal, though ensure_rust_identifier is stricter
    if name.contains('.') || name.contains('/') || name.contains('\\') {
        anyhow::bail!("Invalid tool name: {}", name);
    }

    let current_dir = std::env::current_dir()?;
    let tools_dir = current_dir.join("src").join("tools");
    let safe_name = name.replace('-', "_");
    let tool_path = tools_dir.join(format!("{}.rs", safe_name));

    if !tool_path.exists() {
        anyhow::bail!("Tool '{}' not found at {}", name, tool_path.display());
    }

    // 1. Delete the file
    fs::remove_file(&tool_path)?;
    println!(
        "{} Removed tool file {}",
        style("✔").green(),
        style(tool_path.display()).bold()
    );

    // 2. Remove from src/tools/mod.rs
    let mod_rs = tools_dir.join("mod.rs");
    if mod_rs.exists() {
        let content = fs::read_to_string(&mod_rs)?;
        let mod_decl = format!("pub mod {};\n", safe_name);
        // Also handle potential non-pub mod or different formatting, but strict match is safer for now
        let new_content = content.replace(&mod_decl, "");
        // If it didn't match perfectly, try without newline or just 'mod'
        let new_content = if new_content == content {
            content.replace(&format!("pub mod {};", safe_name), "")
                   .replace(&format!("mod {};", safe_name), "")
        } else {
            new_content
        };

        fs::write(&mod_rs, new_content.trim())?;
        println!("{} Removed module declaration from src/tools/mod.rs", style("✔").green());
    }

    // 3. Remove from src/main.rs wiring
    let tool_call_substr = format!("crate::tools::{}::{}", safe_name, safe_name);
    utils::unwire_in_main(&current_dir, &tool_call_substr)?;

    Ok(())
}
