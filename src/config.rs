use anyhow::Result;
use crossterm::style::Stylize;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
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

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            prefix: "[".into(),
            suffix: "]".into(),
            description: "".into(),
            separator: ", ".into(),
            style_prefix: StyleConfig::default(),
            style_suffix: StyleConfig::default(),
            style_description: StyleConfig::default(),
            style_separator: StyleConfig::default(),
            style_content: StyleConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub output_uuid: OutputConfig,
    pub output_timestamp: OutputConfig,
    pub output_author: OutputConfig,
    pub output_mood: OutputConfig,
    pub output_pace: OutputConfig,
    pub output_book: OutputConfig,
    pub output_genre: OutputConfig,
    pub output_edition: OutputConfig,
    pub output_progress: OutputConfig,
    pub output_language: OutputConfig,
    pub output_publisher: OutputConfig,
    pub output_edition_review: OutputConfig,
}

impl Config {
    pub fn default_as_string() -> Result<String> {
        Ok(toml::to_string(&Self::default())?)
    }
    pub fn read_config() -> Result<Self> {
        Ok(Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Toml::file("config.toml"))
            // .merge(Env::prefixed("APP_"))
            .extract()?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_uuid: OutputConfig {
                prefix: "(".into(),
                suffix: ")".into(),
                style_content: StyleConfig {
                    color: COLOR_DIMMED,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_timestamp: OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_TIMESTAMP,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_author: OutputConfig {
                description: "Written by:".into(),
                separator: " and ".into(),
                style_content: StyleConfig {
                    color: COLOR_AUTHOR,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_mood: OutputConfig {
                description: "Moods:".into(),
                style_content: StyleConfig {
                    color: COLOR_MOOD,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_pace: OutputConfig {
                description: "Pace:".into(),
                style_content: StyleConfig {
                    color: COLOR_PACE,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_book: OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_BOOK,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_genre: OutputConfig {
                description: "Genres:".into(),
                style_content: StyleConfig {
                    color: COLOR_GENRE,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_edition: OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_EDITION,
                    bold: true,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_progress: OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_EDITION,
                    bold: true,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_language: OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_LANGUAGE,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_publisher: OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_PUBLISHER,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_edition_review: OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_EDITION_REVIEW,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
        }
    }
}