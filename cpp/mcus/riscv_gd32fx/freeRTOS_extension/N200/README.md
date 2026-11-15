this is a very slightly modified version of https://github.com/QQxiaoming/gd32vf103_freertos.git

Some information :
The taskSwitch is not done through a ECALL call but through a sw interrupted managed to the timer

The irq_entry function will do the save context/restore context stuff THEN call the vectored interrupt handler
The interrupt handler can do its thing without dealing with the freeRTOS context.