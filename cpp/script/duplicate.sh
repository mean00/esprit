for i in basicDac blink  everything exti i2c_scanner lcd mcp23017 pwm rotaryEncoder signalGenerator ssd1306 st7735 systemTime ws2812b;
do
cp adc/mcuSelect.cmake $i/mcuSelect.cmake
cp adc/platformConfig.cmake $i/
done
