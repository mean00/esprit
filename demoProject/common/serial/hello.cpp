#include "esprit.h"
#include "lnSerial.h"

#define LED LN_SYSTEM_LED

#ifdef USE_RP2040
#define UART_INSTANCE 1
#else
#define UART_INSTANCE 2
#endif

/**
 */
void setup()
{
    lnPinMode(LED, lnOUTPUT);
#ifdef USE_RP2040
    lnPinMode(GPIO17, lnUART);
    lnPinMode(GPIO16, lnUART);
    lnPinMode(GPIO20, lnUART);
    lnPinMode(GPIO21, lnUART);
#endif
}
int rx = 0, tx = 0;
#define RX_SIZE 128
#define RX_TASK_STACK_BYTES 800
// Stack of rxTask: the caller owns it, see lnFreeRTOS_pp.h
static StackType_t _rxTaskStack[LN_TASK_STACK_WORDS(RX_TASK_STACK_BYTES)];
class rxTask : public lnTask
{
  public:
    rxTask(lnSerialRxTx *serial) : lnTask("RX", 3, _rxTaskStack, RX_TASK_STACK_BYTES)
    {
        _ser = serial;
    }
    static void cb(void *cookie, lnSerialCore::Event ev)
    {
        rxTask *t = (rxTask *)cookie;
        t->eventHandler(ev);
    }
    void eventHandler(lnSerialCore::Event ev)
    {
        _evGroup->setEvents(1);
    }
    void run()
    {
        _evGroup = new lnFastEventGroup;
        _evGroup->takeOwnership();
        _ser->setCallback(cb, this);
        _ser->transmit(4, (const uint8_t *)"\nGo\n");
        _ser->enableRx(true);
        uint8_t c[256];
        while (1)
        {
            _evGroup->waitEvents(1);
            uint8_t *ptr = NULL;
            while (1)
            {
                int nb = _ser->getReadPointer(&ptr);
                if (!nb)
                    break;
                xAssert(nb <= RX_SIZE);
                //_ser->transmit(2,(uint8_t *)">>");
                memcpy(c, ptr, nb);
                c[nb] = 0;
                // Logger("%s",c);
                _ser->transmit(nb, (uint8_t *)ptr);
                //_ser->transmit(2,(uint8_t *)">>");
                _ser->consume(nb);
            }
        }
    }
    lnSerialRxTx *_ser;
    lnFastEventGroup *_evGroup;
};

const char *block = ";End of Gcode\n"
                    ";SETTING_3 {global_quality: [general]\\nversion = 4\\nname = KLIPPER_0.3\\nde\n"
                    ";SETTING_3 finition = anycubic_i3_mega\\n\\n[metadata]\\ntype = quality_changes\n"
                    ";SETTING_3 quality_type = draft\\nsetting_version = 22\\n\\n[values]\\nacceler\n"
                    ";SETTING_3 ation_enabled = False\\nadhesion_type = skirt\\nadhesion_z_offset = -\n"
                    ";SETTING_3 0.05\\njerk_enabled = False\\nmaterial_bed_temperature = 55\\nsupport\n"
                    ";SETTING_3 _enable = False, extruder_quality: [[general]\\nversion = 4\n"
                    ";SETTING_3 \\nname = KLIPPER_0.3\\ndefinition = anycubic_i3_mega\\n\\n[metadata]\n"
                    ";SETTING_3 \\ntype = quality_changes\\nquality_type = draft\\nintent_category = \n"
                    ";SETTING_3 default\\nposition = 0\\nsetting_version = 22\\n\\n[values]\\nacceler\n"
                    ";SETTING_3 ation_layer_0 = 500\\ncool_fan_speed = 60\\ncool_fan_speed_0 = 10\\nc\n"
                    ";SETTING_3 ool_fan_speed_max = 60\\ncool_min_layer_time_fan_speed_max = 10.1\\ni\n"
                    ";SETTING_3 nfill_pattern = lines\\ninfill_sparse_density = 12\\ninitial_layer_li\n"
                    ";SETTING_3 ne_width_factor = 120\\nmaterial_final_print_temperature = 205.0\\nma\n"
                    ";SETTING_3 terial_flow_layer_0 = 105\\nmaterial_initial_print_temperature = 205.\n"
                    ";SETTING_3 0\\nmaterial_print_temperature = 205.0\\nmaterial_print_temperature_l\n"
                    ";SETTING_3 ayer_0 = 205\\nretraction_amount = 5.5\\nretraction_speed = 50\\nskin\n"
                    ";SETTING_3 _overlap = 10\\nspeed_print = 45\\nspeed_wall_0 = 30\\nspeed_wall_x =\n"
                    ";SETTING_3  50\\ntop_bottom_pattern_0 = zigzag\\n\\n";

void txOnly()
{
    lnSerialTxOnly *serial = createLnSerialTxOnly(UART_INSTANCE, true, false);
    serial->init();
    serial->setSpeed(115200);
    const char *hi = "here we go ....\n";
    serial->transmit(strlen(hi), (const uint8_t *)hi);
    int size = strlen(block);

    while (1)
    {
        serial->transmit(size, (const uint8_t *)block);
        // Logger(block);
    }
}
void loop()
{
    Logger("Starting lnUart Test\n");

    // txOnly();
    lnSerialRxTx *serial = createLnSerialRxTx(UART_INSTANCE, RX_SIZE, true);
    serial->init();
    serial->setSpeed(115200);
    rxTask receiveTask(serial);
    receiveTask.start();
    int count = 0;
    int round = 0;

    while (1)
    {
        lnDelayMs(200);
        lnDigitalToggle(LED);
        Logger("*\n");
    }
}
// EOF
