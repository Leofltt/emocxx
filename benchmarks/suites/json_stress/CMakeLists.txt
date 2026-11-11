cmake_minimum_required(VERSION 3.20)
project(json_stress_benchmark LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if (NOT DEFINED JSON_INCLUDE_DIR)
    get_filename_component(REPO_JSON_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../.benchmarks/repos/json/single_include" ABSOLUTE)
    set(JSON_INCLUDE_DIR "${REPO_JSON_DIR}")
endif()

if (NOT EXISTS "${JSON_INCLUDE_DIR}/nlohmann/json.hpp")
    message(FATAL_ERROR "Could not find nlohmann/json.hpp at ${JSON_INCLUDE_DIR}")
endif()

include_directories("${JSON_INCLUDE_DIR}")

add_executable(json_stress_suite
    src/json_parse.cpp
    src/json_serialize.cpp
)
set_target_properties(json_stress_suite PROPERTIES LINKER_LANGUAGE CXX)
