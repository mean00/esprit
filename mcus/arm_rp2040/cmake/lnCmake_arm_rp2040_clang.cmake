include(FindPython3)
include(ln_merge_libs)
#
#
#
MACRO(RP_PIO_GENERATE sourcefile target)
  add_custom_command(OUTPUT ${target}
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
  IF(USE_RP2040_PURE_RAM)
    configure_file( "${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/rp2040_linker_ram.ld.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  ELSE()
    configure_file( "${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/rp2040_linker.ld.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  ENDIF()
  ADD_EXECUTABLE(${target} ${ARGN}  )
  TARGET_LINK_LIBRARIES(${target} PUBLIC esprit_dev esprit_single_lib) # duplicates are NOT a mistake !
  IF(LN_CUSTOM_LD_SCRIPT)
    SET(SCRIPT ${LN_CUSTOM_LD_SCRIPT} CACHE INTERNAL "")
  ELSE()
    SET(SCRIPT "${CMAKE_BINARY_DIR}/linker_script.ld" CACHE INTERNAL "")
  ENDIF()
  TARGET_LINK_OPTIONS(${target}  PRIVATE "-T${SCRIPT}")
  #pico_set_linker_script( ${target} ${LD_SCRIPT})
  add_custom_command(TARGET ${target}
                   POST_BUILD
                   COMMAND ${CMAKE_OBJCOPY} -Obinary $<TARGET_FILE:${target}> $<TARGET_FILE:${target}>.bin
                   WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
                   COMMENT "Generating bin file"
    )
  add_custom_command(TARGET ${target}
                   POST_BUILD
                   COMMAND ${CMAKE_SIZE} --format=berkeley $<TARGET_FILE:${target}>
                   WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
                   COMMENT "Memory summary"
    )
  IF(USE_RP2040_PURE_RAM)
  ELSE()
    find_program ( elf2uf2      NAMES elf2uf2)
    if( "x${elf2uf2}" STREQUAL "xelf2uf2-NOTFOUND")
      MESSAGE(WARNING "elf2uf2 not found, it is part of the pico SDK")
    else()
      add_custom_command(TARGET ${target}
            POST_BUILD
            COMMAND elf2uf2  $<TARGET_FILE:${target}> $<TARGET_FILE:${target}>.uf2
            WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
            COMMENT "Generating uf2 files"
            )
    endif()
  ENDIF()
ENDMACRO(GENERATE_GD32_FIRMWARE target)

include(ln_use_library)

MACRO(HASH_GD32_FIRMWARE target)
ENDMACRO()
