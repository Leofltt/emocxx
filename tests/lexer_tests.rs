use emocxx::lexer::{transpile, transpile_source};
use emocxx::mangler::mangle_grapheme;
use std::path::Path;

#[test]
fn test_mangling_single_scalars() {
    assert_eq!(mangle_grapheme("🚀"), "EMO_1f680_JI");
    assert_eq!(mangle_grapheme("📦"), "EMO_1f4e6_JI");
    assert_eq!(mangle_grapheme("💰"), "EMO_1f4b0_JI");
    assert_eq!(mangle_grapheme("💥"), "EMO_1f4a5_JI");
    assert_eq!(mangle_grapheme("🎯"), "EMO_1f3af_JI");
    assert_eq!(mangle_grapheme("🎁"), "EMO_1f381_JI");
}

#[test]
fn test_mangling_variation_selectors() {
    // 🏎️ U+1F3CE, U+FE0F
    assert_eq!(mangle_grapheme("🏎️"), "EMO_1f3ce_fe0f_JI");
    // ⚖️ U+2696, U+FE0F
    assert_eq!(mangle_grapheme("⚖️"), "EMO_2696_fe0f_JI");
}

#[test]
fn test_mangling_skin_tone_modifiers() {
    // 👍🏽 U+1F44D, U+1F3FD
    assert_eq!(mangle_grapheme("👍🏽"), "EMO_1f44d_1f3fd_JI");
}

#[test]
fn test_mangling_zero_width_joiner_sequences() {
    // 👨‍💻 U+1F468, U+200D, U+1F4BB
    assert_eq!(mangle_grapheme("👨‍💻"), "EMO_1f468_200d_1f4bb_JI");
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
        int EMO_1f680_JI = 42;
        int EMO_1f3ce_fe0f_JI = 100;
        auto EMO_1f4a5_JI = EMO_1f680_JI * EMO_1f3ce_fe0f_JI;
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
struct EMO_1f4e6_JI {
    int EMO_1f4b0_JI;
    int EMO_2696_fe0f_JI;
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
template<typename EMO_1f4e6_JI>
EMO_1f4e6_JI EMO_1f680_JI(EMO_1f4e6_JI x) {
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
struct EMO_1f4e6_JI {
    int EMO_1f4b0_JI;
    int EMO_2696_fe0f_JI;
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
    EMO_1f4e6_JI EMO_1f381_JI{ .EMO_1f4b0_JI = 500, .EMO_2696_fe0f_JI = 20 };
    int EMO_1f680_JI = 2;
    auto EMO_1f4a5_JI = EMO_1f381_JI.EMO_1f4b0_JI * EMO_1f680_JI;

    std::cout << "Calculated Score: " << EMO_1f4a5_JI << " 🎯\n";
    return (EMO_1f4a5_JI == 1000) ? 0 : 1;
}"#;
    assert_eq!(transpile(main_cpp), expected_main);
}

#[test]
fn test_line_directive_generation() {
    let code = "int 🚀 = 1;\n";
    let path = Path::new("test_src.cpp");
    let result = transpile_source(code, Some(path));
    assert!(result.starts_with("#line 1 "));
    assert!(result.contains("int EMO_1f680_JI = 1;\n"));
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
#include "EMO_1f4e6_JI.hpp"
#include <my_EMO_1f680_JI/engine.hpp>
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
class EMO_1f916_JI {
public:
    EMO_1f916_JI() : EMO_1f50b_JI(100) {}
    ~EMO_1f916_JI() {}
    void EMO_26a1_JI() { EMO_1f50b_JI += 10; }
private:
    int EMO_1f50b_JI;
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
namespace EMO_1f30c_JI {
    void EMO_1f680_JI() {}
}
int main() {
    EMO_1f30c_JI::EMO_1f680_JI();
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
enum class EMO_1f6a6_JI {
    EMO_1f534_JI,
    EMO_1f7e1_JI,
    EMO_1f7e2_JI,
};
auto state = EMO_1f6a6_JI::EMO_1f7e2_JI;
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
using EMO_1f522_JI = int;

template<typename EMO_1f4e6_JI>
concept EMO_1f4cf_JI = requires(EMO_1f4e6_JI item) {
    item.EMO_1f4d0_JI();
};

template<EMO_1f4cf_JI EMO_1f4e6_JI>
class EMO_1f3ed_JI {
    EMO_1f4e6_JI product;
};
    "#;
    assert_eq!(transpile(code), expected);
}

#[test]
fn test_user_defined_literals() {
    let code = r#"
constexpr long double operator""_🚗(long double km) {
    return km * 1000.0;
}
auto dist = 5.0_🚗;
    "#;
    let expected = r#"
constexpr long double operator""_EMO_1f697_JI(long double km) {
    return km * 1000.0;
}
auto dist = 5.0_EMO_1f697_JI;
    "#;
    assert_eq!(transpile(code), expected);
}

