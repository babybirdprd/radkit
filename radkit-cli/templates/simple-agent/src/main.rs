use radkit::agent::LlmWorker;
use radkit::models::providers::{{ provider_struct }};
use radkit::macros::{tool, LLMOutput};
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, JsonSchema, LLMOutput)]
struct WeatherReport {
    location: String,
    temperature: f64,
    condition: String,
}

#[derive(Deserialize, JsonSchema)]
struct GetWeatherArgs {
    /// City name or location
    location: String,
}

#[tool(description = "Get current weather for a location")]
async fn get_weather(args: GetWeatherArgs) -> ToolResult {
    // In a real app, you would call a weather API here.
    let location = args.location.to_lowercase();
    let (temp, condition) = if location.contains("sf") || location.contains("francisco") {
        (60.0, "Foggy")
    } else {
        (90.0, "Sunny")
    };

    ToolResult::success(json!({
        "temperature": temp,
        "condition": condition,
        "location": args.location,
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize LLM from environment (e.g. {{ provider_env_var }})
    // You can swap this for AnthropicLlm, OpenAILlm, etc.
    let llm = {{ provider_struct }}::from_env("{{ default_model }}")?;

    let worker = LlmWorker::<WeatherReport>::builder(llm)
        .with_system_instructions("You are a helpful weather assistant.")
        .with_tool(get_weather)
        .build();

    // Example run
    let report = worker.run("What's the weather in San Francisco?").await?;

    println!("📍 Location: {}", report.location);
    println!("🌡️  Temperature: {}°F", report.temperature);
    println!("☀️  Condition: {}", report.condition);

    Ok(())
}
