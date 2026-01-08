use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::Path;
use tera::{Context as TeraContext, Tera};

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

#[derive(Parser)]
#[command(name = "radkit")]
#[command(about = "Radkit CLI for scaffolding AI agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new agent project from a template
    Create {
        /// Name of the project directory
        name: Option<String>,

        /// Template to use
        #[arg(short, long)]
        template: Option<String>,

        /// LLM Provider to use (gemini, openai, anthropic)
        #[arg(long)]
        provider: Option<String>,

        /// Path to local radkit dependency (for development)
        #[arg(long)]
        path: Option<String>,
    },
    /// List available templates
    List,
    /// Run the agent
    Run {
        /// Arguments to pass to the agent
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Build the agent
    Build {
        /// Arguments to pass to cargo build
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Tool management commands
    Tool {
        #[command(subcommand)]
        command: ToolCommands,
    },
    /// Skill management commands
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
    /// Provider management commands
    Provider {
        #[command(subcommand)]
        command: ProviderCommands,
    },
}

#[derive(Subcommand)]
enum ToolCommands {
    /// Add a new tool to the project
    Add {
        /// Name of the tool
        name: String,
    },
}

#[derive(Subcommand)]
enum SkillCommands {
    /// Add a new skill to the project
    Add {
        /// Name of the skill
        name: String,
    },
}

#[derive(Subcommand)]
enum ProviderCommands {
    /// Add provider configuration instructions
    Add {
        /// Name of the provider (openai, anthropic, gemini)
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            name,
            template,
            provider,
            path,
        } => {
            create_agent(name, template, provider, path)?;
        }
        Commands::List => {
            list_templates()?;
        }
        Commands::Run { args } => {
            run_agent(args)?;
        }
        Commands::Build { args } => {
            build_agent(args)?;
        }
        Commands::Tool { command } => match command {
            ToolCommands::Add { name } => {
                add_tool(name)?;
            }
        },
        Commands::Skill { command } => match command {
            SkillCommands::Add { name } => {
                add_skill(name)?;
            }
        },
        Commands::Provider { command } => match command {
            ProviderCommands::Add { name } => {
                add_provider(name)?;
            }
        },
    }

    Ok(())
}

fn run_agent(args: Vec<String>) -> Result<()> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("run");
    if !args.is_empty() {
        cmd.arg("--");
        cmd.args(args);
    }

    let status = cmd.status().context("Failed to run cargo run")?;
    if !status.success() {
        anyhow::bail!("Agent run failed");
    }
    Ok(())
}

fn build_agent(args: Vec<String>) -> Result<()> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");
    cmd.args(args);

    let status = cmd.status().context("Failed to run cargo build")?;
    if !status.success() {
        anyhow::bail!("Agent build failed");
    }
    Ok(())
}

fn add_tool(name: String) -> Result<()> {
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

    let tool_path = tools_dir.join(format!("{}.rs", safe_name));
    if tool_path.exists() {
        anyhow::bail!("Tool file 'src/tools/{}.rs' already exists", safe_name);
    }

    let tool_content = format!(
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

#[tool(description = "Description for {name}")]
async fn {name}(_args: {camel_name}Args) -> ToolResult {{
    // Implement tool logic here
    ToolResult::success(json!({{
        "status": "success",
        "message": "Tool executed"
    }}))
}}
"#,
        name = safe_name,
        camel_name = camel_name
    );

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

    Ok(())
}

fn add_skill(name: String) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let cargo_toml = current_dir.join("Cargo.toml");

    if !cargo_toml.exists() {
        anyhow::bail!("Cargo.toml not found. Are you in a radkit agent project root?");
    }

    let skills_dir = current_dir.join("src").join("skills");
    if !skills_dir.exists() {
        fs::create_dir_all(&skills_dir)?;
    }

    // Sanitize name for Rust module/file (snake_case)
    let safe_name = name.replace('-', "_");
    // Camel case for Structs
    let camel_name = to_camel_case(&safe_name);

    let skill_path = skills_dir.join(format!("{}.rs", safe_name));
    if skill_path.exists() {
        anyhow::bail!("Skill file 'src/skills/{}.rs' already exists", safe_name);
    }

    let skill_content = format!(
        r#"use radkit::agent::{{OnRequestResult, SkillHandler}};
use radkit::errors::{{AgentError, AgentResult}};
use radkit::macros::skill;
use radkit::models::Content;
use radkit::runtime::context::{{ProgressSender, State}};
use radkit::runtime::AgentRuntime;
use async_trait::async_trait;

#[skill(
    id = "{safe_name}",
    name = "{camel_name} Skill",
    description = "Description for {camel_name}",
    tags = ["{safe_name}"],
    examples = [],
    input_modes = ["text/plain"],
    output_modes = ["application/json"]
)]
pub struct {camel_name}Skill;

