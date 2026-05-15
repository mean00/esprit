include(${LN_MCU_FOLDER}/cmake/lnCmake_${LN_TOOLCHAIN_EXT}.cmake)
# ______________________________________

macro(LN_APPEND_FLAGS target)
  set_property(
    SOURCE ${target}
    APPEND_STRING
    PROPERTY COMPILE_OPTIONS ${ARGN})
  get_source_file_property(flags ${target} COMPILE_FLAGS)
endmacro(LN_APPEND_FLAGS)
# ______________________________________ This does not seem to work MACRO (LN_APPEND_IDIR target)
# GET_SOURCE_FILE_PROPERTY(flags ${target} INCLUDE_DIRECTORIES) MESSAGE(STATUS "aax ${target}=> ${flags} www")
# SET_PROPERTY( SOURCE ${target} PROPERTY INCLUDE_DIRECTORIES ${ARGN} APPEND ) GET_SOURCE_FILE_PROPERTY(flags ${target}
# INCLUDE_DIRECTORIES) MESSAGE(STATUS "aat ${target}=>${flags} www") ENDMACRO (LN_APPEND_IDIR)

macro(USE_LIBRARY lib)
  add_subdirectory(${ESPRIT_ROOT}/libraries/${lib})
  include_directories(${ESPRIT_ROOT}/libraries/${lib})
  list(APPEND USED_LIBS ${lib})
endmacro(USE_LIBRARY lib)
