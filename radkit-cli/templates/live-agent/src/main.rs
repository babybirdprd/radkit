use radkit::agent::LlmWorker;
use radkit::models::providers::GeminiLlm;
use radkit::models::BaseLlm;
use radkit::macros::{tool, LLMOutput};
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, JsonSchema, LLMOutput)]
struct QuickResponse {
    content: String,
    sentiment: String,
}

#[derive(Deserialize, JsonSchema)]
struct CalculatorArgs {
    expression: String,
}

#[tool(description = "Calculate mathematical expression")]
async fn calculate(args: CalculatorArgs) -> ToolResult {
    // Basic mock calculator
    let result = "42"; // Placeholder
    ToolResult::success(json!({ "result": result, "expression": args.expression }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Using a fast model for "live" feel
    let llm = GeminiLlm::from_env("gemini-2.0-flash-exp")?
        .with_temperature(0.3);

    let worker = LlmWorker::<QuickResponse>::builder(llm)
        .with_system_instructions("You are a high-performance, low-latency assistant. Be concise.")
        .with_tool(calculate)
        .build();

    let response = worker.run("What is the meaning of life?").await?;

    println!("Response: {}", response.content);
    println!("Sentiment: {}", response.sentiment);

    Ok(())
}
