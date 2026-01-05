use radkit::agent::LlmWorker;
use radkit::models::providers::GeminiLlm;
use radkit::macros::{tool, LLMOutput};
use radkit::tools::ToolResult;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, JsonSchema, LLMOutput)]
struct Answer {
    response: String,
    sources: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
struct SearchArgs {
    query: String,
}

// Mock retrieval tool
#[tool(description = "Retrieve relevant documents for a query")]
async fn retrieve_docs(args: SearchArgs) -> ToolResult {
    // In a real RAG system, you would query a vector database (e.g., Qdrant, Pinecone) here.
    // This is a provider-agnostic mock.

    let docs = vec![
        "Radkit is a Rust SDK for building reliable AI agent systems.",
        "The A2A Protocol enables seamless communication between AI agents.",
        "Radkit supports unified LLM interfaces for Anthropic, OpenAI, Gemini, and more.",
    ];

    // Simple keyword matching for demo purposes
    let relevant_docs: Vec<&str> = docs.iter()
        .filter(|d| d.to_lowercase().contains(&args.query.to_lowercase()) || args.query.contains("radkit"))
        .cloned()
        .collect();

    let result = if relevant_docs.is_empty() {
        docs // Fallback to all docs if no match
    } else {
        relevant_docs
    };

    ToolResult::success(json!({
        "documents": result,
        "count": result.len()
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm = GeminiLlm::from_env("gemini-1.5-flash")?;

    let worker = LlmWorker::<Answer>::builder(llm)
        .with_system_instructions("You are a helpful assistant. Use the retrieve_docs tool to answer questions based on the provided context.")
        .with_tool(retrieve_docs)
        .build();

    let query = "What is Radkit?";
    println!("User: {}", query);

    let answer = worker.run(query).await?;

    println!("Answer: {}", answer.response);
    println!("Sources: {:?}", answer.sources);

    Ok(())
}
