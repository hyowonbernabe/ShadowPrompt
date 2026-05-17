// Google Forms multi-step flow.

pub mod answered;
pub mod extractor;
pub mod injector;
pub mod submit_guard;

pub enum FormsMode {
    AutoPaginate,
    SinglePage,
}

pub async fn execute_form_flow(_mode: FormsMode) -> anyhow::Result<()> {
    // TODO: attach Chrome, loop pages, never click Submit, skip user-answered.
    Ok(())
}
