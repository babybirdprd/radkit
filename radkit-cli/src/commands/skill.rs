use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use std::fs;
use crate::utils::to_camel_case;

pub fn add_skill(name: String) -> Result<()> {
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

    // Automatic wiring
    if let Err(e) = wire_skill(&safe_name, &camel_name, &current_dir) {
        println!("{}", style(format!("Warning: Automatic wiring failed: {}", e)).yellow());
    } else {
        println!("{}", style("✔ Automatically wired skill in main.rs").green());
    }

    Ok(())
}

fn wire_skill(skill_name: &str, camel_name: &str, project_root: &std::path::Path) -> Result<()> {
    // 1. Ensure `pub mod skills;` in src/main.rs (or lib.rs)
    let main_rs = project_root.join("src").join("main.rs");
    if !main_rs.exists() {
        return Ok(());
    }

    let mut content = fs::read_to_string(&main_rs)?;

    // Add module declaration if missing
    if !content.contains("mod skills;") {
        if let Some(pos) = content.rfind("use ") {
             if let Some(end_line) = content[pos..].find('\n') {
                 let insert_pos = pos + end_line + 1;
                 content.insert_str(insert_pos, "pub mod skills;\n");
             }
        } else {
            content.insert_str(0, "pub mod skills;\n");
        }
    }

    // 2. Add `.with_skill(...)` to the builder chain
    let skill_call = format!("\n        .with_skill(crate::skills::{}::{}Skill)", skill_name, camel_name);

    if !content.contains(&skill_call.trim()) {
        if let Some(pos) = content.rfind(".build()") {
            content.insert_str(pos, &skill_call);
            fs::write(main_rs, content)?;
        }
    }

    Ok(())
}
