use radkit::agent::{Agent, AgentDefinition, Artifact, LlmFunction, OnRequestResult, SkillHandler};
use radkit::errors::AgentError;
use radkit::macros::{skill, LLMOutput};
use radkit::models::{BaseLlm, Content, Thread};
use radkit::models::providers::GeminiLlm;
use radkit::runtime::context::{ProgressSender, State};
use radkit::runtime::Runtime;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// --- Define Output Structure ---
#[derive(Debug, Serialize, Deserialize, JsonSchema, LLMOutput)]
struct WeatherInfo {
    location: String,
    description: String,
    temperature_c: f64,
}

// --- Define Skill ---
#[skill(
    id = "weather_skill",
    name = "Weather Skill",
    description = "Provides weather information for a given location",
    tags = ["weather", "info"],
    input_modes = ["text/plain"],
    output_modes = ["application/json"]
)]
pub struct WeatherSkill;

#[cfg_attr(all(target_os = "wasi", target_env = "p1"), async_trait::async_trait(?Send))]
#[cfg_attr(
    not(all(target_os = "wasi", target_env = "p1")),
    async_trait::async_trait
)]
impl SkillHandler for WeatherSkill {
    async fn on_request(
        &self,
        _state: &mut State,
        progress: &ProgressSender,
        runtime: &dyn Runtime,
        content: Content,
    ) -> Result<OnRequestResult, AgentError> {
        let llm = runtime.llm_provider().default_llm()?;

        progress.send_update("Analyzing request...").await?;

        // Use LLM to extract location and simulate weather lookup
        // In a real scenario, you might call an external API here or have the LLM use tools
        let weather_fn = LlmFunction::<WeatherInfo>::new_with_system_instructions(
            llm,
            "You are a weather simulator. Generate plausible weather data for the location mentioned in the user's request."
        );

        let input_text = content.first_text().unwrap_or("London");
        let info = weather_fn.run(input_text).await?;

        let artifact = Artifact::from_json("weather.json", &info)?;

        Ok(OnRequestResult::Completed {
            message: Some(Content::from_text(format!(
                "Here is the weather for {}: {}, {:.1}°C",
                info.location, info.description, info.temperature_c
            ))),
            artifacts: vec![artifact],
        })
    }
}

// --- Configure Agent ---
pub fn configure_agent() -> AgentDefinition {
    Agent::builder()
        .with_name("A2A Weather Agent")
        .with_description("An agent that provides weather updates via A2A protocol")
        .with_skill(WeatherSkill)
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm = GeminiLlm::from_env("gemini-1.5-flash")?;

    // Serve the agent on localhost
    println!("Starting agent server on http://127.0.0.1:8080");
    Runtime::builder(configure_agent(), llm)
        .build()
        .serve("127.0.0.1:8080")
        .await?;

    Ok(())
}
