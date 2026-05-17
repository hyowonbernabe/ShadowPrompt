// Inject answers from the LLM back into form DOM. Never touches already-answered
// questions. Never clicks Submit.

use std::collections::HashMap;

pub async fn inject_answers(_answers: &HashMap<String, String>) -> anyhow::Result<()> {
    Ok(())
}
