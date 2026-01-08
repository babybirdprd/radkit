use anyhow::{Context, Result};

pub fn run_agent(args: Vec<String>) -> Result<()> {
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

pub fn build_agent(args: Vec<String>) -> Result<()> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");
    cmd.args(args);

    let status = cmd.status().context("Failed to run cargo build")?;
    if !status.success() {
        anyhow::bail!("Agent build failed");
    }
    Ok(())
}
