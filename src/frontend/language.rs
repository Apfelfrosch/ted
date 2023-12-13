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

    pub fn build_highlighter_config(&self) -> Option<HighlightConfiguration> {
        let mut config = match self {
            Language::Rust => {
                let rust_language = tree_sitter_rust::language();
                let rust_config = HighlightConfiguration::new(
                    rust_language,
                    tree_sitter_rust::HIGHLIGHT_QUERY,
                    tree_sitter_rust::INJECTIONS_QUERY,
                    "",
                )
                .ok()?;
                rust_config
            }
            Language::C => {
                let c_language = tree_sitter_c::language();
                let c_config =
                    HighlightConfiguration::new(c_language, tree_sitter_c::HIGHLIGHT_QUERY, "", "")
                        .ok()?;
                c_config
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