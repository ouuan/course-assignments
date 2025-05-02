#include <arpa/inet.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "../include/forwarding_table.h"

#define STAGE_COUNT 8
#define BRAM_UNIT (18 << 10)
#define ENTRY_SIZE 5
#define NODE_SIZE 68

extern int entryPoolCount[17];
extern int entryPoolPeek[17];
extern int nodePoolCount[STAGE_COUNT][17];
extern int nodePoolPeek[STAGE_COUNT][17];

int entryPoolSize[17];
int nodePoolSize[STAGE_COUNT][17];

ForwardingTableEntry entries[200000];

int main()
{
    char dstStr[50], nextHopStr[50];
    int len, interface;
    struct in6_addr dst, nextHop;

    int count = 0;
    int failed = 0;

    while (scanf("%s %d %s %d", dstStr, &len, nextHopStr, &interface) != EOF)
    {
        inet_pton(AF_INET6, dstStr, &dst);
        inet_pton(AF_INET6, nextHopStr, &nextHop);
        memcpy(&entries[count].dst, &dst, 16);
        memcpy(&entries[count].nextHop, &nextHop, 16);
        entries[count].len = len;
        entries[count].direct = 0;
        entries[count].interface = interface;
        ++count;
    }

    srand(time(0));

    const int N = 10000;

    for (int t = 0; t < N; ++t)
    {
        if (t % (N / 100) == 0)
            printf("progress: %d%%\n", t * 100 / N);
        initForwardingTable();
        for (int i = 0; i < count; ++i)
        {
            if (insertForwardingTableEntry(entries[i]) != 0)
            {
                failed = 1;
                printf("Failed at %d %d\n", t, i);
                break;
            }
        }
        if (failed)
            break;
        for (int i = 1; i < count; ++i)
        {
            const int p = rand() % (i + 1);
            ForwardingTableEntry tmp = entries[i];
            entries[i] = entries[p];
            entries[p] = tmp;
        }
    }

    int sum = 0;
    for (int i = 1; i < 17; ++i)
    {
        printf("%2d: %d / %d\n", i, entryPoolCount[i], entryPoolPeek[i]);
        entryPoolSize[i] = entryPoolPeek[i] * 21 / 20;
        sum += ENTRY_SIZE * entryPoolSize[i] * i;
    }
    printf("Entry half-BRAM count: %d\n", sum / BRAM_UNIT + 1);

    int total = 0;

    for (int s = 0; s < STAGE_COUNT; ++s)
    {
        int sum = 0;
        for (int i = 1; i < 17; ++i)
        {
            printf("%2d: %d / %d\n", i, nodePoolCount[s][i], nodePoolPeek[s][i]);
            nodePoolSize[s][i] = nodePoolPeek[s][i] * 21 / 20 + 64 / i;
            sum += NODE_SIZE * nodePoolSize[s][i] * i;
        }
        total += sum / BRAM_UNIT + 1;
        printf("%d\n", sum / BRAM_UNIT + 1);
    }
    printf("Node half-BRAM count: %d\n", total);

    printf("const int NUMBER_OF_ENTRY_COUNT[17] = {0, ");
    for (int i = 1; i < 17; ++i)
        printf("%d%s", entryPoolSize[i], i == 16 ? "};\n" : ", ");

    printf("const int NUMBER_OF_CHILD_COUNT[STAGE_COUNT][17] = {\n");
    for (int s = 0; s < STAGE_COUNT; ++s)
    {
        printf("    {0, ");
        for (int i = 1; i < 17; ++i)
            printf("%d%s", nodePoolSize[s][i], i == 16 ? "},\n" : ", ");
    }
    printf("};");

    return failed;
}
