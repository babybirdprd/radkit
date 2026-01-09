use crate::rewriter::Rewriter;
use crate::utils::to_camel_case;
use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs;

#[allow(dead_code)]
mod templates {
    pub const TODO_LIST_TEMPLATE: &str = r#"use radkit::agent::{OnRequestResult, SkillHandler};
use radkit::errors::{AgentError, AgentResult};
use radkit::macros::skill;
use radkit::models::Content;
use radkit::runtime::context::{ProgressSender, State};
use radkit::runtime::AgentRuntime;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TodoItem {
    id: usize,
    description: String,
    completed: bool,
}

#[skill(
    id = "todo_list",
    name = "Todo List Skill",
    description = "Manage a todo list with state",
    tags = ["todo", "productivity"],
    examples = [],
    input_modes = ["text/plain"],
    output_modes = ["application/json"]
)]
pub struct TodoListSkill {
    items: Arc<Mutex<Vec<TodoItem>>>,
}

impl Default for TodoListSkill {
    fn default() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl SkillHandler for TodoListSkill {
    async fn on_request(
        &self,
        _state: &mut State,
        progress: &ProgressSender,
        _runtime: &dyn AgentRuntime,
        content: Content,
    ) -> AgentResult<OnRequestResult> {
        let text = content.as_text().unwrap_or_default().trim();
        let mut items = self.items.lock().map_err(|_| AgentError::InternalError("Mutex poisoned".into()))?;

        progress.send_update("Processing todo command...").await?;

        let response = if text.starts_with("add ") {
            let description = text[4..].to_string();
            let id = items.len() + 1;
            items.push(TodoItem { id, description: description.clone(), completed: false });
            format!("Added todo #{}", id)
        } else if text.starts_with("list") {
            if items.is_empty() {
                "No items in todo list.".to_string()
            } else {
                items.iter()
                    .map(|i| format!("[{}] {}: {}", if i.completed { "x" } else { " " }, i.id, i.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        } else if text.starts_with("complete ") {
            if let Ok(id) = text[9..].trim().parse::<usize>() {
                 if let Some(item) = items.iter_mut().find(|i| i.id == id) {
                     item.completed = true;
                     format!("Marked todo #{} as complete", id)
                 } else {
                     format!("Todo #{} not found", id)
                 }
            } else {
                "Invalid ID".to_string()
            }
        } else {
             "Commands: add <text>, list, complete <id>".to_string()
        };

        Ok(OnRequestResult::Completed {
            message: Some(Content::from_text(response)),
            artifacts: vec![],
        })
    }
}
"#;
}

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

    let template_options = vec!["Blank Skill", "Todo List (Stateful)"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a skill template")
        .items(&template_options)
        .default(0)
        .interact()?;

    let skill_content = match selection {
        1 => templates::TODO_LIST_TEMPLATE.to_string(), // Todo list is usually fixed name or adapted?
        // Adapting todo list template to use the user's name:
        // replace "todo_list" with safe_name
        // replace "TodoListSkill" with camel_name + "Skill"
        // replace "Todo List Skill" with description
        _ => {
            // Interactive prompts
            let description: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Description of the skill")
                .default(format!("Description for {}", camel_name))
                .interact_text()?;

            format!(
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
                camel_name = camel_name,
                description = description
            )
        }
    };

    // If todo list was selected, we might need to adjust struct names if user didn't name it "todo_list".
    // For simplicity, if they chose "Todo List", we force the name or adapt it.
    // Let's adapt it if they picked a name.
    let final_content = if selection == 1 {
        templates::TODO_LIST_TEMPLATE
            .replace("todo_list", &safe_name)
            .replace("TodoListSkill", &format!("{}Skill", camel_name))
    } else {
        skill_content
    };

    let skill_path = skills_dir.join(format!("{}.rs", safe_name));
    if skill_path.exists() {
        anyhow::bail!("Skill file 'src/skills/{}.rs' already exists", safe_name);
    }

    fs::write(&skill_path, final_content)?;

    println!(
        "{} Skill created at {}",
        style("✔").green(),
        style(skill_path.display()).bold()
    );

    println!("\nNext steps:");

    let mod_rs = skills_dir.join("mod.rs");
    if !mod_rs.exists() {
        fs::write(&mod_rs, format!("pub mod {};\n", safe_name))?;
        println!("   (Created src/skills/mod.rs and added module declaration)");
    } else {
        let content = fs::read_to_string(&mod_rs)?;
        let mut rewriter = Rewriter::new(&content)?;
        rewriter.add_module_declaration(&safe_name);
        fs::write(&mod_rs, rewriter.to_string())?;
        println!("   (Added module declaration to src/skills/mod.rs)");
    }

    // Automatic wiring
    if let Err(e) = wire_skill(&safe_name, &camel_name, &current_dir) {
        println!(
            "{}",
            style(format!("Warning: Automatic wiring failed: {}", e)).yellow()
        );
    } else {
        println!(
            "{}",
            style("✔ Automatically wired skill in main.rs").green()
        );
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
        let mut rewriter = Rewriter::new(&content)?;
        rewriter.remove_module_declaration(&safe_name);
        fs::write(&mod_rs, rewriter.to_string())?;
        println!(
            "{} Removed module declaration from src/skills/mod.rs",
            style("✔").green()
        );
    }

    // 3. Remove from src/main.rs wiring
    unwire_skill(&safe_name, &camel_name, &current_dir)?;

    Ok(())
}

fn unwire_skill(skill_name: &str, camel_name: &str, project_root: &std::path::Path) -> Result<()> {
    let main_rs = project_root.join("src").join("main.rs");
    if !main_rs.exists() {
        return Ok(());
    }

    let skill_struct_name = format!("{}Skill", camel_name);

    let content = fs::read_to_string(&main_rs)?;
    let mut rewriter = Rewriter::new(&content)?;

    rewriter.remove_skill_wiring(skill_name, &skill_struct_name)?;

    fs::write(main_rs, rewriter.to_string())?;
    println!(
        "{} Removed skill wiring from src/main.rs",
        style("✔").green()
    );

    Ok(())
}

fn wire_skill(skill_name: &str, camel_name: &str, project_root: &std::path::Path) -> Result<()> {
    let main_rs = project_root.join("src").join("main.rs");
    if !main_rs.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&main_rs)?;
    let mut rewriter = Rewriter::new(&content)?;

    // 1. Ensure `pub mod skills;` in src/main.rs
    rewriter.add_module_declaration("skills");

    // 2. Add `.with_skill(...)`
    let skill_struct_name = format!("{}Skill", camel_name);
    rewriter.add_skill_wiring(skill_name, &skill_struct_name)?;

    fs::write(main_rs, rewriter.to_string())?;

    Ok(())
}
