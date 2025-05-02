all: libhowvec.so libhotblocks.so libhotpages.so

lib%.so: %.c
	$(CC) $^ -O2 -shared -fpic -Wl,-soname,$@ -o $@ $(shell pkg-config glib-2.0 --cflags)

.PHONY: clean
clean:
	-rm *.so
