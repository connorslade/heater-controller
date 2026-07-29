cargo objcopy --release -- -O binary target/firmware.bin
stm32flash -w target/firmware.bin -v -g 0x0 -i "-dtr&-rts,dtr,-dtr&rts:rts,,,,,,,,,," /dev/ttyUSB0
