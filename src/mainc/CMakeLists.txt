CMAKE_MINIMUM_REQUIRED(VERSION 2.8)
project(main-c)

# copy-pasta from https://docs.conan.io/en/latest/howtos/cmake_launch.html
if(NOT EXISTS "${CMAKE_BINARY_DIR}/conan.cmake")
    message(STATUS "Downloading conan.cmake from https://github.com/conan-io/cmake-conan")
    file(DOWNLOAD "https://raw.githubusercontent.com/conan-io/cmake-conan/master/conan.cmake"
            "${CMAKE_BINARY_DIR}/conan.cmake")
endif()

include(${CMAKE_BINARY_DIR}/conan.cmake)

conan_cmake_run(CONANFILE conanfile.txt
        BASIC_SETUP CMAKE_TARGETS
        BUILD missing)

# test using timerfd instead of timer wheel
#set(USING_TIMERFD ON CACHE "" INTERNAL FORCE)

include_directories(.)
add_library(mainc STATIC mainc.c)

target_link_libraries(mainc ${CONAN_LIBS})

install(TARGETS mainc DESTINATION .)
