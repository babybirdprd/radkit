use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs;
use crate::utils::to_camel_case;

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
    let current_dir = std::env::current_dir()?;
    let cargo_toml = current_dir.join("Cargo.toml");

    if !cargo_toml.exists() {
        anyhow::bail!("Cargo.toml not found. Are you in a radkit agent project root?");
    }

    let tools_dir = current_dir.join("src").join("tools");
    if !tools_dir.exists() {
        fs::create_dir_all(&tools_dir)?;
    }

    // Sanitize name for Rust module/file (snake_case)
    let safe_name = name.replace('-', "_");
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

    // Optional: Try to append to mod.rs automatically
    let mod_rs = tools_dir.join("mod.rs");
    let mod_entry = format!("pub mod {};\n", safe_name);

    if !mod_rs.exists() {
         fs::write(&mod_rs, &mod_entry)?;
         println!("   (Created src/tools/mod.rs and added module declaration)");
    } else {
         let content = fs::read_to_string(&mod_rs)?;
         if !content.contains(&format!("mod {};", safe_name)) {
             use std::io::Write;
             let mut file = fs::OpenOptions::new().append(true).open(&mod_rs)?;
             file.write_all(mod_entry.as_bytes())?;
             println!("   (Added module declaration to src/tools/mod.rs)");
         }
    }

    println!("1. Ensure `pub mod tools;` is in `src/main.rs` or `src/lib.rs`.");
    println!("2. Register the tool in `src/main.rs`:");
    println!(
        "   .with_tool(crate::tools::{}::{})",
        safe_name, safe_name
    );

    // Automatic wiring
    if let Err(e) = wire_tool(&safe_name, &current_dir) {
        println!("{}", style(format!("Warning: Automatic wiring failed: {}", e)).yellow());
    } else {
        println!("{}", style("✔ Automatically wired tool in main.rs").green());
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
    unwire_tool(&safe_name, &current_dir)?;

    Ok(())
}

fn unwire_tool(tool_name: &str, project_root: &std::path::Path) -> Result<()> {
    let main_rs = project_root.join("src").join("main.rs");
    if !main_rs.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&main_rs)?;

    // Pattern to look for: .with_tool(crate::tools::{tool_name}::{tool_name})
    // We check for the tool call ignoring whitespace
    let tool_call_substr = format!("crate::tools::{}::{}", tool_name, tool_name);

    let mut new_lines: Vec<&str> = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        if line.contains(".with_tool(") && line.contains(&tool_call_substr) {
            changed = true;
            continue; // Skip this line
        }
        new_lines.push(line);
    }

    if changed {
        fs::write(main_rs, new_lines.join("\n"))?;
        println!("{} Removed tool wiring from src/main.rs", style("✔").green());
    } else {
         println!("{}", style("Warning: Could not find tool wiring in src/main.rs to remove").yellow());
    }

    Ok(())
}

fn wire_tool(tool_name: &str, project_root: &std::path::Path) -> Result<()> {
    // 1. Ensure `pub mod tools;` in src/main.rs (or lib.rs)
    let main_rs = project_root.join("src").join("main.rs");
    if !main_rs.exists() {
        return Ok(());
    }

    let mut content = fs::read_to_string(&main_rs)?;

    // Add module declaration if missing
    if !content.contains("mod tools;") {
        // Naive insertion: find the last `use` or `mod` and insert after, or at top
        // For simplicity, let's insert after the last `use ...;` block or at top if none.
        if let Some(pos) = content.rfind("use ") {
             if let Some(end_line) = content[pos..].find('\n') {
                 let insert_pos = pos + end_line + 1;
                 content.insert_str(insert_pos, "pub mod tools;\n");
             }
        } else {
            content.insert_str(0, "pub mod tools;\n");
        }
    }

    // 2. Add `.with_tool(...)` to the builder chain
    // Look for `.builder(` or `.with_tool(` or `.with_system_instructions(`
    // We want to insert .with_tool(crate::tools::{tool_name}::{tool_name})

    let tool_call = format!("\n        .with_tool(crate::tools::{}::{})", tool_name, tool_name);

    if !content.contains(&tool_call.trim()) {
        // Try to find a good anchor point.
        // We look for `.build()` and insert before it.
        if let Some(pos) = content.rfind(".build()") {
            content.insert_str(pos, &tool_call);
            fs::write(main_rs, content)?;
        }
    }

    Ok(())
}
