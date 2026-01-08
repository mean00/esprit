/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
/**
 */
class lnCpuID
{
  public:
    enum LN_VENDOR
    {
        LN_VENDOR_UNKOWN,
        LN_VENDOR_STM,
        LN_VENDOR_GD,
        LN_VENDOR_WCH,
        LN_VENDOR_ESP
    };
    enum LN_MCU
    {
        LN_MCU_UNKNOWN,
        LN_MCU_ARM_F3,
        LN_MCU_ARM_F4,
        LN_MCU_RISCV,
        LN_MCU_XTENSA,
    };

  public:
    static void identify();
    static LN_VENDOR vendor();
    static LN_MCU mcu();
    static const char *mcuAsString();

    static int flashSize();
    static int ramSize();

    static const char *idAsString();
    static int clockSpeed();
};
