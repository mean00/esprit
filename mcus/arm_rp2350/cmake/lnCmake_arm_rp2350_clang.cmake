INCLUDE(FindPython3)
include(ln_merge_libs)
#
#
#
MACRO(RP_PIO_GENERATE sourcefile target)
  ADD_CUSTOM_COMMAND(OUTPUT ${target}
                   DEPENDS ${sourcefile}
                   COMMAND pioasm ${sourcefile} ${target}
                   WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
                   COMMENT "Compiling pio asm... "
    )

ENDMACRO()
#
#
#
MACRO(GENERATE_GD32_FIRMWARE target)
  LN_MERGE_LIBS()
  IF(USE_RP2350_PURE_RAM)
    CONFIGURE_FILE("${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/rp2350_linker_ram.ld.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  ELSE()
    CONFIGURE_FILE("${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/rp2350_linker.ld.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  ENDIF()
  ADD_EXECUTABLE(${target} ${ARGN})
  TARGET_SOURCES(${target} PRIVATE ${LN_MCU_FOLDER}/conf/bs2_default_padded_checksummed.S)

  TARGET_LINK_LIBRARIES(${target} PUBLIC  esprit_single_lib esprit_dev)
  target_link_directories(${target} PUBLIC ${CMAKE_BINARY_DIR})
  IF(LN_CUSTOM_LD_SCRIPT)
    SET(SCRIPT ${LN_CUSTOM_LD_SCRIPT} CACHE INTERNAL "")
  ELSE()
    SET(SCRIPT "${CMAKE_BINARY_DIR}/linker_script.ld" CACHE INTERNAL "")
  ENDIF()
  TARGET_LINK_OPTIONS(${target}  PRIVATE "-T${SCRIPT}")
  #pico_set_linker_script( ${target} ${LD_SCRIPT})
  ADD_CUSTOM_COMMAND(TARGET ${target}
                   POST_BUILD
                   COMMAND ${CMAKE_OBJCOPY} -Obinary $<TARGET_FILE:${target}> $<TARGET_FILE:${target}>.bin
                   WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
                   COMMENT "Generating bin file"
    )
  ADD_CUSTOM_COMMAND(TARGET ${target}
                   POST_BUILD
                   COMMAND ${CMAKE_SIZE} --format=berkeley $<TARGET_FILE:${target}>
                   WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
                   COMMENT "Memory summary"
    )
  IF(USE_RP2350_PURE_RAM)
  ELSE()
    include(lnUf2)
    ADD_UF2(${target})
  ENDIF()
ENDMACRO()

include(ln_use_library)

MACRO(HASH_GD32_FIRMWARE target)
ENDMACRO()
