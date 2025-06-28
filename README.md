 # esprit, simple but efficient embedded runtime


The aim of this project is to offer an Arduino style framework for the Longan Nano board (RISCV)or CH32V307 (RISCV) or GD32F1/F3 (ARM) boards.
It also works with STM32F1 chips.

The API is 99% the same between the Riscv & Arm versions. Mostly tiny pinout/clock/interrupt changes.

_What are the differences compared to vanilla Arduino ?_
* Cmake based build system, use whatever IDE you want.
* Support ARM and RISCV CH32V307/GD32 MCUs, same API (tested with GD32VF103, GD32F103,GD32F303, CH32F103 and STM32F103).
* FreeRTOS out of the box, using the GD32 RISCV port made by QQxiaoming [1]/ WCH RISCV Port /  vanilla ARM M3/M4
* Better peripherals support (DMA, tasks..)
* Only loosely compatible with Arduino API but easy to port
* Partial rust binding (not compatible with embedded_hal)

A note of warning : This is my take on what i would like to have/use for such a board
It might not fit your needs.


_Demos_
The demoProject contains simple API demonstration programs.

_How to build the demos :_
* Symlink/copy/git submodule the esprit folder under the demo folder
Fill-in platformconfig.cmake to point to your toolchain (adjust flags if needed)
mkdir build && cd build && cmake .. && make
* edit mcuSelect.Cmake to define which MCU you are using

By default they use a shared common_CMakeLists.txt / common_mcuSelect etc...

_Bonus:_

There is also some python helper scripts to have basic FreeRTOS support with gdb


_Heritage:_
* FreeRTOS from FreeRTOS
* Embedded_printf from [2]
* The riscv freeRTOS port is from QQxiaming (see [1])
* A couple of header file from arduino to offer a basic compatibility API
* A couple of header from nucleisys to access n200 extra functions
* TinyUsb for usb support
* Everything else has been written from scratch, no licensing issue.


The demoProject folder contains sample code to show the API.

[1] https://github.com/QQxiaoming/gd32vf103_freertos.git
[2] https://github.com/mpaland/printf


