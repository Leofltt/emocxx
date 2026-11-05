#include <boost/mp11.hpp>
#include <tuple>
#include <type_traits>

void run_asio_dummy();
void run_beast_dummy();
void run_spirit_dummy();

using namespace boost::mp11;

using Types = std::tuple<int, double, char, float, long, short, unsigned int>;
using Filtered = mp_copy_if<Types, std::is_integral>;
static_assert(mp_size<Filtered>::value == 5, "MP11 metaprogramming verification");

int main() {
    run_asio_dummy();
    run_beast_dummy();
    run_spirit_dummy();
    return 0;
}

