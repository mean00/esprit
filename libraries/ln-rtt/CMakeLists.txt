ADD_LIBRARY(ln-rtt STATIC)
SET(S ${CMAKE_CURRENT_SOURCE_DIR})
TARGET_SOURCES(ln-rtt PRIVATE
    ${S}/src/LN_RTT.cpp
    ${S}/src/LN_RTT_var.cpp
)
TARGET_INCLUDE_DIRECTORIES(ln-rtt PUBLIC ${S}/include)
TARGET_INCLUDE_DIRECTORIES(ln-rtt PRIVATE ${S}/private_include)
TARGET_INCLUDE_DIRECTORIES(esprit_dev INTERFACE ${S}/include)
target_link_libraries(ln-rtt PRIVATE esprit_dev)
