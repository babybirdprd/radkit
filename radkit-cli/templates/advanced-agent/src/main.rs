use radkit::agent::Agent;
use radkit::models::providers::{{ provider_struct }};
use radkit::runtime::RuntimeHandle;
use std::sync::Arc;

mod skills;
use skills::research::ResearchSkill;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing Advanced Agent...");

    // 1. Initialize LLM
    let llm = {{ provider_struct }}::from_env("{{ default_model }}")?;

    // 2. Create Agent with Skills
    // Note: Skill-based agents use the `Agent` builder and `RuntimeHandle`
    // instead of `LlmWorker`.
    let agent = Agent::builder()
        .with_name("{{ project_name }}")
        .with_preamble("You are an advanced research assistant.")
        .with_skill(ResearchSkill)
        .build();

    // 3. Create Runtime
    let runtime = RuntimeHandle::builder(agent, llm).build();

    println!("Agent ready! (This template demonstrates the Runtime architecture)");
    println!("In a real application, you would connect this to A2A or a server.");

    // Example usage of the runtime (if applicable in this template's context)
    // For now, we just show it compiles and initializes.

    Ok(())
}
