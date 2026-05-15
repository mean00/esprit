set(EPR ${ESPRIT_ROOT}/embedded_printf/)
include_directories(${EPR})

foreach(common printf.c)
  list(APPEND EPS ${EPR}/${common})
endforeach(common printf.c)

add_library(embeddedPrintf OBJECT ${EPS})
target_include_directories(embeddedPrintf PRIVATE ${EPR})
target_include_directories(esprit_dev INTERFACE ${EPR})
target_compile_options(embeddedPrintf PRIVATE -DPRINTF_DISABLE_SUPPORT_DOUBLE -DPRINTF_DISABLE_SUPPORT_EXPONENTIAL
                                              -DPRINTF_DISABLE_SUPPORT_LONG_LONG -DPRINTF_DISABLE_SUPPORT_PTRDIFF_T)
if(LN_ENABLE_FLOAT_PRINTF)

else()
  target_compile_options(embeddedPrintf PRIVATE -DPRINTF_DISABLE_SUPPORT_FLOAT)
endif()
