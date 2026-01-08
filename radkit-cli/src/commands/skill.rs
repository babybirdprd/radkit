use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use std::fs;
use crate::utils::{self, to_camel_case, validate_rust_identifier};

pub fn add_skill(name: String) -> Result<()> {
    // Sanitize name for Rust module/file (snake_case)
    let safe_name = name.replace('-', "_");

    // Validate after sanitization
    validate_rust_identifier(&safe_name)?;

    let current_dir = std::env::current_dir()?;
    let cargo_toml = current_dir.join("Cargo.toml");

    if !cargo_toml.exists() {
        anyhow::bail!("Cargo.toml not found. Are you in a radkit agent project root?");
    }

    let skills_dir = current_dir.join("src").join("skills");
    if !skills_dir.exists() {
        fs::create_dir_all(&skills_dir)?;
    }

    // Camel case for Structs
    let camel_name = to_camel_case(&safe_name);

    // Interactive prompts
    let description: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Description of the skill")
        .default(format!("Description for {}", camel_name))
        .interact_text()?;

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
    description = "{description}",
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

    // Use utils to ensure mod declaration in main.rs
    utils::ensure_mod_decl(&current_dir, "skills")?;

    // Use utils to register module in skills/mod.rs
    utils::register_child_module(&skills_dir, &safe_name)?;

    println!("1. Ensure `pub mod skills;` is in `src/main.rs` or `src/lib.rs` (attempted automatically).");
    println!("2. Register the skill in `src/main.rs`:");
    println!(
        "   .with_skill(crate::skills::{}::{}Skill)",
        safe_name, camel_name
    );

    // Automatic wiring using utils
    let skill_call = format!("\n        .with_skill(crate::skills::{}::{}Skill)", safe_name, camel_name);
    if let Err(e) = utils::wire_in_main(&current_dir, &skill_call) {
        println!("{}", style(format!("Warning: Automatic wiring failed: {}", e)).yellow());
    }

    Ok(())
}

pub fn list_skills() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let skills_dir = current_dir.join("src").join("skills");

    if !skills_dir.exists() {
        println!("No skills found (src/skills directory does not exist).");
        return Ok(());
    }

    println!("Available skills:");
    let mut found = false;
    for entry in fs::read_dir(skills_dir)? {
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
        println!("  (No skills found)");
    }

    Ok(())
}

pub fn remove_skill(name: String) -> Result<()> {
    // Basic validation, ensure_rust_identifier is stricter
    if name.contains('.') || name.contains('/') || name.contains('\\') {
        anyhow::bail!("Invalid skill name: {}", name);
    }

    let current_dir = std::env::current_dir()?;
    let skills_dir = current_dir.join("src").join("skills");
    let safe_name = name.replace('-', "_");
    let camel_name = to_camel_case(&safe_name);
    let skill_path = skills_dir.join(format!("{}.rs", safe_name));

    if !skill_path.exists() {
        anyhow::bail!("Skill '{}' not found at {}", name, skill_path.display());
    }

    // 1. Delete the file
    fs::remove_file(&skill_path)?;
    println!(
        "{} Removed skill file {}",
        style("✔").green(),
        style(skill_path.display()).bold()
    );

    // 2. Remove from src/skills/mod.rs
    let mod_rs = skills_dir.join("mod.rs");
    if mod_rs.exists() {
        let content = fs::read_to_string(&mod_rs)?;
        let mod_decl = format!("pub mod {};\n", safe_name);
        let new_content = content.replace(&mod_decl, "");
        // Try fallback
        let new_content = if new_content == content {
            content.replace(&format!("pub mod {};", safe_name), "")
                   .replace(&format!("mod {};", safe_name), "")
        } else {
            new_content
        };

        fs::write(&mod_rs, new_content.trim())?;
        println!("{} Removed module declaration from src/skills/mod.rs", style("✔").green());
    }

    // 3. Remove from src/main.rs wiring
    let skill_call_substr = format!("crate::skills::{}::{}Skill", safe_name, camel_name);
    utils::unwire_in_main(&current_dir, &skill_call_substr)?;

    Ok(())
}
