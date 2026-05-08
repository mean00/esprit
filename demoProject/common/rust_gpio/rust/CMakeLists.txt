INCLUDE(rnCmake)
SET(LN_LTO_RUST_FLAGS ${LN_LTO_RUST_FLAGS} -C opt-level=s)  # we try to make arm code smaller, we dont care for hosted
RUST_ADD( rust_demo ${CMAKE_CURRENT_SOURCE_DIR}/rust_demo/Cargo.toml )


