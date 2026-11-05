//! Streaming FSM lexer for emocxx.
//!
//! Five explicit states:
//! 1. NORMAL: Scans tokens. Emits mangled ASCII identifiers for emoji grapheme clusters.
//! 2. IN_STRING: Preserves double-quoted string literals verbatim.
//! 3. IN_CHAR: Preserves single-quoted character literals verbatim.
//! 4. IN_RAW_STRING: Enters on R"delimiter( and preserves contents unmodified until )delimiter" is matched.
//! 5. IN_COMMENT: Preserves line (//) and block (/* ... */) comments without modification.

use crate::mangler::{is_emoji_grapheme, mangle_grapheme};
use std::path::Path;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq, Eq)]
pub enum LexerState {
    Normal,
    InString,
    InChar,
    InRawString(String),
    InLineComment,
    InBlockComment,
}

/// Transpiles C++ source code, converting emoji identifiers into mangled ASCII tokens
/// while preserving strings, raw strings, chars, and comments.
///
/// If `file_path` is provided, prepends `#line 1 "<absolute_path>"` for source transparency.
pub fn transpile_source(input: &str, file_path: Option<&Path>) -> String {
    let mut output = String::new();

    if let Some(path) = file_path {
        let abs_path = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf());
        let path_str = abs_path.to_string_lossy().replace('\\', "/");
        output.push_str(&format!("#line 1 \"{}\"\n", path_str));
    }

    let transpiled = transpile(input);
    output.push_str(&transpiled);
    output
}

