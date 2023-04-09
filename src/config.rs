use std::path::PathBuf;

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
    bold:   bool,
    italic: bool,
    color:  crossterm::style::Color,
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
            color:  COLOR_WHITE,
            bold:   false,
            italic: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    pub display_uuid:      bool,
    pub prefix:            String,
    pub suffix:            String,
    pub description:       String,
    pub separator:         String,
    pub style_prefix:      StyleConfig,
    pub style_suffix:      StyleConfig,
    pub style_description: StyleConfig,
    pub style_separator:   StyleConfig,
    pub style_content:     StyleConfig,
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

    pub async fn format_str(
        &self,
        content: impl ToString,
        _conn: &sqlx::SqlitePool,
        _config: &Config,
    ) -> Result<String> {
        let prefix = self.prefix.style(&self.style_prefix);
        let suffix = self.suffix.style(&self.style_suffix);
        let description = self.description.style(&self.style_description);
        let content = content.to_string().style(&self.style_content);
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
            display_uuid:      false,
            prefix:            "[".into(),
            suffix:            "]".into(),
            description:       "".into(),
            separator:         ", ".into(),
            style_prefix:      StyleConfig::default(),
            style_suffix:      StyleConfig::default(),
            style_description: StyleConfig {
                italic: true,
                ..StyleConfig::default()
            },
            style_separator:   StyleConfig::default(),
            style_content:     StyleConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_location:        std::path::PathBuf,
    pub output_uuid:              OutputConfig,
    pub output_timestamp:         OutputConfig,
    pub output_author:            OutputConfig,
    pub output_mood:              OutputConfig,
    pub output_pace:              OutputConfig,
    pub output_book:              OutputConfig,
    pub output_genre:             OutputConfig,
    pub output_edition:           OutputConfig,
    pub output_progress:          OutputConfig,
    pub output_language:          OutputConfig,
    pub output_publisher:         OutputConfig,
    pub output_edition_review:    OutputConfig,
    pub output_rating:            OutputConfig,
    pub output_recommended_true:  OutputConfig,
    pub output_recommended_false: OutputConfig,
    pub output_last_updated:      OutputConfig,
    pub output_page_count:        OutputConfig,
    pub output_release_date:      OutputConfig,
    pub output_series:            OutputConfig,
    pub output_review:            OutputConfig,
    pub output_isbn:              OutputConfig,
    pub output_format:            OutputConfig,
    pub output_binding:           OutputConfig,
    pub output_dimensions:        OutputConfig,
    pub output_price:             OutputConfig,
    pub output_part_index:        OutputConfig,
    pub output_error:             OutputConfig,
}

impl Config {
    pub fn default_as_string() -> Result<String> {
        Ok(toml::to_string(&Self::default())?)
    }

    pub fn read_config() -> Result<Self> {
        Ok(Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("TOMEX_"))
            .extract()?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_location:        PathBuf::from("~/.local/share/tomex/database"),
            output_uuid:              OutputConfig {
                prefix: "(".into(),
                suffix: ")".into(),
                style_content: StyleConfig {
                    color: COLOR_DIMMED,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_timestamp:         OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_TIMESTAMP,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_author:            OutputConfig {
                description: "Written by:".into(),
                separator: " and ".into(),
                style_content: StyleConfig {
                    color: COLOR_AUTHOR,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_mood:              OutputConfig {
                description: "Moods:".into(),
                style_content: StyleConfig {
                    color: COLOR_MOOD,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_pace:              OutputConfig {
                description: "Pace:".into(),
                style_content: StyleConfig {
                    color: COLOR_PACE,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_book:              OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_BOOK,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_genre:             OutputConfig {
                description: "Genres:".into(),
                style_content: StyleConfig {
                    color: COLOR_GENRE,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_edition:           OutputConfig {
                display_uuid: true,
                style_content: StyleConfig {
                    color: COLOR_EDITION,
                    bold: true,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_progress:          OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_EDITION,
                    bold: true,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_language:          OutputConfig {
                description: "Written in:".into(),
                separator: " and ".into(),
                style_content: StyleConfig {
                    color: COLOR_LANGUAGE,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_publisher:         OutputConfig {
                description: "Publisher:".into(),
                style_content: StyleConfig {
                    color: COLOR_PUBLISHER,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_edition_review:    OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_EDITION_REVIEW,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_rating:            OutputConfig {
                description: "Rating:".into(),
                style_content: StyleConfig {
                    color: COLOR_RATING,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_recommended_true:  OutputConfig {
                description: "Recommended:".into(),
                style_content: StyleConfig {
                    bold: true,
                    color: COLOR_RECOMMENDED_TRUE,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_recommended_false: OutputConfig {
                description: "Recommended:".into(),
                style_content: StyleConfig {
                    bold: true,
                    color: COLOR_RECOMMENDED_FALSE,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_last_updated:      OutputConfig {
                description: "Last updated:".into(),
                style_description: StyleConfig {
                    italic: true,
                    ..StyleConfig::default()
                },
                style_content: StyleConfig {
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_page_count:        OutputConfig {
                description: "Page count:".into(),
                style_description: StyleConfig {
                    italic: true,
                    ..StyleConfig::default()
                },
                style_content: StyleConfig {
                    color: COLOR_PAGE_COUNT,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_release_date:      OutputConfig {
                description: "Released:".into(),
                style_description: StyleConfig {
                    italic: true,
                    ..StyleConfig::default()
                },
                style_content: StyleConfig::default(),
                ..OutputConfig::default()
            },
            output_series:            OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_SERIES,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_review:            OutputConfig {
                style_content: StyleConfig {
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_isbn:              OutputConfig {
                style_content: StyleConfig {
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_format:            OutputConfig {
                description: "Format:".into(),
                style_content: StyleConfig {
                    color: COLOR_FORMAT,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_binding:           OutputConfig {
                description: "Binding:".into(),
                style_content: StyleConfig {
                    color: COLOR_BINDING,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_dimensions:        OutputConfig {
                style_content: StyleConfig {
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_price:             OutputConfig {
                style_content: StyleConfig {
                    color: COLOR_PRICE,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_part_index:        OutputConfig {
                description: "Volume".into(),
                style_content: StyleConfig {
                    color: COLOR_PART_INDEX,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
            output_error:             OutputConfig {
                description: "Error".into(),
                style_content: StyleConfig {
                    color: COLOR_ERROR,
                    ..StyleConfig::default()
                },
                ..OutputConfig::default()
            },
        }
    }
}
