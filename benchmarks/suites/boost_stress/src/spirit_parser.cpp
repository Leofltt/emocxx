#include <boost/spirit/home/x3.hpp>
#include <string>
#include <vector>

namespace x3 = boost::spirit::x3;

void run_spirit_dummy() {
    std::string input = "123, 456, 789";
    std::vector<int> numbers;
    auto iter = input.begin();
    auto end = input.end();
    x3::phrase_parse(iter, end, x3::int_ % ',', x3::space, numbers);
}

