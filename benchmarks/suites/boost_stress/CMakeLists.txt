cmake_minimum_required(VERSION 3.20)
project(boost_stress_benchmark LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Boost REQUIRED)

add_executable(boost_stress_suite
    src/asio_server.cpp
    src/beast_http.cpp
    src/spirit_parser.cpp
    src/mp11_metaprog.cpp
)

target_link_libraries(boost_stress_suite PRIVATE Boost::headers)

