HEADERS=$(wildcard ../include/*.h)

all: size_test coe_gen fib_gen

size_test: size_test.c forwarding_table.o common.o
	$(CC) -O2 -D_STDINT_H_ $^ -o $@

coe_gen: coe_gen.c forwarding_table.o common.o
	$(CC) -O2 -D_STDINT_H_ $^ -o $@

%.o: ../%.c $(HEADERS)
	$(CC) -O2 -I ../include -D SIZE_TEST -c $< -o $@

.PHONY: clean
clean:
	-rm *.o size_test coe_gen
