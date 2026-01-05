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
    },
    /// List available templates
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { name, template } => {
            create_agent(name, template)?;
        }
        Commands::List => {
            list_templates()?;
        }
    }

    Ok(())
}

fn list_templates() -> Result<()> {
    println!("Available templates:");
    for entry in TEMPLATES.dirs() {
        let name = entry.path().file_name().unwrap().to_string_lossy();
        println!("  - {}", style(name).cyan());
    }
    Ok(())
}

fn create_agent(name: Option<String>, template: Option<String>) -> Result<()> {
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

    let target_dir = Path::new(&project_name);
    if target_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", project_name);
    }

    let template_dir = TEMPLATES
        .get_dir(&template_name)
        .context(format!("Template '{}' not found", template_name))?;

    println!(
        "Creating new project '{}' using template '{}'...",
        style(&project_name).green(),
        style(&template_name).cyan()
    );

    let mut tera = Tera::default();
    let mut context = TeraContext::new();
    context.insert("project_name", &project_name);

    // Let's use a helper for recursive extraction
    extract_recursive(template_dir, target_dir, &template_name, &mut tera, &context)?;

    println!(
        "\n{} Project created successfully in {}\n",
        style("✔").green(),
        style(target_dir.display()).bold()
    );
    println!("To get started:");
    println!("  cd {}", project_name);
    println!("  export GEMINI_API_KEY=your_key_here");
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
        // Only process Cargo.toml for now
        let final_content = if path.file_name().map_or(false, |n| n == "Cargo.toml") {
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
