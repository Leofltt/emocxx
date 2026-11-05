cmake_minimum_required(VERSION 3.20)
project(emocxx_acceptance_test LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(test_app
    src/main.cpp
)

target_include_directories(test_app PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Enable debug symbols for debugger validation
target_compile_options(test_app PRIVATE -g)

