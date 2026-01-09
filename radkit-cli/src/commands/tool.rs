use crate::rewriter::Rewriter;
use crate::utils::to_camel_case;
use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs;

#[allow(dead_code)]
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
    println!("Searching web for: {}", args.query);

    ToolResult::success(json!({
        "results": [
            { "title": "Result 1", "snippet": "Snippet for result 1" },
            { "title": "Result 2", "snippet": "Snippet for result 2" }
        ]
    }))
}
"#;

    pub const FILE_READER_TEMPLATE: &str = r#"use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use std::fs;

#[derive(Deserialize, JsonSchema)]
struct FileReaderArgs {
    path: String,
}

#[tool(description = "Read the contents of a file")]
async fn file_reader(args: FileReaderArgs) -> ToolResult {
    match fs::read_to_string(&args.path) {
        Ok(content) => ToolResult::success(json!({ "content": content })),
        Err(e) => ToolResult::error(format!("Failed to read file: {}", e)),
    }
}
"#;

    pub const FILE_WRITER_TEMPLATE: &str = r#"use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use std::fs;

#[derive(Deserialize, JsonSchema)]
struct FileWriterArgs {
    path: String,
    content: String,
}

#[tool(description = "Write content to a file")]
async fn file_writer(args: FileWriterArgs) -> ToolResult {
    match fs::write(&args.path, &args.content) {
        Ok(_) => ToolResult::success(json!({ "status": "success", "path": args.path })),
        Err(e) => ToolResult::error(format!("Failed to write file: {}", e)),
    }
}
"#;

    pub const COMMAND_RUNNER_TEMPLATE: &str = r#"use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use std::process::Command;

#[derive(Deserialize, JsonSchema)]
struct CommandRunnerArgs {
    command: String,
    args: Vec<String>,
}

#[tool(description = "Run a shell command")]
async fn command_runner(args: CommandRunnerArgs) -> ToolResult {
    match Command::new(&args.command).args(&args.args).output() {
        Ok(output) => {
             let stdout = String::from_utf8_lossy(&output.stdout);
             let stderr = String::from_utf8_lossy(&output.stderr);
             ToolResult::success(json!({
                 "stdout": stdout,
                 "stderr": stderr,
                 "exit_code": output.status.code()
             }))
        },
        Err(e) => ToolResult::error(format!("Failed to run command: {}", e)),
    }
}
"#;

    pub const HTTP_REQUEST_TEMPLATE: &str = r#"use radkit::macros::tool;
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, JsonSchema)]
struct HttpRequestArgs {
    url: String,
    method: String,
    body: Option<String>,
}

#[tool(description = "Make an HTTP request")]
async fn http_request(args: HttpRequestArgs) -> ToolResult {
    // Requires `reqwest` dependency in Cargo.toml
    // For this template to work out of the box, we might need to add it or warn.
    // Simulating request for now.
    println!("{} request to {}", args.method, args.url);

    ToolResult::success(json!({
        "status": 200,
        "body": "Response content placeholder"
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
    let template_options = vec![
        "Blank Tool",
        "Calculator",
        "Web Search",
        "File Reader",
        "File Writer",
        "Command Runner",
        "HTTP Request",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a tool template")
        .items(&template_options)
        .default(0)
        .interact()?;

    let tool_content = match selection {
        1 => templates::CALCULATOR_TEMPLATE
            .replace("calculator", &safe_name)
            .replace("Calculator", &camel_name),
        2 => templates::WEB_SEARCH_TEMPLATE
            .replace("web_search", &safe_name)
            .replace("WebSearch", &camel_name),
        3 => templates::FILE_READER_TEMPLATE
            .replace("file_reader", &safe_name)
            .replace("FileReader", &camel_name),
        4 => templates::FILE_WRITER_TEMPLATE
            .replace("file_writer", &safe_name)
            .replace("FileWriter", &camel_name),
        5 => templates::COMMAND_RUNNER_TEMPLATE
            .replace("command_runner", &safe_name)
            .replace("CommandRunner", &camel_name),
        6 => templates::HTTP_REQUEST_TEMPLATE
            .replace("http_request", &safe_name)
            .replace("HttpRequest", &camel_name),
        _ => {
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
}}

#[tool(description = "{description}")]
async fn {name}(_args: {camel_name}Args) -> ToolResult {{
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

    // Mod registration using Rewriter for src/tools/mod.rs logic?
    // src/tools/mod.rs is just a file with `pub mod x;`.
    // Rewriter works on File object.

    let mod_rs = tools_dir.join("mod.rs");
    if !mod_rs.exists() {
        fs::write(&mod_rs, format!("pub mod {};\n", safe_name))?;
        println!("   (Created src/tools/mod.rs and added module declaration)");
    } else {
        let content = fs::read_to_string(&mod_rs)?;
        let mut rewriter = Rewriter::new(&content)?;
        rewriter.add_module_declaration(&safe_name);
        fs::write(&mod_rs, rewriter.to_string())?;
        println!("   (Added module declaration to src/tools/mod.rs)");
    }

    // Automatic wiring in main.rs
    if let Err(e) = wire_tool(&safe_name, &current_dir) {
        println!(
            "{}",
            style(format!("Warning: Automatic wiring failed: {}", e)).yellow()
        );
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
        let mut rewriter = Rewriter::new(&content)?;
        rewriter.remove_module_declaration(&safe_name);
        fs::write(&mod_rs, rewriter.to_string())?;
        println!(
            "{} Removed module declaration from src/tools/mod.rs",
            style("✔").green()
        );
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
    let mut rewriter = Rewriter::new(&content)?;

    rewriter.remove_tool_wiring(tool_name, tool_name)?; // tool mod and func usually same name

    fs::write(main_rs, rewriter.to_string())?;
    println!(
        "{} Removed tool wiring from src/main.rs",
        style("✔").green()
    );

    Ok(())
}

fn wire_tool(tool_name: &str, project_root: &std::path::Path) -> Result<()> {
    let main_rs = project_root.join("src").join("main.rs");
    if !main_rs.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&main_rs)?;
    let mut rewriter = Rewriter::new(&content)?;

    // 1. Ensure `pub mod tools;` in src/main.rs
    rewriter.add_module_declaration("tools");

    // 2. Add `.with_tool(...)`
    rewriter.add_tool_wiring(tool_name, tool_name)?;

    fs::write(main_rs, rewriter.to_string())?;

    Ok(())
}
