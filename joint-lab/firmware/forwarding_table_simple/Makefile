HEADERS=$(wildcard ../include/*.h)

.PHONY: all
all: forwarding_table forwarding_table_simple makedata

forwarding_table: main.c forwarding_table.o common.o
	$(CC) -g -D_STDINT_H_ $^ -o $@

forwarding_table_simple: main.c forwarding_table_simple.o common.o
	$(CC) -g -D_STDINT_H_ $^ -o $@

makedata: makedata.o
	$(CC) -g  $^ -o $@

forwarding_table_simple.o: forwarding_table_simple.c $(HEADERS)
	$(CC) -g -I ../include -c $< -o $@

makedata.o: makedata.c $(HEADERS)
	$(CC) -g -c $< -o $@

%.o: ../%.c $(HEADERS)
	$(CC) -g -I ../include -c $< -o $@

.PHONY: clean
clean:
	-rm *.o forwarding_table forwarding_table_simple
