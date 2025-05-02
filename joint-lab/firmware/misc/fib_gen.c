#include <arpa/inet.h>
#include <stdio.h>
#include <stdlib.h>

int main()
{
    char dstStr[50], nextHopStr[50];
    uint8_t len, interface;
    struct in6_addr dst, nextHop;

    FILE *out = fopen("../fib.c", "w");

    fprintf(out, "#include \"fib.h\"\n");
    fprintf(out, "const FibEntry __attribute__((section(\".fib\"))) fib[200000] = {\n");

    while (scanf("%s %hhu %s %hhu", dstStr, &len, nextHopStr, &interface) != EOF)
    {
        fprintf(out, "{{.addr32={");
        inet_pton(AF_INET6, dstStr, &dst);
        for (int i = 0; i < 4; ++i)
            fprintf(out, "0x%x,", dst.s6_addr32[i]);
        fprintf(out, "}},%hhu,%hhu,%hhu},\n", len, interface, nextHopStr[0] == ':');
    }

    fprintf(out, "};");

    fclose(out);

    return 0;
}
