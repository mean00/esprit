#!/bin/bash
#
gen_cpp() {
  echo "__________________cpp:${1}_________________________"
  bash ../../../cmake/rustgen.sh ${1} ${2} ${3}
}

gen_c() {
  echo "__________________c:${1}_________________________"
  bash ../../../cmake/rustgen_c.sh ${1} ${2} ${3}
}
# Will work for CH32/STM32 and GD32
bash ../../../cmake/rustgen.sh lnGPIO_c.h ../src/rn_gpio_bp_c.rs ../../../mcus/common_bluepill/include #${PWD}
bash ../../../cmake/rustgen.sh lnGPIO_c.h ../src/rn_gpio_rp2040_c.rs ../../../mcus/arm_rp2040/include  #${PWD}
bash ../../../cmake/rustgen.sh lnGPIO_c.h ../src/rn_gpio_esp32_c.rs ../../../mcus/riscv_esp32/include  #${PWD}

gen_cpp lnTiming_adc_c.h ../src/rn_timing_adc_c.rs ${PWD}
gen_cpp lnTimer_c.h ../src/rn_timer_c.rs ../../../mcus/common_bluepill/include
gen_cpp lnExti_c.h ../src/rn_exti_c.rs ${PWD}
gen_c lnI2C_c.h ../src/rn_i2c_c.rs ${PWD}
gen_cpp lnSPI_c.h ../src/rn_spi_c.rs ${PWD}
gen_c lnCDC_c.h ../src/rn_cdc_c.rs ${PWD}
gen_c lnUSB_c.h ../src/rn_usb_c.rs ${PWD}
gen_c lnFast_EventGroup_c.h ../src/rn_fast_event_c.rs ${PWD} ../../../FreeRTOS
#gen_c lnFreeRTOS_c.h ../src/rn_freertos_c.rs ${PWD} ../../../FreeRTOS/
echo "__________________c:FreeRTOS_________________________"
bash ../../../cmake/rustgen.sh lnFreeRTOS_c.h ../src/rn_freertos_c.rs ../../../freertos_config/ ../../../FreeRTOS/
gen_cpp ../../../include/lnDebug.h ../src/rn_debug_c.rs ${PWD}
