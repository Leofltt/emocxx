# emocxx-toolchain.cmake
# Toolchain script forwarding compiler invocations through the emocxx transpiler proxy.

if(DEFINED REAL_CXX)
    find_program(RESOLVED_REAL_CXX NAMES "${REAL_CXX}" REQUIRED)
elseif(DEFINED ENV{REAL_CXX})
    find_program(RESOLVED_REAL_CXX NAMES "$ENV{REAL_CXX}" REQUIRED)
elseif(DEFINED ENV{EMOCXX_REAL_CXX})
    find_program(RESOLVED_REAL_CXX NAMES "$ENV{EMOCXX_REAL_CXX}" REQUIRED)
else()
    find_program(RESOLVED_REAL_CXX NAMES clang++ g++ REQUIRED)
endif()

if(NOT DEFINED EMOCXX_BIN)
    find_program(EMOCXX_BIN NAMES emocxx PATHS "${CMAKE_CURRENT_LIST_DIR}/../target/release" "${CMAKE_CURRENT_LIST_DIR}/../target/debug" REQUIRED)
endif()

set(CMAKE_CXX_COMPILER "${EMOCXX_BIN}")
set(CMAKE_CXX_COMPILER_ARG1 "--real-cxx=${RESOLVED_REAL_CXX}")
set(CMAKE_CXX_COMPILER_FORCED TRUE)

