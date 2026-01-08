use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Input};
use radkit::agent::LlmWorker;
use radkit::models::providers::{{ provider_struct }};
use radkit::models::Thread;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize LLM
    let llm = {{ provider_struct }}::from_env("{{ default_model }}")?;
    println!("{} Initialized {}", style("✔").green(), style("{{ provider_name }}").cyan());

    // 2. Create the worker (Using Thread as output for raw text)
    // Note: radkit::models::Thread implements LLMOutputTrait (via tryparse::LlmDeserialize)
    // if `radkit` exports it correctly. However, `Thread` itself doesn't inherently imply
    // LlmDeserialize unless we derived it.
    //
    // Actually, looking at radkit source, Thread is a struct. It does NOT derive LLMOutput.
    // So we cannot use LlmWorker<Thread>.
    //
    // Instead, we will use a simple wrapper struct for the response.

    use radkit::macros::LLMOutput;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, JsonSchema, LLMOutput)]
    struct ChatResponse {
        message: String,
    }

    let worker = LlmWorker::<ChatResponse>::builder(llm)
        .with_system_instructions("You are a helpful AI assistant. Answer concisively.")
        .build();

    println!("{}", style("Interactive Chat (Type 'exit' to quit)").bold());
    println!("{}", style("----------------------------------------").dim());

    let mut thread = Thread::new();
    let term = Term::stdout();

    loop {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(style("You").bold().to_string())
            .interact_text()?;

        if input.trim().eq_ignore_ascii_case("exit") {
            break;
        }

        thread = thread.add_event(radkit::models::Event::user(input));

        term.write_line(&format!("{}", style("Assistant is thinking...").dim()))?;

        // Run the worker and get the new thread state back
        let (response, new_thread) = worker.run_and_continue(thread).await?;
        thread = new_thread;

        // Clear the "thinking" line
        term.move_cursor_up(1)?;
        term.clear_line()?;

        println!("{}: {}", style("Assistant").blue().bold(), response.message);
        println!();
    }

    Ok(())
}
