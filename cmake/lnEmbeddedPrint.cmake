set(EPR ${ESPRIT_CPP_ROOT}/embedded_printf/)
include_directories(${EPR})

foreach(common printf.c)
  list(APPEND EPS ${EPR}/${common})
endforeach(common printf.c)

if(LN_ENABLE_RUST AND (LN_EXT STREQUAL "arm_gd32fx" OR LN_EXT STREQUAL "riscv_ch32v3x" OR LN_EXT STREQUAL "arm_rp2040" OR LN_EXT STREQUAL "arm_rp2350"))
  add_library(embeddedPrintf INTERFACE)
  target_include_directories(esprit_dev INTERFACE ${EPR})
else()
  add_library(embeddedPrintf OBJECT ${EPS})
  target_include_directories(embeddedPrintf PRIVATE ${EPR})
  target_include_directories(esprit_dev INTERFACE ${EPR})
  target_compile_options(embeddedPrintf PRIVATE -DPRINTF_DISABLE_SUPPORT_DOUBLE -DPRINTF_DISABLE_SUPPORT_EXPONENTIAL
                                                -DPRINTF_DISABLE_SUPPORT_LONG_LONG -DPRINTF_DISABLE_SUPPORT_PTRDIFF_T)
  if(LN_ENABLE_FLOAT_PRINTF)

  else()
    target_compile_options(embeddedPrintf PRIVATE -DPRINTF_DISABLE_SUPPORT_FLOAT)
  endif()
endif()
