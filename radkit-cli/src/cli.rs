use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "radkit")]
#[command(about = "Radkit CLI for scaffolding AI agents", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum ToolCommands {
    /// Add a new tool to the project
    Add {
        /// Name of the tool
        name: String,
    },
}

#[derive(Subcommand)]
pub enum SkillCommands {
    /// Add a new skill to the project
    Add {
        /// Name of the skill
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ProviderCommands {
    /// Add provider configuration instructions
    Add {
        /// Name of the provider (openai, anthropic, gemini)
        name: String,
    },
}
