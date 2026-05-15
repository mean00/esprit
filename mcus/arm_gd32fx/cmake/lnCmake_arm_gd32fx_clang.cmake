include(FindPython3)
include(ln_merge_libs)
macro(GENERATE_GD32_FIRMWARE target)
  ln_merge_libs()
  configure_file("${LN_MCU_FOLDER}/boards/bluepill/clang.lld.in" "${CMAKE_BINARY_DIR}/linker_script.ld" @ONLY)
  add_executable(${target} ${ARGN} ${LN_MCU_FOLDER}/src/start.S ${LN_MCU_FOLDER}/src/start.cpp
                           ${LN_MCU_FOLDER}/src/vector_table.S)
  # TARGET_LINK_LIBRARIES(${target} PUBLIC ${USED_LIBS} ) # duplicates are NOT a mistake ! duplicates are NOT a mistake
  # !
  target_link_libraries(${target} PUBLIC esprit_single_lib esprit_dev)
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
endmacro()
macro(HASH_GD32_FIRMWARE target)
  add_custom_command(
    TARGET ${target}
    POST_BUILD
    COMMAND ${CMAKE_OBJCOPY} -Obinary $<TARGET_FILE:${target}> $<TARGET_FILE:${target}>.tmp
    COMMAND python3 ${ESPRIT_ROOT}/script/lnCRC32_armChecksum.py $<TARGET_FILE:${target}>.tmp
            $<TARGET_FILE:${target}>.ck_bin
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Generating checksumed bin file")
endmacro()

include(ln_use_library)
