use anyhow::{Context, Result};
use colored::*;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use include_dir::{include_dir, Dir};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use tera::{Context as TeraContext, Tera};

static TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub fn create_agent(
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
            let providers = vec![
                "Gemini",
                "OpenAI",
                "Anthropic",
                "DeepSeek",
                "Grok",
                "OpenRouter",
            ];
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
        project_name.as_str().green().bold(),
        template_name.as_str().cyan(),
        provider_name.as_str().yellow()
    );

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Scaffolding project files...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut tera = Tera::default();
    let mut context = TeraContext::new();
    context.insert("project_name", &project_name);

    // Set provider-specific variables
    let (provider_struct, provider_env_var, default_model) = match provider_name.as_str() {
        "gemini" => ("GeminiLlm", "GEMINI_API_KEY", "gemini-1.5-flash"),
        "openai" => ("OpenAILlm", "OPENAI_API_KEY", "gpt-4o"),
        "anthropic" => (
            "AnthropicLlm",
            "ANTHROPIC_API_KEY",
            "claude-3-5-sonnet-20240620",
        ),
        "deepseek" => ("DeepSeekLlm", "DEEPSEEK_API_KEY", "deepseek-chat"),
        "grok" => ("GrokLlm", "XAI_API_KEY", "grok-beta"),
        "openrouter" => (
            "OpenRouterLlm",
            "OPENROUTER_API_KEY",
            "google/gemini-2.0-flash-001",
        ),
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
    extract_recursive(
        template_dir,
        target_dir,
        &template_name,
        &mut tera,
        &context,
    )?;

    pb.finish_and_clear();

    println!(
        "\n{} Project created successfully in {}\n",
        "✔".green(),
        target_dir.display().to_string().bold()
    );
    println!("To get started:");
    println!("  cd {}", project_name);
    println!("  export {}=your_key_here", provider_env_var);
    println!("  cargo run");

    Ok(())
}

pub fn list_templates() -> Result<()> {
    println!("Available templates:");
    for entry in TEMPLATES.dirs() {
        let name = entry.path().file_name().unwrap().to_string_lossy();
        println!("  - {}", style(name).cyan());
    }
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
