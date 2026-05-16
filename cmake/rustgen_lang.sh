#!/bin/bash
#set -x
#
if [ "$#" -lt 4 ]; then
  echo "Usage: rustgen_lang.sh header.h output.rs lang extra_dir [extra_dir2] [blocklist]"
  exit 1
fi
#input=$1
output=$2
ME=$0
export EXTRA_DIR=$4
export LANG=$3
export ME=$(realpath $ME)
export ME=$(dirname $ME)
export BINDGEN=~/.cargo/bin/bindgen
if [ -f "$BINDGEN" ]; then
  echo "."
else
  export BINDGEN="/usr/bin/bindgen"
fi
echo " bindgen is $BINDGEN ."
export LN=$(realpath $ME/..)

# Parse optional arguments
EXTRA_DIR2=""
BLOCKLIST=""
if [ "$#" -ge 5 ]; then
  if [ "$5" = "blocklist" ]; then
    BLOCKLIST="blocklist"
  else
    export EXTRA_DIR2=-I$5
  fi
fi
if [ "$#" -ge 6 ]; then
  if [ "$6" = "blocklist" ]; then
    BLOCKLIST="blocklist"
  fi
fi

export PLATFORM_CLANG_PATH=$(cmake -DUSE_CLANG=1 -DHOME=$LN -P $ME/toolchain_env.cmake 2>&1)
#export PLATFORM_TOOLCHAIN_PATH=/home/fx/Arduino_stm32/arm-gcc-2021q4/bin

export PATH=$PLATFORM_CLANG_PATH:$PATH

echo "** Processing $1 **"
echo "   using <$LN> as basedir **"
echo "   using <$PLATFORM_CLANG_PATH> as PLATFORM_CLANG_PATH **"
echo "$1 => $2"
export TMP=$1.tmp
echo "Tmp .h file = $TMP"
rm -f $TMP
echo "#ifdef __ARM_ARCH " >$TMP
echo "   #undef uint32_t " >>$TMP
echo "   #define _UINT32_T_DECLARED " >>$TMP
echo "   #define uint32_t unsigned int " >>$TMP
echo "#endif" >>$TMP
cat $1 >>$TMP
\rm -f rnEsprit.rs.tmp

# Build bindgen flags
BINDGEN_FLAGS=""
if [ "$BLOCKLIST" = "blocklist" ]; then
  BINDGEN_FLAGS="--blocklist-type lnPin --raw-line \"use crate::pin_types::lnPin;\""
fi

$BINDGEN \
  --rust-edition 2024 --rust-target="1.85" \
  --use-core --no-doc-comments \
  --no-layout-tests $TMP \
  --ctypes-prefix cty \
  --rustified-enum "ln.*" \
  ${BINDGEN_FLAGS} \
  -o $2.tmp \
  -- -x ${LANG} -DLN_ARCH=LN_ARCH_ARM \
  -funsigned-char \
  -I${EXTRA_DIR} \
  ${EXTRA_DIR2} \
  -I$LN -I$LN/include/ -I$LN/arduinoLayer/include/ \
  -I$LN/FreeRTOS/include/ \
  -I$LN/mcus/arm_gd32fx/boards/bluepill/ \
  -I$LN/mcus/arm_gd32fx/include/ \
  -I$LN/mcus/common_bluepill/ \
  -I$LN/legacy/boards/bluepill/ \
  -I$LN/FreeRTOS/portable/GCC/ARM_CM3/ \
  -target "thumbv7m-none-eabi" \
  -I${PLATFORM_CLANG_PATH}/../lib/clang-runtimes/arm-none-eabi/armv7m_soft_nofp/include/

#-I${PLATFORM_CLANG_PATH}/../arm-none-eabi/include/ \

cat header.rs.in >${output}
cat $2.tmp >>${output}
cat tail.rs.in >>${output}
\rm -f $2.tmp
#--whitelist-function  "^[lLx][nN].*|deadEnd|Logger" \
