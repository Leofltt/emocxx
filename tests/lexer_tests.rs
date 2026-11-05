use emocxx::lexer::{transpile, transpile_source};
use emocxx::mangler::mangle_grapheme;
use std::path::Path;

#[test]
fn test_mangling_single_scalars() {
    assert_eq!(mangle_grapheme("🚀"), "_EMO_1f680_");
    assert_eq!(mangle_grapheme("📦"), "_EMO_1f4e6_");
    assert_eq!(mangle_grapheme("💰"), "_EMO_1f4b0_");
    assert_eq!(mangle_grapheme("💥"), "_EMO_1f4a5_");
    assert_eq!(mangle_grapheme("🎯"), "_EMO_1f3af_");
    assert_eq!(mangle_grapheme("🎁"), "_EMO_1f381_");
}

#[test]
fn test_mangling_variation_selectors() {
    // 🏎️ U+1F3CE, U+FE0F
    assert_eq!(mangle_grapheme("🏎️"), "_EMO_1f3ce_fe0f_");
    // ⚖️ U+2696, U+FE0F
    assert_eq!(mangle_grapheme("⚖️"), "_EMO_2696_fe0f_");
}

#[test]
fn test_mangling_skin_tone_modifiers() {
    // 👍🏽 U+1F44D, U+1F3FD
    assert_eq!(mangle_grapheme("👍🏽"), "_EMO_1f44d_1f3fd_");
}

#[test]
fn test_mangling_zero_width_joiner_sequences() {
    // 👨‍💻 U+1F468, U+200D, U+1F4BB
    assert_eq!(mangle_grapheme("👨‍💻"), "_EMO_1f468_200d_1f4bb_");
}

#[test]
fn test_string_literals_unmodified() {
    let code = r#"
        std::string s1 = "Rocket: 🚀";
        const char* s2 = "👨‍💻 in the zone: 🎯";
        std::string s3 = "Multiple emojis: 🚀🏎️👍🏽";
    "#;
    assert_eq!(transpile(code), code);
}

#[test]
fn test_char_literals_unmodified() {
    let code = "char a = '🚀'; char b = '🏎️';";
    assert_eq!(transpile(code), code);
}

#[test]
fn test_raw_string_literals_unmodified() {
    let code = r#"
        const char* raw1 = R"(This is a raw string with 🚀 and 💥)";
        const char* raw2 = R"custom(Another raw string with 👨‍💻 and 🎯)custom";
        const char* raw3 = R"delim(Multiline
with 📦 and 💰
)delim";
    "#;
    assert_eq!(transpile(code), code);
}

#[test]
fn test_comments_unmodified() {
    let code = r#"
        // Line comment with 🚀 and 💥
        /* Block comment with 👨‍💻 and 🎯 */
        /*
         * Multiline block comment
         * with 📦, 💰, and ⚖️
         */
        int a = 1;
    "#;
    assert_eq!(transpile(code), code);
}

#[test]
fn test_variable_declarations_and_expressions() {
    let code = r#"
        int 🚀 = 42;
        int 🏎️ = 100;
        auto 💥 = 🚀 * 🏎️;
    "#;
    let expected = r#"
        int _EMO_1f680_ = 42;
        int _EMO_1f3ce_fe0f_ = 100;
        auto _EMO_1f4a5_ = _EMO_1f680_ * _EMO_1f3ce_fe0f_;
    "#;
    assert_eq!(transpile(code), expected);
}

#[test]
fn test_struct_and_member_variables() {
    let header = r#"
#pragma once
struct 📦 {
    int 💰;
    int ⚖️;
};
"#;
    let expected = r#"
#pragma once
struct _EMO_1f4e6_ {
    int _EMO_1f4b0_;
    int _EMO_2696_fe0f_;
};
"#;
    assert_eq!(transpile(header), expected);
}

#[test]
fn test_function_and_template_declarations() {
    let code = r#"
template<typename 📦>
📦 🚀(📦 x) {
    return x;
}
"#;
    let expected = r#"
template<typename _EMO_1f4e6_>
_EMO_1f4e6_ _EMO_1f680_(_EMO_1f4e6_ x) {
    return x;
}
"#;
    assert_eq!(transpile(code), expected);
}

