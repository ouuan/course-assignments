#include <stdio.h>
#include <string.h>
#include <stdint.h>

int main() {

    FILE *out = fopen("coe/inst.coe", "w");
    fprintf(out, "memory_initialization_radix=16;\nmemory_initialization_vector=\n");
    uint8_t a0, a1, a2, a3;
    int cnt = 3072;
    int i = 1;
    while (scanf("%c%c%c%c", &a0, &a1, &a2, &a3) != EOF) {
        if (i > cnt) {
            printf("Error: inst out of range\n");
            fclose(out);
            return 0;
        }
        fprintf(out, "%02x%02x%02x%02x%c\n", a3, a2, a1, a0, ",;"[i==cnt]);
        i += 1;
    }
    printf("total inst: %d\n", i-1);
    for ( ; i <= cnt; ++i) {
      fprintf(out, "00000000%c\n", ",;"[i==cnt]);
    }
    fclose(out);
}