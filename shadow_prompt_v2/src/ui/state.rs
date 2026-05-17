// Centralized UI state, owned by the UI thread, mutated only in response to UICommand.

pub struct UiState {
    pub hidden: bool,
    pub indicator: super::commands::IndicatorState,
    pub form_indicator: super::commands::FormIndicatorState,
    pub overlay_text: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            hidden: false,
            indicator: super::commands::IndicatorState::Ready,
            form_indicator: super::commands::FormIndicatorState::Hidden,
            overlay_text: None,
        }
    }
}
