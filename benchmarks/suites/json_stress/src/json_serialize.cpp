#include <nlohmann/json.hpp>
#include <map>
#include <string>
#include <vector>

void test_serialize() {
    nlohmann::json j;
    j["name"] = "emocxx_benchmark";
    j["tags"] = std::vector<std::string>{"c++20", "templates", "json", "metaprogramming"};
    j["mapping"] = std::map<std::string, int>{
        {"alpha", 1},
        {"beta", 2},
        {"gamma", 3}
    };

    std::string dump = j.dump(4);
    (void)dump;
}
