//! Smart code rendering with syntax highlighting
//!
//! Detects code in tool results and renders with appropriate highlighting.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Detect language from file extension
pub fn detect_language(file_path: &str) -> Option<Language> {
    let extension = file_path.split('.').next_back()?;
    match extension {
        "rs" => Some(Language::Rust),
        "js" | "jsx" | "ts" | "tsx" => Some(Language::JavaScript),
        "py" => Some(Language::Python),
        "go" => Some(Language::Go),
        "java" => Some(Language::Java),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
        "json" => Some(Language::Json),
        "toml" => Some(Language::Toml),
        "yaml" | "yml" => Some(Language::Yaml),
        "md" => Some(Language::Markdown),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Language {
    Rust,
    JavaScript,
    Python,
    Go,
    Java,
    C,
    Cpp,
    Json,
    Toml,
    Yaml,
    Markdown,
}

impl Language {
    /// Get keywords for this language
    fn keywords(&self) -> &[&str] {
        match self {
            Language::Rust => &[
                "fn", "let", "mut", "const", "static", "impl", "trait", "struct", "enum",
                "pub", "use", "mod", "crate", "super", "self", "if", "else", "match",
                "for", "while", "loop", "break", "continue", "return", "async", "await",
            ],
            Language::JavaScript => &[
                "function", "const", "let", "var", "if", "else", "for", "while", "return",
                "class", "extends", "import", "export", "async", "await", "try", "catch",
            ],
            Language::Python => &[
                "def", "class", "if", "elif", "else", "for", "while", "return", "import",
                "from", "as", "try", "except", "with", "lambda", "async", "await",
            ],
            Language::Go => &[
                "func", "var", "const", "if", "else", "for", "range", "return", "struct",
                "interface", "type", "package", "import", "defer", "go", "chan",
            ],
            _ => &[],
        }
    }

    /// Get types for this language
    fn types(&self) -> &[&str] {
        match self {
            Language::Rust => &["String", "str", "i32", "i64", "u32", "u64", "f32", "f64", "bool", "usize", "Option", "Result", "Vec"],
            Language::JavaScript | Language::Python => &["string", "number", "boolean", "object", "array"],
            Language::Go => &["string", "int", "int64", "float64", "bool", "map", "slice"],
            _ => &[],
        }
    }
}

/// Render code with simple syntax highlighting
pub fn highlight_code(code: &str, language: Option<Language>) -> Vec<Line<'static>> {
    let mut lines = vec![];

    for line in code.lines() {
        if let Some(lang) = language {
            lines.push(highlight_line(line, lang));
        } else {
            lines.push(Line::from(line.to_string()));
        }
    }

    lines
}

/// Highlight a single line of code
fn highlight_line(line: &str, language: Language) -> Line<'static> {
    let keywords = language.keywords();
    let types = language.types();

    let mut spans = vec![];
    let mut current = String::new();
    let mut in_string = false;
    let mut string_char = ' ';

    for ch in line.chars() {
        // Handle string literals
        if ch == '"' || ch == '\'' {
            if !in_string {
                // Flush current word
                if !current.is_empty() {
                    spans.push(word_to_span(&current, keywords, types));
                    current.clear();
                }
                in_string = true;
                string_char = ch;
                current.push(ch);
            } else if ch == string_char {
                current.push(ch);
                spans.push(Span::styled(current.clone(), Style::default().fg(Color::Green)));
                current.clear();
                in_string = false;
            } else {
                current.push(ch);
            }
        } else if in_string {
            current.push(ch);
        } else if ch.is_whitespace() || ch == '(' || ch == ')' || ch == '{' || ch == '}' || ch == '[' || ch == ']' || ch == ';' || ch == ',' || ch == ':' {
            // Flush current word
            if !current.is_empty() {
                spans.push(word_to_span(&current, keywords, types));
                current.clear();
            }
            spans.push(Span::raw(ch.to_string()));
        } else {
            current.push(ch);
        }
    }

    // Flush remaining
    if !current.is_empty() {
        if in_string {
            spans.push(Span::styled(current, Style::default().fg(Color::Green)));
        } else {
            spans.push(word_to_span(&current, keywords, types));
        }
    }

    Line::from(spans)
}

/// Convert a word to a styled span
fn word_to_span(word: &str, keywords: &[&str], types: &[&str]) -> Span<'static> {
    if keywords.contains(&word) {
        Span::styled(word.to_string(), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
    } else if types.contains(&word) {
        Span::styled(word.to_string(), Style::default().fg(Color::Cyan))
    } else if word.starts_with("//") {
        Span::styled(word.to_string(), Style::default().fg(Color::DarkGray))
    } else if word.chars().all(|c| c.is_numeric() || c == '.') {
        Span::styled(word.to_string(), Style::default().fg(Color::Yellow))
    } else {
        Span::raw(word.to_string())
    }
}

/// Parse Edit tool result JSON and render with code highlighting
pub fn render_edit_result(content: &str) -> Option<Vec<Line<'static>>> {
    // Try to parse as JSON
    let json: serde_json::Value = serde_json::from_str(content).ok()?;

    let mut lines = vec![];

    // Extract file path
    if let Some(file_path) = json.get("file_path").and_then(|v| v.as_str()) {
        lines.push(Line::from(vec![
            Span::styled(" File: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),  // nf-fa-file_code
            Span::styled(file_path.to_string(), Style::default().fg(Color::Yellow)),
        ]));
        lines.push(Line::from(""));

        let language = detect_language(file_path);

        // Show old code if present
        if let Some(old_string) = json.get("old_string").and_then(|v| v.as_str()) {
            lines.push(Line::from(Span::styled(
                " Removed:",  // nf-fa-minus_circle
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            for line in highlight_code(old_string, language) {
                let mut spans = vec![Span::styled("  - ", Style::default().fg(Color::Red))];
                spans.extend(line.spans);
                lines.push(Line::from(spans));
            }
            lines.push(Line::from(""));
        }

        // Show new code if present
        if let Some(new_string) = json.get("new_string").and_then(|v| v.as_str()) {
            lines.push(Line::from(Span::styled(
                " Added:",  // nf-fa-plus_circle
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            for line in highlight_code(new_string, language) {
                let mut spans = vec![Span::styled("  + ", Style::default().fg(Color::Green))];
                spans.extend(line.spans);
                lines.push(Line::from(spans));
            }
        }

        return Some(lines);
    }

    None
}
