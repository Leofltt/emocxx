# emocxx-toolchain.cmake
# Toolchain script forwarding compiler invocations through the emocxx transpiler proxy.

if(NOT DEFINED REAL_CXX)
    if(DEFINED ENV{REAL_CXX})
        set(REAL_CXX "$ENV{REAL_CXX}")
    elseif(DEFINED ENV{EMOCXX_REAL_CXX})
        set(REAL_CXX "$ENV{EMOCXX_REAL_CXX}")
    else()
        find_program(REAL_CXX NAMES clang++ g++ REQUIRED)
    endif()
endif()

if(NOT DEFINED EMOCXX_BIN)
    find_program(EMOCXX_BIN NAMES emocxx PATHS "${CMAKE_CURRENT_LIST_DIR}/../target/release" "${CMAKE_CURRENT_LIST_DIR}/../target/debug" REQUIRED)
endif()

set(CMAKE_CXX_COMPILER "${EMOCXX_BIN}")
set(CMAKE_CXX_COMPILER_ARG1 "--real-cxx=${REAL_CXX}")
set(CMAKE_CXX_COMPILER_FORCED TRUE)

