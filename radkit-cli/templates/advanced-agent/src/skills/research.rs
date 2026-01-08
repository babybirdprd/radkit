use radkit::agent::{OnRequestResult, SkillHandler};
use radkit::errors::{AgentError, AgentResult};
use radkit::macros::skill;
use radkit::models::Content;
use radkit::runtime::context::{ProgressSender, State};
use radkit::runtime::AgentRuntime;
use async_trait::async_trait;

#[skill(
    id = "research",
    name = "Research Assistant",
    description = "Performs deep research on a topic",
    tags = ["research", "analysis"],
    examples = ["Research the history of Rust"],
    input_modes = ["text/plain"],
    output_modes = ["application/json"]
)]
pub struct ResearchSkill;

#[async_trait]
impl SkillHandler for ResearchSkill {
    async fn on_request(
        &self,
        _state: &mut State,
        progress: &ProgressSender,
        _runtime: &dyn AgentRuntime,
        content: Content,
    ) -> AgentResult<OnRequestResult> {
        let topic = content.as_text().unwrap_or_default();

        progress.send_update(format!("Starting research on: {}", topic)).await?;

        // Simulate some work
        progress.send_update("Searching knowledge base...").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        progress.send_update("Analyzing results...").await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let result_text = format!("Research complete for '{}'. Found 3 key sources.", topic);

        Ok(OnRequestResult::Completed {
            message: Some(Content::from_text(&result_text)),
            artifacts: vec![],
        })
    }
}
