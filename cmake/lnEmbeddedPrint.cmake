SET(EPR ${ESPRIT_ROOT}/embedded_printf/)
include_directories( ${EPR})

FOREACH(common printf.c)
  LIST(APPEND EPS ${EPR}/${common})
ENDFOREACH(common printf.c)



ADD_LIBRARY( embeddedPrintf OBJECT ${EPS})
target_include_directories(embeddedPrintf PRIVATE ${EPR})
target_include_directories(esprit_dev INTERFACE ${EPR})
target_compile_options(embeddedPrintf PRIVATE  -DPRINTF_DISABLE_SUPPORT_FLOAT -DPRINTF_DISABLE_SUPPORT_DOUBLE -DPRINTF_DISABLE_SUPPORT_EXPONENTIAL -DPRINTF_DISABLE_SUPPORT_LONG_LONG -DPRINTF_DISABLE_SUPPORT_PTRDIFF_T)