/// Core transpiler running the 5-state lexer FSM.
pub fn transpile(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    let mut prev_char: Option<char> = None;

    while i < len {
        let b = bytes[i];

        // 0. Preprocessor include check: #include "..." or #include <...>
        // Rewrites emoji file paths to mangled ASCII so compilers never reject non-ASCII includes
        if b == b'#' {
            let slice = &input[i..];
            if let Some(rest) = slice.strip_prefix('#') {
                let trimmed = rest.trim_start();
                if trimmed.starts_with("include") {
                    let after_include = &trimmed["include".len()..];
                    if after_include.starts_with(' ') || after_include.starts_with('\t') {
                        let inc_trimmed = after_include.trim_start();
                        if inc_trimmed.starts_with('"') || inc_trimmed.starts_with('<') {
                            let open_delim = inc_trimmed.chars().next().unwrap();
                            let close_delim = if open_delim == '"' { '"' } else { '>' };
                            if let Some(close_pos) = inc_trimmed[1..].find(close_delim) {
                                let include_path = &inc_trimmed[1..=close_pos];
                                let mangled_path = crate::mangler::mangle_emojis_in_identifier(include_path);

                                let prefix_len = slice.len() - inc_trimmed.len();
                                let total_len = prefix_len + 1 + close_pos + 1;

                                out.push_str(&slice[..prefix_len]);
                                out.push(open_delim);
                                out.push_str(&mangled_path);
                                out.push(close_delim);

                                i += total_len;
                                prev_char = Some(close_delim);
                                continue;
                            }
                        }
                    }
                }
            }
        }

        // 1. Raw string literal check: R"delimiter(
        // Preceded by not an identifier char (or start of file / whitespace / punctuation)
        if b == b'R' && i + 1 < len && bytes[i + 1] == b'"' {
            let is_ident_prefix = prev_char.map_or(false, |c| c.is_ascii_alphanumeric() || c == '_');
            if !is_ident_prefix {
                // Check if this matches R"delim(
                if let Some((delim, start_content)) = parse_raw_string_delimiter(input, i + 2) {
                    out.push('R');
                    out.push('"');
                    out.push_str(&delim);
                    out.push('(');

                    i = start_content;
                    // Scan until )delim"
                    let closing_needle = format!("){}\"", delim);
                    if let Some(pos) = input[i..].find(&closing_needle) {
                        out.push_str(&input[i..i + pos]);
                        out.push_str(&closing_needle);
                        i += pos + closing_needle.len();
                        prev_char = Some('"');
                        continue;
                    } else {
                        // Unterminated raw string: consume rest
                        out.push_str(&input[i..]);
                        break;
                    }
                }
            }
        }

        // 2. Line comment: //
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            if let Some(pos) = input[i..].find('\n') {
                out.push_str(&input[i..=i + pos]);
                i += pos + 1;
                prev_char = Some('\n');
            } else {
                out.push_str(&input[i..]);
                i = len;
                prev_char = None;
            }
            continue;
        }

        // 3. Block comment: /* ... */
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            if let Some(pos) = input[i..].find("*/") {
                out.push_str(&input[i..i + pos + 2]);
                i += pos + 2;
                prev_char = Some('/');
            } else {
                out.push_str(&input[i..]);
                i = len;
                prev_char = None;
            }
            continue;
        }

        // 4. Double-quoted string literal: "..."
        if b == b'"' {
            out.push('"');
            i += 1;
            while i < len {
                let c = bytes[i];
                if c == b'\\' {
                    out.push('\\');
                    i += 1;
                    if i < len {
                        // Push escaped char (or UTF-8 continuation)
                        let ch = input[i..].chars().next().unwrap();
                        out.push(ch);
                        i += ch.len_utf8();
                    }
                    continue;
                }
                if c == b'"' {
                    out.push('"');
                    i += 1;
                    break;
                }
                let ch = input[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
            prev_char = Some('"');
            continue;
        }

        // 5. Single-quoted character literal: '...'
        if b == b'\'' {
            out.push('\'');
            i += 1;
            while i < len {
                let c = bytes[i];
                if c == b'\\' {
                    out.push('\\');
                    i += 1;
                    if i < len {
                        let ch = input[i..].chars().next().unwrap();
                        out.push(ch);
                        i += ch.len_utf8();
                    }
                    continue;
                }
                if c == b'\'' {
                    out.push('\'');
                    i += 1;
                    break;
                }
                let ch = input[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
            prev_char = Some('\'');
            continue;
        }

        // 6. Normal token: Check if non-ASCII / emoji grapheme cluster
        if b >= 0x80 {
            let slice = &input[i..];
            let grapheme = slice.graphemes(true).next().unwrap();
            if is_emoji_grapheme(grapheme) {
                out.push_str(&mangle_grapheme(grapheme));
            } else {
                out.push_str(grapheme);
            }
            i += grapheme.len();
            prev_char = grapheme.chars().last();
            continue;
        }

        // Standard ASCII character in NORMAL state
        let ch = b as char;
        out.push(ch);
        prev_char = Some(ch);
        i += 1;
    }

    out
}

/// Attempts to parse raw string delimiter starting after `R"`.
/// Returns (delimiter, index_after_left_paren) if valid.
fn parse_raw_string_delimiter(input: &str, start_pos: usize) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    let mut delim = String::new();
    let mut idx = start_pos;

    while idx < bytes.len() {
        let b = bytes[idx];
        if b == b'(' {
            return Some((delim, idx + 1));
        }
        // Delimiter max 16 chars and must not contain space, (, ), \, control chars
        if delim.len() >= 16 || b == b' ' || b == b')' || b == b'\\' || b <= 0x20 {
            return None;
        }
        delim.push(b as char);
        idx += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mangling_in_normal_code() {
        let src = "int 🚀 = 2;\nauto 💥 = 🎁.💰 * 🚀;";
        let expected = "int EMO_1f680_JI = 2;\nauto EMO_1f4a5_JI = EMO_1f381_JI.EMO_1f4b0_JI * EMO_1f680_JI;";
        assert_eq!(transpile(src), expected);
    }

    #[test]
    fn test_string_literal_preservation() {
        let src = r#"std::cout << "Rocket: 🚀" << " 🎯\n";"#;
        assert_eq!(transpile(src), src);
    }

    #[test]
    fn test_char_literal_preservation() {
        let src = "char c = '🚀';";
        assert_eq!(transpile(src), src);
    }

    #[test]
    fn test_raw_string_preservation() {
        let src = r#"const char* p = R"foo(Rocket: 🚀 and 💥)foo";"#;
        assert_eq!(transpile(src), src);

        let src_empty_delim = r#"const char* p = R"(Rocket: 🚀 and 💥)";"#;
        assert_eq!(transpile(src_empty_delim), src_empty_delim);
    }

    #[test]
    fn test_comment_preservation() {
        let src_line = "// This is a rocket: 🚀\nint a = 1;";
        assert_eq!(transpile(src_line), src_line);

        let src_block = "/* Launch 🚀 with score 🎯 */\nint b = 2;";
        assert_eq!(transpile(src_block), src_block);
    }

    #[test]
    fn test_acceptance_header_transpilation() {
        let header = "// include/engine.hpp\n#pragma once\nstruct 📦 {\n    int 💰;\n    int ⚖️;\n};\n";
        let expected = "// include/engine.hpp\n#pragma once\nstruct EMO_1f4e6_JI {\n    int EMO_1f4b0_JI;\n    int EMO_2696_fe0f_JI;\n};\n";
        assert_eq!(transpile(header), expected);
    }

    #[test]
    fn test_acceptance_source_transpilation() {
        let src = r#"// src/main.cpp
#include <iostream>
#include "engine.hpp"

int main() {
    📦 🎁{ .💰 = 500, .⚖️ = 20 };
    int 🚀 = 2;
    auto 💥 = 🎁.💰 * 🚀;

    std::cout << "Calculated Score: " << 💥 << " 🎯\n";
    return (💥 == 1000) ? 0 : 1;
}
"#;
        let expected = r#"// src/main.cpp
#include <iostream>
#include "engine.hpp"

int main() {
    EMO_1f4e6_JI EMO_1f381_JI{ .EMO_1f4b0_JI = 500, .EMO_2696_fe0f_JI = 20 };
    int EMO_1f680_JI = 2;
    auto EMO_1f4a5_JI = EMO_1f381_JI.EMO_1f4b0_JI * EMO_1f680_JI;

    std::cout << "Calculated Score: " << EMO_1f4a5_JI << " 🎯\n";
    return (EMO_1f4a5_JI == 1000) ? 0 : 1;
}
"#;
        assert_eq!(transpile(src), expected);
    }
}

