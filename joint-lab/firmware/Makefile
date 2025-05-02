TRIPLE = riscv64-unknown-elf-
GCC = $(TRIPLE)gcc
OBJCOPY = $(TRIPLE)objcopy
OBJDUMP = $(TRIPLE)objdump
QEMU = qemu-system-riscv32

DEFINES = -DPRINTF_DISABLE_SUPPORT_FLOAT -DPRINTF_DISABLE_SUPPORT_EXPONENTIAL \
          -DPRINTF_DISABLE_SUPPORT_LONG_LONG
CFLAGS = -fno-builtin -nostdlib -nostdinc -static -Wl,--gc-sections,--print-gc-sections -Og -g -Wall \
         -Iinclude $(DEFINES) -march=rv32i -mabi=ilp32 -DRV32 -flto
LDFLAGS = -z separate-code

-include Makefile.config

override EN_UART16550 ?= n
ifeq ($(EN_UART16550), y)
	CFLAGS += -DENABLE_UART16550
endif

HEADERS=$(wildcard include/*.h)
SOURCES=$(wildcard *.c *.S)
OBJECTS=$(patsubst %.c,%.o,$(wildcard *.c)) $(patsubst %.S,%.o,$(wildcard *.S))
TARGET=kernel.elf

.PHONY: all
all:
	mkdir -p bin
	for i in single intra0 intra1 intra2 inter0 inter1 inter2; do \
		make $$i ; \
	done

.PHONY: single
single: CONFIG_FLAGS=-D BASE_MAC5=0x20 -D DIRECT2=0x497 -D BASE_DIRECT3=0 -D RIPNG_UPDATE_TIME=30000000 -D RIPNG_UPDATE_PER_TIMEOUT=6
single: clean prod
	mv kernel.bin bin/single.bin

.PHONY: intra0
intra0: CONFIG_FLAGS=-D BASE_MAC5=0x20 -D BASE_LL7=0x1020 -D DIRECT2=0x497 -D BASE_DIRECT3=0 -D RIPNG_UPDATE_TIME=3000000 -D RIPNG_UPDATE_PER_TIMEOUT=10
intra0: clean prod
	mv kernel.bin bin/intra0.bin

.PHONY: intra1
intra1: CONFIG_FLAGS=-D BASE_MAC5=0x24 -D BASE_LL7=0x1024 -D DIRECT2=0x497 -D BASE_DIRECT3=0x100 -D RIPNG_UPDATE_TIME=3000000 -D RIPNG_UPDATE_PER_TIMEOUT=10
intra1: clean prod
	mv kernel.bin bin/intra1.bin

.PHONY: intra2
intra2: CONFIG_FLAGS=-D BASE_MAC5=0x28 -D BASE_LL7=0x1028 -D DIRECT2=0x497 -D BASE_DIRECT3=0x200 -D RIPNG_UPDATE_TIME=3000000 -D RIPNG_UPDATE_PER_TIMEOUT=10
intra2: clean prod
	mv kernel.bin bin/intra2.bin

.PHONY: inter0
inter0: CONFIG_FLAGS=-D BASE_MAC5=0x20 -D BASE_LL7=0x1020 -D DIRECT2=0x490 -D BASE_DIRECT3=0 -D RIPNG_UPDATE_TIME=3000000 -D RIPNG_UPDATE_PER_TIMEOUT=10
inter0: clean prod
	mv kernel.bin bin/inter0.bin

.PHONY: inter1
inter1: CONFIG_FLAGS=-D BASE_MAC5=0x24 -D BASE_LL7=0x1024 -D DIRECT2=0x491 -D BASE_DIRECT3=0 -D RIPNG_UPDATE_TIME=3000000 -D RIPNG_UPDATE_PER_TIMEOUT=10
inter1: clean prod
	mv kernel.bin bin/inter1.bin

.PHONY: inter2
inter2: CONFIG_FLAGS=-D BASE_MAC5=0x28 -D BASE_LL7=0x1028 -D DIRECT2=0x492 -D BASE_DIRECT3=0 -D RIPNG_UPDATE_TIME=3000000 -D RIPNG_UPDATE_PER_TIMEOUT=10
inter2: clean prod
	mv kernel.bin bin/inter2.bin

.PHONY: prod
prod: $(OBJECTS) $(TARGET) kernel.bin

.PHONY: viasm
viasm: $(TARGET)
	$(OBJDUMP) -xd $< | vi -

.PHONY: inst
inst: $(TARGET)
	$(OBJDUMP) -Mno-aliases -d $< | ./inst.py

%.asm: %.elf
	$(OBJDUMP) -xd $< > $@

%.bin: %.elf
	$(OBJCOPY) -S -j .text -j .*data -O binary $< $@
	dd if=/dev/null of=$@ bs=1 count=1 seek=8M

%.o: %.c $(HEADERS)
	$(GCC) $(CFLAGS) $(CONFIG_FLAGS) -c $< -o $@

%.o: %.S $(HEADERS)
	$(GCC) $(CFLAGS) -c $< -o $@

$(TARGET): $(OBJECTS) linker.ld
	$(GCC) -Tlinker.ld $(CFLAGS) $(LDFLAGS) $(OBJECTS) -lgcc -o $@

.PHONY: sim
sim: $(TARGET)
	$(QEMU) -machine virt -nographic -m 64M -kernel $(TARGET) -s -bios none

.PHONY: debug
debug: $(TARGET)
	$(QEMU) -machine virt -nographic -m 64M -kernel $(TARGET) -S -s -bios none

profile: $(TARGET)
	$(QEMU) -machine virt -nographic -m 64M -kernel $(TARGET) -s -bios none \
		-plugin qemu-plugins/libhowvec.so,inline=on \
		-plugin qemu-plugins/libhotblocks.so \
		-plugin qemu-plugins/libhotpages.so,pagesize=1024 \
		-d plugin

.PHONY: clean
clean:
	-rm *.o *.elf *.bin *.asm *.so $(TARGET)
