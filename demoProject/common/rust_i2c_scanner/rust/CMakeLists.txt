include(rnCmake)
set(LN_LTO_RUST_FLAGS ${LN_LTO_RUST_FLAGS} -C opt-level=s) # we try to make arm code smaller, we dont care for hosted
if(LN_MCU STREQUAL "RP2040" OR LN_MCU STREQUAL "RP2350")
  rust_add(rust_demo ${CMAKE_CURRENT_SOURCE_DIR}/rust_demo/Cargo.toml rp2040)
else()
  rust_add(rust_demo ${CMAKE_CURRENT_SOURCE_DIR}/rust_demo/Cargo.toml)
endif()
