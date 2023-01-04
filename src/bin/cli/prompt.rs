use std::borrow::Cow;

use reedline::{Prompt, PromptEditMode, PromptViMode, PromptHistorySearchStatus};

pub struct BokhyllePrompt;

impl Prompt for BokhyllePrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        Cow::Owned("Bokhylle ".into())
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        Cow::Owned("".into())
    }

    fn render_prompt_indicator(
        &self,
        prompt_mode: reedline::PromptEditMode,
    ) -> std::borrow::Cow<str> {
        match prompt_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => "> ".into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => "> ".into(),
                PromptViMode::Insert => "> ".into(),
            },
            PromptEditMode::Custom(str) => format!("({})", str).into(),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        Cow::Borrowed(":> ".into())
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}
