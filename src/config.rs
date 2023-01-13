use anyhow::Result;
use crossterm::style::Stylize;
use serde::{Deserialize, Serialize};

use crate::{default_colors::*, traits::DisplayTerminal};

#[derive(Debug, Serialize, Deserialize)]
pub struct StyleConfig {
    bold: bool,
    italic: bool,
    color: crossterm::style::Color,
}

impl StyleConfig {
    fn style(&self, s: impl ToString) -> String {
        let mut s = s.to_string().with(self.color);
        if self.bold {
            s = s.bold();
        }
        if self.italic {
            s = s.italic();
        }
        s.to_string()
    }
}

pub trait Styleable {
    fn style(&self, c: &StyleConfig) -> String;
}

impl<T> Styleable for T
where
    T: ToString + std::fmt::Display,
{
    fn style(&self, c: &StyleConfig) -> String {
        c.style(self)
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            color: COLOR_WHITE,
            bold: false,
            italic: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    pub prefix: String,
    pub suffix: String,
    pub description: String,
    pub separator: String,
    pub style_prefix: StyleConfig,
    pub style_suffix: StyleConfig,
    pub style_description: StyleConfig,
    pub style_separator: StyleConfig,
    pub style_content: StyleConfig,
}

impl OutputConfig {
    pub async fn format(
        &self,
        content: impl DisplayTerminal,
        conn: &sqlx::SqlitePool,
        config: &Config,
    ) -> Result<String> {
        let prefix = self.prefix.style(&self.style_prefix);
        let suffix = self.suffix.style(&self.style_suffix);
        let description = self.description.style(&self.style_description);
        // let content = content.style(&self.style_content);
        let content = &DisplayTerminal::fmt_to_string(&content, conn, Some(""), config).await?;
        Ok(format!("{prefix}{description} {content}{suffix}"))
    }
    pub async fn format_vec(
        &self,
        content: Vec<impl ToString + std::fmt::Display + DisplayTerminal>,
        conn: &sqlx::SqlitePool,
        config: &Config,
    ) -> Result<String> {
        let prefix = self.prefix.style(&self.style_prefix);
        let suffix = self.suffix.style(&self.style_suffix);
        let description = self.description.style(&self.style_description);
        let separator = self.separator.style(&self.style_separator);
        let mut s = format!("{prefix}{description} ");
        let mut i = content.into_iter().peekable();
        while let Some(x) = i.next() {
            s.push_str(&DisplayTerminal::fmt_to_string(&x, conn, Some(""), config).await?);
            if i.peek().is_some() {
                s.push_str(&separator);
            }
        }
        s.push_str(&suffix);
        Ok(s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub output_author: OutputConfig,
}

impl Config {
    pub fn default_as_string() -> Result<String> {
        Ok(toml::to_string(&Self::default())?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_author: OutputConfig {
                prefix: "[".into(),
                style_prefix: StyleConfig::default(),
                suffix: "]".into(),
                style_suffix: StyleConfig::default(),
                description: "written by".into(),
                style_description: StyleConfig::default(),
                separator: " and ".into(),
                style_separator: StyleConfig::default(),
                style_content: StyleConfig {
                    color: COLOR_AUTHOR,
                    ..StyleConfig::default()
                },
            },
        }
    }
}
