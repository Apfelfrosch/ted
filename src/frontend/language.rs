use std::process::Command;

use ratatui::style::Color;
use tree_sitter_highlight::HighlightConfiguration;

pub const HIGHLIGHTED_TOKENS: &[&str] = &["keyword", "string", "type"];

pub const HIGHLIGHT_THEME: &[(&str, Color)] = &[
    ("keyword", Color::Yellow),
    ("string", Color::Green),
    ("type", Color::Yellow),
];

pub fn get_highlight_color(token_type: &str) -> Option<Color> {
    for (k, c) in HIGHLIGHT_THEME.iter() {
        if *k == token_type {
            return Some(*c);
        }
    }
    None
}

#[derive(Copy, Clone)]
pub enum Language {
    Rust,
    C,
    Go,
}

impl Language {
    pub fn by_file_name(s: &str) -> Option<Language> {
        if s.ends_with(".rs") {
            Some(Language::Rust)
        } else if s.ends_with(".c") {
            Some(Language::C)
        } else if s.ends_with(".go") {
            Some(Language::Go)
        } else {
            None
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::C => "C",
            Language::Go => "Go",
        }
    }

    pub fn format_command(&self, p: &str) -> Option<Command> {
        match self {
            Language::C => None,
            Language::Go => {
                let mut cmd = Command::new("go");
                cmd.arg("fmt");
                cmd.arg(p);
                Some(cmd)
            }
            Language::Rust => {
                let mut cmd = Command::new("rustfmt");
                cmd.arg(p);
                Some(cmd)
            }
        }
    }

    pub fn build_highlighter_config(&self) -> Option<HighlightConfiguration> {
        let mut config = match self {
            Language::Rust => {
                let rust_language = tree_sitter_rust::language();
                HighlightConfiguration::new(
                    rust_language,
                    tree_sitter_rust::HIGHLIGHT_QUERY,
                    tree_sitter_rust::INJECTIONS_QUERY,
                    "",
                )
                .ok()?
            }
            Language::C => {
                let c_language = tree_sitter_c::language();

                HighlightConfiguration::new(c_language, tree_sitter_c::HIGHLIGHT_QUERY, "", "")
                    .ok()?
            }

            Language::Go => HighlightConfiguration::new(
                tree_sitter_go::language(),
                tree_sitter_go::HIGHLIGHT_QUERY,
                "",
                "",
            )
            .ok()?,
        };
        config.configure(HIGHLIGHTED_TOKENS);
        Some(config)
    }
}
