
SET(SRCS nvmCore.cpp gd32/nvm_gd32.cpp )
include_directories(${CMAKE_CURRENT_SOURCE_DIR} ${CMAKE_CURRENT_SOURCE_DIR}/gd32)
add_library(NeVerMind STATIC ${SRCS})
target_include_directories(NeVerMind PUBLIC ${CMAKE_CURRENT_SOURCE_DIR} ${CMAKE_CURRENT_SOURCE_DIR}/gd32)
target_link_libraries(NeVerMind PRIVATE esprit_dev)
target_include_directories(esprit_dev INTERFACE ${CMAKE_CURRENT_SOURCE_DIR} ${CMAKE_CURRENT_SOURCE_DIR}/gd32)
