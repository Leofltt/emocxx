#include <nlohmann/json.hpp>
#include <string>
#include <vector>

void test_serialize();

void test_parse() {
    std::string raw_data = R"({
        "project": "emocxx",
        "features": ["emoji_transpiler", "streaming_lexer", "shadow_cache"],
        "metrics": {
            "throughput_mb_s": 240.5,
            "latency_ms": 1.2,
            "nested": {
                "active": true,
                "count": 42
            }
        }
    })";

    auto parsed = nlohmann::json::parse(raw_data);
    volatile auto count = parsed["metrics"]["nested"]["count"].get<int>();
    (void)count;
}

int main() {
    test_parse();
    test_serialize();
    return 0;
}
