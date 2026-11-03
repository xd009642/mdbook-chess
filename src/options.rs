use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Orientation {
    #[default]
    White,
    Black,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    Classic,
    Green,
    Blue,
    Monochrome,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Colors {
    pub light: Option<String>,
    pub dark: Option<String>,
    pub highlight: Option<String>,
    pub arrow: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Annotation {
    Brilliant,
    Good,
    Interesting,
    Dubious,
    Mistake,
    Blunder,
}

impl Annotation {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Brilliant => "!!",
            Self::Good => "!",
            Self::Interesting => "!?",
            Self::Dubious => "?!",
            Self::Mistake => "?",
            Self::Blunder => "??",
        }
    }
    pub fn color(&self) -> &'static str {
        match self {
            Self::Brilliant => "#087e8b",
            Self::Good => "#287344",
            Self::Interesting => "#5b5ca8",
            Self::Dubious => "#866414",
            Self::Mistake => "#a65316",
            Self::Blunder => "#b52d39",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Appearance {
    pub orientation: Orientation,
    pub coordinates: bool,
    pub theme: Theme,
    pub colors: Colors,
    pub size: u16,
    pub align: Align,
    pub caption: Option<String>,
    pub alt: Option<String>,
    pub annotations: BTreeMap<String, Annotation>,
    pub last_move: bool,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            orientation: Orientation::White,
            coordinates: false,
            theme: Theme::Classic,
            colors: Colors::default(),
            size: 302,
            align: Align::Left,
            caption: None,
            alt: None,
            annotations: BTreeMap::new(),
            last_move: false,
        }
    }
}

impl Appearance {
    pub fn validate(&self) -> Result<()> {
        if !(64..=2048).contains(&self.size) {
            bail!("size must be between 64 and 2048 pixels");
        }
        for color in [
            &self.colors.light,
            &self.colors.dark,
            &self.colors.highlight,
            &self.colors.arrow,
        ]
        .into_iter()
        .flatten()
        {
            if color.len() != 7
                || !color.starts_with('#')
                || !color[1..].bytes().all(|b| b.is_ascii_hexdigit())
            {
                bail!("color '{color}' must use #RRGGBB");
            }
        }
        for square in self.annotations.keys() {
            parse_square(square)?;
        }
        Ok(())
    }

    pub fn palette(&self) -> (&str, &str, &str, &str) {
        let (light, dark) = match self.theme {
            Theme::Classic => ("#ffce9e", "#d18b47"),
            Theme::Green => ("#eeeed2", "#769656"),
            Theme::Blue => ("#dee6ed", "#7392ac"),
            Theme::Monochrome => ("#f0f0f0", "#999999"),
        };
        (
            self.colors.light.as_deref().unwrap_or(light),
            self.colors.dark.as_deref().unwrap_or(dark),
            self.colors.highlight.as_deref().unwrap_or("#ff0000"),
            self.colors.arrow.as_deref().unwrap_or("#008a00"),
        )
    }
}

/// chess::Square's parser accepts trailing text; diagram fields require a complete square.
pub fn parse_square(value: &str) -> Result<chess::Square> {
    if value.len() != 2 || !value.is_ascii() {
        bail!("invalid square '{value}'");
    }
    value
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid square '{value}': {e}"))
}
