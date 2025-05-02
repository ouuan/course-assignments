#include <arpa/inet.h>
#include <stdio.h>
#include <string.h>

#include "../include/forwarding_table.h"

#define NEXT_HOP_SIZE 32
#define STAGE_COUNT 8

extern const int NUMBER_OF_ENTRY_COUNT[17];
extern const int NUMBER_OF_CHILD_COUNT[STAGE_COUNT][17];

extern NextHopData nextHopList[NEXT_HOP_SIZE];
extern NextHopId entryPool[1 << 18];
extern BitmapNodeData nodePool[STAGE_COUNT][1 << 17];

int main()
{
    char dstStr[50], nextHopStr[50];
    struct in6_addr dst, nextHop;
    ForwardingTableEntry entry;
    entry.direct = 0;

    initForwardingTable();

    while (scanf("%s %hhu %s %hhu", dstStr, &entry.len, nextHopStr, &entry.interface) != EOF)
    {
        if (nextHopStr[0] == ':' && nextHopStr[1] == ':')
            entry.direct = 1;
        else
            entry.direct = 0;
        inet_pton(AF_INET6, dstStr, &dst);
        inet_pton(AF_INET6, nextHopStr, &nextHop);
        memcpy(&entry.dst, &dst, 16);
        memcpy(&entry.nextHop, &nextHop, 16);
        insertForwardingTableEntry(entry);
    }

    FILE *out = fopen("coe/next-hop.coe", "w");
    fprintf(out, "memory_initialization_radix=2;\nmemory_initialization_vector=\n");
    for (int i = 0; i < NEXT_HOP_SIZE; ++i)
    {
        fprintf(out, "%d", nextHopList[i].directAndInterface >> 2);
        fprintf(out, "%d", (nextHopList[i].directAndInterface >> 1) & 1);
        fprintf(out, "%d", nextHopList[i].directAndInterface & 1);
        for (int j = 15; j >= 0; --j)
            for (int k = 7; k >= 0; --k)
                fprintf(out, "%d", (nextHopList[i].nextHop.addr8[j] >> k) & 1);
        fprintf(out, i == NEXT_HOP_SIZE - 1 ? ";\n" : ",\n");
    }
    fclose(out);

    out = fopen("coe/entry.coe", "w");
    fprintf(out, "memory_initialization_radix=2;\nmemory_initialization_vector=\n");
    int size = 0;
    for (int i = 1; i <= 16; ++i)
        size += NUMBER_OF_ENTRY_COUNT[i] * i;
    for (int i = 0; i < size; ++i)
    {
        for (int j = 4; j >= 0; --j)
            fprintf(out, "%d", (entryPool[i] >> j) & 1);
        fprintf(out, i == size - 1 ? ";\n" : ",\n");
    }
    fclose(out);

    for (int i = 0; i < STAGE_COUNT; ++i)
    {
        char filename[30];
        sprintf(filename, "coe/node-%d.coe", i);
        FILE *out = fopen(filename, "w");
        fprintf(out, "memory_initialization_radix=2;\nmemory_initialization_vector=\n");
        int size = 0;
        for (int j = 1; j <= 16; ++j)
            size += NUMBER_OF_CHILD_COUNT[i][j] * j;
        for (int j = 0; j < size; ++j)
        {
            for (int k = 17; k >= 0; --k)
                fprintf(out, "%d", (nodePool[i][j].entryAddr >> k) & 1);
            for (int k = 17; k >= 0; --k)
                fprintf(out, "%d", (nodePool[i][j].ceAddr >> k) & 1);
            for (int k = 31; k >= 0; --k)
                fprintf(out, "%d", (nodePool[i][j].masks >> k) & 1);
            fprintf(out, j == size - 1 ? ";\n" : ",\n");
        }
        fclose(out);
    }
}
