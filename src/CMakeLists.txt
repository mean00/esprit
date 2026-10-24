#

add_library(ln_utils OBJECT)
target_sources(ln_utils PRIVATE lnFreeRTOS_pp.cpp lnFreeRTOSDebug.cpp systemHelper.cpp lnStopWatch.cpp)
# The logger has a Rust twin with the same C ABI (rust_esprit/src/logger_c.rs,
# CMake option USE_RUST_LOGGER): exactly one of the two provides Logger(),
# Logger_C(), Logger_chars(), setLogger(), LoggerInit(), Logger_crash() and
# lnScratchRegister.
if(NOT USE_RUST_LOGGER)
  target_sources(ln_utils PRIVATE lnDebug.cpp)
endif()
# The static storage of the FreeRTOS wrappers (see lnFreeRTOS_pp.h) lives inside
# the wrapped object, so it is allocated where that object is declared.  This
# translation unit keeps -fdata-sections anyway: its own statics (and the
# freeRTOSDebug reference) then get one .bss section each and --gc-sections can
# drop the ones a firmware never uses; without it the whole .bss of this object
# would be kept.
set_source_files_properties(lnFreeRTOS_pp.cpp PROPERTIES COMPILE_OPTIONS "-fdata-sections")
# Millisecond timebase: tick-based default, or an external (per-MCU) one
# selected via LN_EXTERNAL_SYSTEM_HELPER (e.g. ESP-IDF's esp_timer-based
# lnSystemTime_esp32.cpp in mcus/riscv_esp32/src/).
if(NOT LN_EXTERNAL_SYSTEM_HELPER)
  target_sources(ln_utils PRIVATE lnSystemTime.cpp)
endif()
target_link_libraries(ln_utils PRIVATE esprit_dev)
if(LN_EXTERNAL_MALLOC)

else()
  target_sources(ln_utils PRIVATE systemAlloc.cpp ln_libc.cpp)
endif()

if(LN_ENABLE_ETH)
  target_sources(ln_utils PRIVATE lnSocketRunner.cpp)
endif()

if(LN_DEBUG_UART)
  message(STATUS "Using uart ${LN_DEBUG_UART} for debug")
  target_compile_definitions(ln_utils PRIVATE LN_DEBUG_UART=${LN_DEBUG_UART})
endif()
