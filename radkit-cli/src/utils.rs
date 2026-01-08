use anyhow::{Context, Result};
use console::style;
use std::fs;
use std::path::Path;
use std::io::Write;

pub fn to_camel_case(s: &str) -> String {
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

pub fn validate_rust_identifier(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Name cannot be empty");
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        anyhow::bail!("Name must start with a letter or underscore");
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' {
            anyhow::bail!("Name must contain only alphanumeric characters and underscores");
        }
    }

    let keywords = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
        "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
        "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
        "where", "while", "async", "await", "dyn",
    ];

    if keywords.contains(&name) {
        anyhow::bail!("'{}' is a reserved Rust keyword", name);
    }

    Ok(())
}

/// Ensures that `mod {mod_name};` exists in `src/main.rs` and that `src/{mod_name}/mod.rs` exists.
pub fn ensure_mod_decl(project_root: &Path, mod_name: &str) -> Result<()> {
    // 1. Ensure src/{mod_name}/mod.rs exists
    let mod_dir = project_root.join("src").join(mod_name);
    if !mod_dir.exists() {
        fs::create_dir_all(&mod_dir)?;
    }

    let mod_rs = mod_dir.join("mod.rs");
    if !mod_rs.exists() {
         fs::write(&mod_rs, "")?; // Create empty mod.rs if it doesn't exist
         println!("   (Created src/{}/mod.rs)", mod_name);
    }

    // 2. Ensure `pub mod {mod_name};` in src/main.rs (or lib.rs if main doesn't exist)
    // We prioritize main.rs for agents
    let main_rs = project_root.join("src").join("main.rs");
    if !main_rs.exists() {
        return Ok(());
    }

    let mut content = fs::read_to_string(&main_rs)?;

    let mod_decl = format!("mod {};", mod_name);
    if !content.contains(&mod_decl) {
        // Naive insertion: find the last `use` or `mod` and insert after, or at top
        if let Some(pos) = content.rfind("use ") {
             if let Some(end_line) = content[pos..].find('\n') {
                 let insert_pos = pos + end_line + 1;
                 content.insert_str(insert_pos, &format!("pub mod {};\n", mod_name));
             }
        } else {
            content.insert_str(0, &format!("pub mod {};\n", mod_name));
        }
        fs::write(main_rs, content)?;
        println!("   (Added `pub mod {};` to src/main.rs)", mod_name);
    }

    Ok(())
}

/// Registers a child module (e.g. `my_tool`) in the parent directory's `mod.rs` (e.g. `src/tools/mod.rs`).
pub fn register_child_module(parent_dir: &Path, child_name: &str) -> Result<()> {
    let mod_rs = parent_dir.join("mod.rs");

    // Ensure mod.rs exists (it should, but just in case)
    if !mod_rs.exists() {
        fs::write(&mod_rs, "")?;
    }

    let mod_entry = format!("pub mod {};\n", child_name);
    let content = fs::read_to_string(&mod_rs)?;

    // Check if it's already there (naive check)
    if !content.contains(&format!("mod {};", child_name)) {
        let mut file = fs::OpenOptions::new().append(true).open(&mod_rs)?;
        file.write_all(mod_entry.as_bytes())?;
        println!("   (Added module declaration to {})", mod_rs.display());
    }

    Ok(())
}

pub fn wire_in_main(project_root: &Path, code_snippet: &str) -> Result<()> {
    let main_rs = project_root.join("src").join("main.rs");
    if !main_rs.exists() {
        return Ok(());
    }

    let mut content = fs::read_to_string(&main_rs)?;

    if !content.contains(code_snippet.trim()) {
        // Try to find a good anchor point.
        // We look for `.build()` and insert before it.
        if let Some(pos) = content.rfind(".build()") {
            content.insert_str(pos, code_snippet);
            fs::write(main_rs, content)?;
            println!("{}", style("✔ Automatically wired in main.rs").green());
        } else {
            println!("{}", style("Warning: Could not find .build() in main.rs to wire component").yellow());
        }
    }

    Ok(())
}

pub fn unwire_in_main(project_root: &Path, search_substr: &str) -> Result<()> {
    let main_rs = project_root.join("src").join("main.rs");
    if !main_rs.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&main_rs)?;
    let mut new_lines: Vec<&str> = Vec::new();
    let mut changed = false;

    for line in content.lines() {
        if line.contains(search_substr) {
            changed = true;
            continue; // Skip this line
        }
        new_lines.push(line);
    }

    if changed {
        fs::write(main_rs, new_lines.join("\n"))?;
        println!("{} Removed wiring from src/main.rs", style("✔").green());
    } else {
         println!("{}", style("Warning: Could not find wiring in src/main.rs to remove").yellow());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("my_tool"), "MyTool");
        assert_eq!(to_camel_case("my-tool"), "MyTool");
        assert_eq!(to_camel_case("tool"), "Tool");
    }

    #[test]
    fn test_validate_rust_identifier() {
        assert!(validate_rust_identifier("valid_name").is_ok());
        assert!(validate_rust_identifier("ValidName").is_ok());
        assert!(validate_rust_identifier("_valid").is_ok());

        assert!(validate_rust_identifier("invalid-name").is_err());
        assert!(validate_rust_identifier("1nvalid").is_err());
        assert!(validate_rust_identifier("fn").is_err());
        assert!(validate_rust_identifier("").is_err());
    }
}