#[test]
fn test_acceptance_specification_end_to_end() {
    let header = r#"// include/engine.hpp
#pragma once
struct 📦 {
    int 💰;
    int ⚖️;
};"#;

    let expected_header = r#"// include/engine.hpp
#pragma once
struct _EMO_1f4e6_ {
    int _EMO_1f4b0_;
    int _EMO_2696_fe0f_;
};"#;
    assert_eq!(transpile(header), expected_header);

    let main_cpp = r#"// src/main.cpp
#include <iostream>
#include "engine.hpp"

int main() {
    📦 🎁{ .💰 = 500, .⚖️ = 20 };
    int 🚀 = 2;
    auto 💥 = 🎁.💰 * 🚀;

    std::cout << "Calculated Score: " << 💥 << " 🎯\n";
    return (💥 == 1000) ? 0 : 1;
}"#;

    let expected_main = r#"// src/main.cpp
#include <iostream>
#include "engine.hpp"

int main() {
    _EMO_1f4e6_ _EMO_1f381_{ ._EMO_1f4b0_ = 500, ._EMO_2696_fe0f_ = 20 };
    int _EMO_1f680_ = 2;
    auto _EMO_1f4a5_ = _EMO_1f381_._EMO_1f4b0_ * _EMO_1f680_;

    std::cout << "Calculated Score: " << _EMO_1f4a5_ << " 🎯\n";
    return (_EMO_1f4a5_ == 1000) ? 0 : 1;
}"#;
    assert_eq!(transpile(main_cpp), expected_main);
}

#[test]
fn test_line_directive_generation() {
    let code = "int 🚀 = 1;\n";
    let path = Path::new("test_src.cpp");
    let result = transpile_source(code, Some(path));
    assert!(result.starts_with("#line 1 "));
    assert!(result.contains("int _EMO_1f680_ = 1;\n"));
}

#[test]
fn test_emoji_in_include_directives() {
    let code = r#"
#include "📦.hpp"
#include <my_🚀/engine.hpp>
#include "engine.hpp"
#include <iostream>
"#;
    let expected = r#"
#include "_EMO_1f4e6_.hpp"
#include <my__EMO_1f680_/engine.hpp>
#include "engine.hpp"
#include <iostream>
"#;
    assert_eq!(transpile(code), expected);
}

#[test]
fn test_emoji_in_source_filename_line_directive() {
    let code = "int a = 1;\n";
    let path = Path::new("src/🚀.cpp");
    let result = transpile_source(code, Some(path));
    assert!(result.starts_with("#line 1 \""));
    assert!(result.contains("🚀.cpp\""));
}

#[test]
fn test_class_constructor_destructor_and_methods() {
    let code = r#"
class 🤖 {
public:
    🤖() : 🔋(100) {}
    ~🤖() {}
    void ⚡() { 🔋 += 10; }
private:
    int 🔋;
};
    "#;
    let expected = r#"
class _EMO_1f916_ {
public:
    _EMO_1f916_() : _EMO_1f50b_(100) {}
    ~_EMO_1f916_() {}
    void _EMO_26a1_() { _EMO_1f50b_ += 10; }
private:
    int _EMO_1f50b_;
};
    "#;
    assert_eq!(transpile(code), expected);
}

#[test]
fn test_namespaces_and_scoped_access() {
    let code = r#"
namespace 🌌 {
    void 🚀() {}
}
int main() {
    🌌::🚀();
}
    "#;
    let expected = r#"
namespace _EMO_1f30c_ {
    void _EMO_1f680_() {}
}
int main() {
    _EMO_1f30c_::_EMO_1f680_();
}
    "#;
    assert_eq!(transpile(code), expected);
}

#[test]
fn test_enums_and_enum_classes() {
    let code = r#"
enum class 🚦 {
    🔴,
    🟡,
    🟢,
};
auto state = 🚦::🟢;
    "#;
    let expected = r#"
enum class _EMO_1f6a6_ {
    _EMO_1f534_,
    _EMO_1f7e1_,
    _EMO_1f7e2_,
};
auto state = _EMO_1f6a6_::_EMO_1f7e2_;
    "#;
    assert_eq!(transpile(code), expected);
}

#[test]
fn test_templates_concepts_and_type_aliases() {
    let code = r#"
using 🔢 = int;

template<typename 📦>
concept 📏 = requires(📦 item) {
    item.📐();
};

template<📏 📦>
class 🏭 {
    📦 product;
};
    "#;
    let expected = r#"
using _EMO_1f522_ = int;

template<typename _EMO_1f4e6_>
concept _EMO_1f4cf_ = requires(_EMO_1f4e6_ item) {
    item._EMO_1f4d0_();
};

template<_EMO_1f4cf_ _EMO_1f4e6_>
class _EMO_1f3ed_ {
    _EMO_1f4e6_ product;
};
    "#;
    assert_eq!(transpile(code), expected);
}

#[test]
fn test_user_defined_literals() {
    let code = r#"
constexpr long double operator""_🚀(long double km) {
    return km * 1000.0;
}
auto dist = 5.0_🚀;
    "#;
    let expected = r#"
constexpr long double operator""__EMO_1f680_(long double km) {
    return km * 1000.0;
}
auto dist = 5.0__EMO_1f680_;
    "#;
    assert_eq!(transpile(code), expected);
}