#[async_trait]
impl SkillHandler for {camel_name}Skill {{
    async fn on_request(
        &self,
        _state: &mut State,
        progress: &ProgressSender,
        _runtime: &dyn AgentRuntime,
        content: Content,
    ) -> AgentResult<OnRequestResult> {{
        // Implement skill logic here
        progress.send_update("Processing...").await?;

        Ok(OnRequestResult::Completed {{
            message: Some(Content::from_text("Skill executed")),
            artifacts: vec![],
        }})
    }}
}}
"#,
        safe_name = safe_name,
        camel_name = camel_name
    );

    fs::write(&skill_path, skill_content)?;

    println!(
        "{} Skill created at {}",
        style("✔").green(),
        style(skill_path.display()).bold()
    );

    println!("\nNext steps:");

    // Optional: Try to append to mod.rs automatically
    let mod_rs = skills_dir.join("mod.rs");
    let mod_entry = format!("pub mod {};\n", safe_name);

    if !mod_rs.exists() {
         fs::write(&mod_rs, &mod_entry)?;
         println!("   (Created src/skills/mod.rs and added module declaration)");
    } else {
         let content = fs::read_to_string(&mod_rs)?;
         if !content.contains(&format!("mod {};", safe_name)) {
             use std::io::Write;
             let mut file = fs::OpenOptions::new().append(true).open(&mod_rs)?;
             file.write_all(mod_entry.as_bytes())?;
             println!("   (Added module declaration to src/skills/mod.rs)");
         }
    }

    println!("1. Ensure `pub mod skills;` is in `src/main.rs` or `src/lib.rs`.");
    println!("2. Register the skill in `src/main.rs`:");
    println!(
        "   .with_skill(crate::skills::{}::{}Skill)",
        safe_name, camel_name
    );

    Ok(())
}

fn add_provider(name: String) -> Result<()> {
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

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn list_templates() -> Result<()> {
    println!("Available templates:");
    for entry in TEMPLATES.dirs() {
        let name = entry.path().file_name().unwrap().to_string_lossy();
        println!("  - {}", style(name).cyan());
    }
    Ok(())
}

fn create_agent(
    name: Option<String>,
    template: Option<String>,
    provider: Option<String>,
    path: Option<String>,
) -> Result<()> {
    let project_name = match name {
        Some(n) => n,
        None => Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Project name")
            .interact_text()?,
    };

    let template_name = match template {
        Some(t) => t,
        None => {
            let available: Vec<String> = TEMPLATES
                .dirs()
                .map(|d| d.path().file_name().unwrap().to_string_lossy().to_string())
                .collect();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose a template")
                .items(&available)
                .default(0)
                .interact()?;
            available[selection].clone()
        }
    };

    let provider_name = match provider {
        Some(p) => p.to_lowercase(),
        None => {
            let providers = vec!["Gemini", "OpenAI", "Anthropic"];
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose an LLM provider")
                .items(&providers)
                .default(0)
                .interact()?;
            providers[selection].to_lowercase()
        }
    };

    let target_dir = Path::new(&project_name);
    if target_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", project_name);
    }

    let template_dir = TEMPLATES
        .get_dir(&template_name)
        .context(format!("Template '{}' not found", template_name))?;

    println!(
        "Creating new project '{}' using template '{}' with {}...",
        style(&project_name).green(),
        style(&template_name).cyan(),
        style(&provider_name).yellow()
    );

    let mut tera = Tera::default();
    let mut context = TeraContext::new();
    context.insert("project_name", &project_name);

    // Set provider-specific variables
    let (provider_struct, provider_env_var, default_model) = match provider_name.as_str() {
        "gemini" => ("GeminiLlm", "GEMINI_API_KEY", "gemini-1.5-flash"),
        "openai" => ("OpenAILlm", "OPENAI_API_KEY", "gpt-4o"),
        "anthropic" => ("AnthropicLlm", "ANTHROPIC_API_KEY", "claude-3-5-sonnet-20240620"),
        _ => ("GeminiLlm", "GEMINI_API_KEY", "gemini-1.5-flash"), // Default fallback
    };
    context.insert("provider_struct", provider_struct);
    context.insert("provider_env_var", provider_env_var);
    context.insert("default_model", default_model);
    context.insert("provider_name", &provider_name);

    // Dependency configuration
    let radkit_dependency = if let Some(local_path) = path {
        format!("{{ path = \"{}\", features = [\"runtime\"] }}", local_path)
    } else {
        // Use a default version or allow it to be updated
        "{ version = \"0.0.4\", features = [\"runtime\"] }".to_string()
    };
    context.insert("radkit_dependency", &radkit_dependency);

    // Let's use a helper for recursive extraction
    extract_recursive(template_dir, target_dir, &template_name, &mut tera, &context)?;

    println!(
        "\n{} Project created successfully in {}\n",
        style("✔").green(),
        style(target_dir.display()).bold()
    );
    println!("To get started:");
    println!("  cd {}", project_name);
    println!("  export {}=your_key_here", provider_env_var);
    println!("  cargo run");

    Ok(())
}

fn extract_recursive(
    dir: &Dir,
    base_target: &Path,
    template_root_name: &str,
    tera: &mut Tera,
    context: &TeraContext,
) -> Result<()> {
    for file in dir.files() {
        let path = file.path();
        // The path in `include_dir` is relative to the `include_dir!` root (templates/)
        // e.g. "simple-agent/Cargo.toml"

        let relative_path = path.strip_prefix(template_root_name).unwrap_or(path);
        let dest_path = base_target.join(relative_path);

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = file.contents_utf8().unwrap_or("");
        // Process Cargo.toml and Rust files
        let is_template_file = path.file_name().map_or(false, |n| {
            let s = n.to_string_lossy();
            s == "Cargo.toml" || s.ends_with(".rs")
        });

        let final_content = if is_template_file {
             tera.render_str(content, context)?
        } else {
            content.to_string()
        };

        fs::write(dest_path, final_content)?;
    }

    for subdir in dir.dirs() {
        extract_recursive(subdir, base_target, template_root_name, tera, context)?;
    }

    Ok(())
}
