//! Unicode emoji and symbol mangling logic according to emocxx specification.
//!
//! Mangling schema: `_EMO_<codepoints_hex>_`
//! Examples:
//! - 🚀 (U+1F680) -> `_EMO_1f680_`
//! - 🏎️ (U+1F3CE, U+FE0F) -> `_EMO_1f3ce_fe0f_`
//! - 👍🏽 (U+1F44D, U+1F3FD) -> `_EMO_1f44d_1f3fd_`
//! - 👨‍💻 (U+1F468, U+200D, U+1F4BB) -> `_EMO_1f468_200d_1f4bb_`

use unicode_segmentation::UnicodeSegmentation;

/// Returns true if a Unicode codepoint belongs to an emoji, symbol, or variation selector range.
pub fn is_emoji_codepoint(c: char) -> bool {
    let u = c as u32;
    matches!(
        u,
        0x1F000..=0x1FAFF // Emoticons, Misc Symbols & Pictographs, Transport, Alphanumeric Supplement, Extended-A
        | 0x2600..=0x27BF // Miscellaneous Symbols (e.g. ⚖ U+2696), Dingbats
        | 0x2300..=0x23FF // Misc Technical (e.g. ⌚, ⌛, ⏩, ⏪)
        | 0x2B00..=0x2BFF // Misc Symbols and Arrows (e.g. ⭐, ⭕)
        | 0x200D          // Zero-Width Joiner (ZWJ)
        | 0xFE00..=0xFE0F // Variation Selectors (VS-1 to VS-16)
        | 0xE0100..=0xE01EF // Variation Selectors Supplement
        | 0x203C | 0x2049 // ‼️, ⁉️
        | 0x2122 | 0x2139 // ™, ℹ️
        | 0x2194..=0x21AA // Arrows
        | 0x25A0..=0x25FF // Geometric Shapes
        | 0x2934 | 0x2935 // ⤴, ⤵
        | 0x3030 | 0x303D // 〰, 〽
        | 0x3297 | 0x3299 // ㊗, ㊙
    )
}

/// Returns true if the grapheme cluster contains any emoji or symbol codepoints.
pub fn is_emoji_grapheme(grapheme: &str) -> bool {
    grapheme.chars().any(is_emoji_codepoint)
}

/// Mangles an emoji grapheme cluster into a C++ compliant ASCII identifier.
/// Format: `_EMO_<hex1>_<hex2>_..._` with lowercase hex.
pub fn mangle_grapheme(grapheme: &str) -> String {
    let mut mangled = String::from("_EMO_");
    let mut first = true;
    for c in grapheme.chars() {
        if !first {
            mangled.push('_');
        }
        mangled.push_str(&format!("{:x}", c as u32));
        first = false;
    }
    mangled.push('_');
    mangled
}

/// Helper function to segment a string into grapheme clusters and mangle emoji clusters.
pub fn mangle_emojis_in_identifier(token: &str) -> String {
    let mut result = String::new();
    for g in token.graphemes(true) {
        if is_emoji_grapheme(g) {
            result.push_str(&mangle_grapheme(g));
        } else {
            result.push_str(g);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_scalar_symbol() {
        assert_eq!(mangle_grapheme("🚀"), "_EMO_1f680_");
        assert_eq!(mangle_grapheme("📦"), "_EMO_1f4e6_");
        assert_eq!(mangle_grapheme("💰"), "_EMO_1f4b0_");
        assert_eq!(mangle_grapheme("💥"), "_EMO_1f4a5_");
        assert_eq!(mangle_grapheme("🎯"), "_EMO_1f3af_");
        assert_eq!(mangle_grapheme("🎁"), "_EMO_1f381_");
    }

    #[test]
    fn test_variation_selector_preserved() {
        // 🏎️ U+1F3CE, U+FE0F
        assert_eq!(mangle_grapheme("🏎️"), "_EMO_1f3ce_fe0f_");
        // ⚖️ U+2696, U+FE0F
        assert_eq!(mangle_grapheme("⚖️"), "_EMO_2696_fe0f_");
    }

    #[test]
    fn test_skin_tone_modifier() {
        // 👍🏽 U+1F44D, U+1F3FD
        assert_eq!(mangle_grapheme("👍🏽"), "_EMO_1f44d_1f3fd_");
    }

    #[test]
    fn test_zero_width_joiner_sequence() {
        // 👨‍💻 U+1F468, U+200D, U+1F4BB
        assert_eq!(mangle_grapheme("👨‍💻"), "_EMO_1f468_200d_1f4bb_");
    }
}

