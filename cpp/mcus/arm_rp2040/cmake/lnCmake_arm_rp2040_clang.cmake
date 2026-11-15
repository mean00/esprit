include(FindPython3)
include(ln_merge_libs)
#
macro(RP_PIO_GENERATE sourcefile target)
  add_custom_command(
    OUTPUT ${target}
    DEPENDS ${sourcefile}
    COMMAND pioasm ${sourcefile} ${target}
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Compiling pio asm... ")

endmacro()
#
macro(GENERATE_GD32_FIRMWARE target)

  ln_merge_libs()
  if(USE_RP2040_PURE_RAM)
    configure_file("${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/rp2040_linker_ram.ld.in"
                   "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  else()
    configure_file("${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/rp2040_linker.ld.in"
                   "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  endif()
  add_executable(${target} ${ARGN})
  target_sources(${target} PRIVATE ${LN_MCU_FOLDER}/conf/bs2_default_padded_checksummed.S)
  target_link_libraries(${target} PUBLIC esprit_dev esprit_single_lib) # duplicates are NOT a mistake !
  target_link_directories(${target} PUBLIC ${CMAKE_BINARY_DIR})
  if(LN_CUSTOM_LD_SCRIPT)
    set(SCRIPT
        ${LN_CUSTOM_LD_SCRIPT}
        CACHE INTERNAL "")
  else()
    set(SCRIPT
        "${CMAKE_BINARY_DIR}/linker_script.ld"
        CACHE INTERNAL "")
  endif()
  target_link_options(${target} PRIVATE "-T${SCRIPT}")
  # pico_set_linker_script( ${target} ${LD_SCRIPT})
  add_custom_command(
    TARGET ${target}
    POST_BUILD
    COMMAND ${CMAKE_OBJCOPY} -Obinary $<TARGET_FILE:${target}> $<TARGET_FILE:${target}>.bin
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Generating bin file")
  add_custom_command(
    TARGET ${target}
    POST_BUILD
    COMMAND ${CMAKE_SIZE} --format=berkeley $<TARGET_FILE:${target}>
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Memory summary")
  if(USE_RP2040_PURE_RAM)

  else()
    include(lnUf2)
    add_uf2(${target})
  endif()
endmacro(GENERATE_GD32_FIRMWARE target)

include(ln_use_library)

macro(HASH_GD32_FIRMWARE target)

endmacro()
