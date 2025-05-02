#include "forwarding_table.h"

#include <printf.h>

#define DEPTH_PER_STAGE 4
#define STAGE_COUNT (32 / DEPTH_PER_STAGE)
#define NEXT_HOP_SIZE 32

#define RIPNG_GC_SIZE 100000

const int NUMBER_OF_ENTRY_COUNT[17] = {0,   51396, 10170, 5235, 4028, 1718, 1450, 1273, 1264,
                                       921, 910,   891,   895,  941,  1004, 1157, 2288};

const int NUMBER_OF_CHILD_COUNT[STAGE_COUNT][17] = {
    {0, 73, 35, 23, 17, 13, 11, 9, 8, 7, 6, 5, 5, 4, 4, 4, 5},
    {0, 3595, 2603, 1477, 531, 209, 176, 162, 163, 160, 156, 158, 160, 171, 177, 198, 357},
    {0, 33620, 6017, 3008, 2035, 962, 746, 596, 570, 345, 270, 205, 205, 188, 170, 180, 412},
    {0, 74, 35, 23, 17, 13, 11, 10, 9, 7, 6, 5, 5, 4, 4, 4, 4},
    {0, 74, 35, 23, 17, 13, 11, 10, 9, 7, 6, 5, 5, 4, 4, 4, 4},
    {0, 74, 35, 23, 17, 13, 11, 10, 9, 7, 6, 5, 5, 4, 4, 4, 4},
    {0, 74, 35, 23, 17, 13, 11, 10, 9, 7, 6, 5, 5, 4, 4, 4, 4},
    {0, 74, 35, 23, 17, 13, 11, 10, 9, 7, 6, 5, 5, 4, 4, 4, 4},
};

#ifdef RV32
NextHopData __attribute__((section(".nextHopList"))) nextHopList[NEXT_HOP_SIZE];
NextHopId __attribute__((section(".entryPool"))) entryPool[1 << 18];
BitmapNodeData __attribute__((section(".nodePool"))) nodePool[STAGE_COUNT][1 << 17];
#else
NextHopData nextHopList[NEXT_HOP_SIZE];
NextHopId entryPool[1 << 18];
BitmapNodeData nodePool[STAGE_COUNT][1 << 17];
#endif

int nextHopCount;

NextHopId *entryPoolStack[90000];
NextHopId *entryPoolStart[17];
int entryPoolTop[17];

RIPngData ripngData[1 << 18];
Ipv6Prefix ripngGcList[RIPNG_GC_SIZE];
int ripngGcCount;

BitmapNodeData *nodePoolStack[STAGE_COUNT][1 << 16];
BitmapNodeData *nodePoolStart[STAGE_COUNT][17];
int nodePoolTop[STAGE_COUNT][17];
BitmapNodeData *root;

int
#ifdef RV32
    __attribute__((section(".dpy")))
#endif
    entryCount;

#ifdef SIZE_TEST
int entryPoolCount[17];
int entryPoolPeek[17];
int nodePoolCount[STAGE_COUNT][17];
int nodePoolPeek[STAGE_COUNT][17];

void addCount(int *count, int *peek)
{
    *count += 1;
    if (*count > *peek)
        *peek = *count;
}
#endif

static inline int popcount(int x)
{
#ifdef RV32
    int result;
    asm(".insn i 0b0010011, 0b001, %0, %1, 0b011000000010" : "=r"(result) : "r"(x));
    return result;
#else
    return __builtin_popcount(x);
#endif
}

static inline int bext(int x, int p)
{
#ifdef RV32
    int result;
    asm(".insn r 0b0110011, 0b101, 0b0100100, %0, %1, %2" : "=r"(result) : "r"(x), "r"(p));
    return result;
#else
    return (x >> p) & 1;
#endif
}

static inline int bset(int x, int p)
{
#ifdef RV32
    int result;
    asm(".insn r 0b0110011, 0b001, 0b0010100, %0, %1, %2" : "=r"(result) : "r"(x), "r"(p));
    return result;
#else
    return x | (1 << p);
#endif
}

static inline int ctz(int x)
{
#ifdef RV32
    int result;
    asm(".insn i 0b0010011, 0b001, %0, %1, 0b011000000001" : "=r"(result) : "r"(x));
    return result;
#else
    return __builtin_ctz(x);
#endif
}

// @returns the nextHopId corresponding to *nextHop*, or -1 if the list is full
static int getNextHopId(RoutingTableEntry *entry)
{
    const int directAndInterface = (entry->direct << 2) | entry->interface;
    for (int i = nextHopCount - 1; i >= 0; --i)  // entries at the beginning are direct routes
    {
        if (nextHopList[i].directAndInterface == directAndInterface &&
            (entry->direct || addrCmp(&nextHopList[i].nextHop, entry->nextHop) == 0))
            return i;
    }
    if (nextHopCount == NEXT_HOP_SIZE)
        return -1;
    const int id = nextHopCount;
    nextHopCount++;
    volatile NextHopData *d = nextHopList + id;
    d->nextHop = *entry->nextHop;
    d->directAndInterface = (entry->direct << 2) | entry->interface;
    return id;
}

static RIPngInfo *getRipngInfo(NextHopId *entryAddr)
{
    return (RIPngInfo *)ripngData + (entryAddr - entryPool);
}

static int sizeOfEntryAddr(NextHopId *addr, int minSize)
{
    int size;
    for (size = minSize; size < 16; ++size)
        if (addr < entryPoolStart[size + 1])
            break;
    return size;
}

// @returns new base address, or NULL if fail
static NextHopId *addEntry(NextHopId *oldAddr, int minNewBlockSize, NextHopId newData,
                           RIPngInfo newRipng, int newIndex)
{
    int newBlockSize;
    NextHopId *newAddr;

    for (newBlockSize = minNewBlockSize; newBlockSize <= 16; ++newBlockSize)
    {
        newAddr = entryPoolStack[entryPoolTop[newBlockSize]];
        if (newAddr >= entryPoolStart[newBlockSize])
            break;
    }
    if (newBlockSize > 16)
    {
        printf("ERROR: no space for entry with size %d\r\n", minNewBlockSize);
        return NULL;
    }

    --entryPoolTop[newBlockSize];
    newAddr[newIndex] = newData;
    RIPngInfo *newRipngAddr = getRipngInfo(newAddr);
    newRipngAddr[newIndex] = newRipng;

#ifdef SIZE_TEST
    addCount(&entryPoolCount[newBlockSize], &entryPoolPeek[newBlockSize]);
#endif

    if (oldAddr)
    {
        const int oldBlockSize = sizeOfEntryAddr(oldAddr, minNewBlockSize - 1);
#ifdef SIZE_TEST
        --entryPoolCount[oldBlockSize];
#endif
        entryPoolStack[++entryPoolTop[oldBlockSize]] = oldAddr;
        RIPngInfo *oldRipngAddr = getRipngInfo(oldAddr);
        for (int i = 0; i < newIndex; ++i)
        {
            newAddr[i] = oldAddr[i];
            newRipngAddr[i] = oldRipngAddr[i];
        }
        for (int i = newIndex + 1; i < minNewBlockSize; ++i)
        {
            newAddr[i] = oldAddr[i - 1];
            newRipngAddr[i] = oldRipngAddr[i - 1];
        }
    }

    return newAddr;
}

static void deleteEntry(NextHopId *addr, int usedSize, int deletedIndex)
{
    if (usedSize == 1)
    {
        // free block
        const int blockSize = sizeOfEntryAddr(addr, 1);
        entryPoolStack[++entryPoolTop[blockSize]] = addr;
#ifdef SIZE_TEST
        --entryPoolCount[blockSize];
#endif
    }
    else
    {
        // only move entries, no switching memory pool
        RIPngInfo *ripngAddr = getRipngInfo(addr);
        for (int i = deletedIndex; i < usedSize - 1; ++i)
        {
            addr[i] = addr[i + 1];
            ripngAddr[i] = ripngAddr[i + 1];
        }
    }
}

static int sizeOfNodeAddr(BitmapNodeData *addr, int stage, int minSize)
{
    int size;
    for (size = minSize; size < 16; ++size)
        if (addr < nodePoolStart[stage][size + 1])
            break;
    return size;
}

static BitmapNode readNode(BitmapNodeData *data, int depth)
{
    BitmapNode ret;

    ret.isLeaf = data->masks >> 31;
    ret.entryMask = (data->masks >> 16) & 0x7fff;
    ret.ceMask = data->masks & 0xffff;

    ret.entryAddr = entryPool + data->entryAddr;

    if (ret.isLeaf)
        ret.ceAddr.extraEntryAddr = entryPool + data->ceAddr;
    else
        ret.ceAddr.childAddr = nodePool[(depth + 1) / DEPTH_PER_STAGE] + data->ceAddr;

    return ret;
}

static void writeNode(volatile BitmapNodeData *addr, BitmapNode *node, int depth)
{
    addr->masks = (node->isLeaf << 31) | (node->entryMask << 16) | node->ceMask;
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmaybe-uninitialized"
    addr->ceAddr = node->isLeaf ? node->ceAddr.extraEntryAddr - entryPool
                                : node->ceAddr.childAddr - nodePool[(depth + 1) / DEPTH_PER_STAGE];
    addr->entryAddr = node->entryAddr - entryPool;
#pragma GCC diagnostic pop
}

static int allocateNode(int minSize, int stage, BitmapNodeData **allocatedAddr)
{
    BitmapNodeData *addr;
    int size;
    for (size = minSize; size <= 16; size += 1)
    {
        addr = nodePoolStack[stage][nodePoolTop[stage][size]];
        if (addr >= nodePoolStart[stage][size])
            break;
    }
    if (size > 16)
    {
        printf("ERROR: no space for node with size %d at stage %d\r\n", minSize, stage);
        *allocatedAddr = NULL;
        return 0;
    }
    --nodePoolTop[stage][size];
#ifdef SIZE_TEST
    addCount(&nodePoolCount[stage][size], &nodePoolPeek[stage][size]);
#endif
    *allocatedAddr = addr;
    return size;
}

static BitmapNodeData *addNode(BitmapNodeData *oldAddr, BitmapNode *newNode, int newIndex,
                               int minNewBlockSize, int depth)
{
    const int stage = (depth + 1) / DEPTH_PER_STAGE;

    BitmapNodeData *newAddr;
    allocateNode(minNewBlockSize, stage, &newAddr);
    if (newAddr == NULL)
        return NULL;
    writeNode(newAddr + newIndex, newNode, depth + 1);

    if (oldAddr)
    {
        const int oldBlockSize = sizeOfNodeAddr(oldAddr, stage, minNewBlockSize - 1);
#ifdef SIZE_TEST
        --nodePoolCount[stage][oldBlockSize];
#endif
        nodePoolStack[stage][++nodePoolTop[stage][oldBlockSize]] = oldAddr;
        for (int i = 0; i < newIndex; ++i)
            newAddr[i] = oldAddr[i];
        for (int i = newIndex + 1; i < minNewBlockSize; ++i)
            newAddr[i] = oldAddr[i - 1];
    }

    return newAddr;
}

static int getValIndexFromMask(uint32_t mask, uint8_t bitIndex)
{
    return popcount(mask & ((1 << bitIndex) - 1));
}

// Pushdown the extra entries of a node with isLeaf == 1 to its children
// @returns 0 if success, 1 if fail
static int pushdownEntries(BitmapNode *node, int depth)
{
    const int usedSize = popcount(node->ceMask);
    if (usedSize == 0)
    {
        node->isLeaf = 0;
        return 0;
    }

    const int childDepth = depth + 1;
    const int childStage = childDepth / DEPTH_PER_STAGE;

    BitmapNodeData *children;
    const int allocatedSize = allocateNode(usedSize, childStage, &children);
    if (children == NULL)
        return 1;

    for (int i = 0; i < usedSize; ++i)
    {
        BitmapNode child;
        child.entryAddr = addEntry(NULL, 1, (node->ceAddr.extraEntryAddr)[i],
                                   *getRipngInfo(node->ceAddr.extraEntryAddr + i), 0);
        if (child.entryAddr == NULL)
        {
            // rollback finished actions
            for (int j = 0; j < i; ++j)
            {
                const BitmapNode child = readNode(children + j, childDepth);
                entryPoolStack[++entryPoolTop[1]] = child.entryAddr;
#ifdef SIZE_TEST
                --entryPoolCount[1];
#endif
            }
#ifdef SIZE_TEST
            --nodePoolCount[childStage][allocatedSize];
#endif
            nodePoolStack[childStage][++nodePoolTop[childStage][allocatedSize]] = children;
            return 1;
        }
        child.isLeaf = 1;
        child.ceMask = 0;
        child.entryMask = 0x4000;
        writeNode(children + i, &child, childDepth);
    }

    const int oldEntryBlockSize = sizeOfEntryAddr(node->ceAddr.extraEntryAddr, usedSize);
    entryPoolStack[++entryPoolTop[oldEntryBlockSize]] = node->ceAddr.extraEntryAddr;
#ifdef SIZE_TEST
    --entryPoolCount[oldEntryBlockSize];
#endif

    node->isLeaf = 0;
    node->ceAddr.childAddr = children;

    return 0;
}

void initForwardingTable()
{
    entryCount = 0;
    nextHopCount = 0;

    entryPoolTop[0] = 0;
    entryPoolStart[0] = entryPool;

    for (int i = 0; i < 16; ++i)
    {
#ifdef SIZE_TEST
        entryPoolCount[i + 1] = 0;
#endif
        NextHopId *currentEntry = entryPoolStart[i] + i * NUMBER_OF_ENTRY_COUNT[i];
        entryPoolStart[i + 1] = currentEntry;
        NextHopId **currentEntryStack = &entryPoolStack[entryPoolTop[i] + 1];
        for (int j = 0; j < NUMBER_OF_ENTRY_COUNT[i + 1]; ++j)
        {
            *currentEntryStack = currentEntry;
            currentEntryStack += 1;
            currentEntry += i + 1;
        }
        entryPoolTop[i + 1] = entryPoolTop[i] + NUMBER_OF_ENTRY_COUNT[i + 1];
    }

    for (int s = 0; s < STAGE_COUNT; ++s)
    {
        nodePoolTop[s][0] = 0;
        nodePoolStart[s][0] = nodePool[s];

        for (int i = 0; i < 16; ++i)
        {
#ifdef SIZE_TEST
            nodePoolCount[s][i + 1] = 0;
#endif
            BitmapNodeData *currentNode = nodePoolStart[s][i] + i * NUMBER_OF_CHILD_COUNT[s][i];
            nodePoolStart[s][i + 1] = currentNode;
            BitmapNodeData **currentNodeStack = &nodePoolStack[s][nodePoolTop[s][i] + 1];
            for (int j = 0; j < NUMBER_OF_CHILD_COUNT[s][i + 1]; ++j)
            {
                *currentNodeStack = currentNode;
                currentNodeStack += 1;
                currentNode += i + 1;
            }
            nodePoolTop[s][i + 1] = nodePoolTop[s][i] + NUMBER_OF_CHILD_COUNT[s][i + 1];
        }
    }

    // allocate root at address 0
    nodePoolStack[0][1] = nodePoolStack[0][nodePoolTop[0][1]];
    nodePoolStack[0][nodePoolTop[0][1]] = nodePool[0];

    BitmapNode u;
    u.isLeaf = 1;
    u.entryMask = 0;
    u.ceMask = 0;
    allocateNode(1, 0, &root);
    writeNode(root, &u, 0);
}

static uint8_t get4bitAddrPart(const Ipv6Address *addr, int partIndex)
{
    const uint8_t octet = addr->addr8[partIndex >> 1];
    return partIndex & 1 ? octet & 0xf : octet >> 4;
}

static void updateEntry(BitmapNodeData *uAddr, BitmapNode *u, int depth, RoutingTableEntry *rte,
                        uint8_t maskIndex, NextHopId **entryBase, int bitmapIndex,
                        uint32_t *entryMask, int nextHopId)
{
    if (bext(*entryMask, maskIndex))
    {
        // found existing entry

        RIPngInfo *ripngInfo = getRipngInfo(*entryBase + bitmapIndex);

        if (nextHopId == (*entryBase)[bitmapIndex])
        {
            // same next hop
            if (rte->metricWithCost >= RIPNG_METRIC_INF)
            {
                deleteEntry(*entryBase, popcount(*entryMask), bitmapIndex);
                *entryMask &= ~(1 << maskIndex);
                writeNode(uAddr, u, depth);
                if (ripngGcCount < RIPNG_GC_SIZE)
                {
                    ripngGcList[ripngGcCount].addr = *rte->prefix;
                    ripngGcList[ripngGcCount].len = rte->len;
                    ripngGcCount++;
                }
                --entryCount;
            }
            else
            {
                ripngInfo->metric = rte->metricWithCost;
                ripngInfo->timeout = RIPNG_UPDATE_PER_TIMEOUT;
            }
        }
        else if (rte->metricWithCost < ripngInfo->metric ||
                 (rte->metricWithCost == ripngInfo->metric &&
                  ripngInfo->timeout < RIPNG_UPDATE_PER_TIMEOUT / 2))
        {
            (*entryBase)[bitmapIndex] = nextHopId;
            ripngInfo->metric = rte->metricWithCost;
            ripngInfo->timeout = RIPNG_UPDATE_PER_TIMEOUT;
        }
    }
    else
    {
        // entry not found

        if (rte->metricWithCost >= RIPNG_METRIC_INF)
            return;

        RIPngInfo ripngInfo;
        ripngInfo.metric = rte->metricWithCost;
        ripngInfo.timeout = RIPNG_UPDATE_PER_TIMEOUT;

        NextHopId *newAddr = addEntry(*entryMask ? *entryBase : NULL, popcount(*entryMask) + 1,
                                      nextHopId, ripngInfo, bitmapIndex);
        if (newAddr == NULL)
            return;
        *entryBase = newAddr;
        *entryMask = bset(*entryMask, maskIndex);
        writeNode(uAddr, u, depth);
        ++entryCount;
    }
}

void updateForwardingTable(RoutingTableEntry rte)
{
    const int nextHopId = getNextHopId(&rte);
    if (nextHopId == -1)
        return;

    BitmapNodeData *uAddr = root;
    BitmapNode u = readNode(uAddr, 0);

    // find the node
    for (int i = 0; (i + 1) * 4 <= rte.len; ++i)
    {
        const uint8_t addrPart = get4bitAddrPart(rte.prefix, i);
        const int index = getValIndexFromMask(u.ceMask, addrPart);
        if (!u.isLeaf && bext(u.ceMask, addrPart))
        {
            // target is in the subtree
            uAddr = u.ceAddr.childAddr + index;
            u = readNode(uAddr, i + 1);
        }
        else if (u.isLeaf && (i + 1) * 4 == rte.len)
        {
            // target is in the extra entries
            updateEntry(uAddr, &u, i, &rte, addrPart, &u.ceAddr.extraEntryAddr, index, &u.ceMask,
                        nextHopId);
            return;
        }
        else  // create a new child for the target
        {
            if (rte.metricWithCost >= RIPNG_METRIC_INF)
                return;
            if (u.isLeaf)
            {
                if (pushdownEntries(&u, i) == 1)
                    return;
                if (bext(u.ceMask, addrPart))  // subtree created during pushdown
                {
                    writeNode(uAddr, &u, i);
                    uAddr = u.ceAddr.childAddr + index;
                    u = readNode(uAddr, i + 1);
                    continue;
                }
            }
            BitmapNode v;
            v.isLeaf = 1;
            v.entryMask = 0;
            v.ceMask = 0;
            BitmapNodeData *newAddr =
                addNode(u.ceMask ? u.ceAddr.childAddr : NULL, &v, index, popcount(u.ceMask) + 1, i);
            if (newAddr == NULL)
                return;
            u.ceAddr.childAddr = newAddr;
            u.ceMask = bset(u.ceMask, addrPart);
            writeNode(uAddr, &u, i);
            uAddr = newAddr + index;
            u = v;
        }
    }

    // calculate entry position
    const uint8_t addrPart = get4bitAddrPart(rte.prefix, rte.len / 4);
    uint8_t entryIndex = addrPart >> 1;
    for (int i = rte.len & 3; i < 3; ++i)
        entryIndex = (entryIndex >> 1) | 8;
    const int index = getValIndexFromMask(u.entryMask, entryIndex);
    const int depth = rte.len / 4;

    updateEntry(uAddr, &u, depth, &rte, entryIndex, &u.entryAddr, index, &u.entryMask, nextHopId);
}

// @returns 1 if exists, 0 if not
static int queryExistence(Ipv6Prefix *prefix)
{
    BitmapNodeData *uAddr = root;
    BitmapNode u = readNode(uAddr, 0);

    for (int i = 0; (i + 1) * 4 <= prefix->len; ++i)
    {
        const uint8_t addrPart = get4bitAddrPart(&prefix->addr, i);
        const int index = getValIndexFromMask(u.ceMask, addrPart);
        if (bext(u.ceMask, addrPart))
        {
            if (u.isLeaf)
                return (i + 1) * 4 == prefix->len;
            else
            {
                uAddr = u.ceAddr.childAddr + index;
                u = readNode(uAddr, i + 1);
            }
        }
        else
            return 0;
    }

    const uint8_t addrPart = get4bitAddrPart(&prefix->addr, prefix->len / 4);
    uint8_t entryIndex = addrPart >> 1;
    for (int i = prefix->len & 3; i < 3; ++i)
        entryIndex = (entryIndex >> 1) | 8;
    return bext(u.entryMask, entryIndex);
}

static volatile RIPngRTE *generateRTE(NextHopId *entryAddr, uint32_t *entryMask, int maskIndex,
                                      int bitmapIndex)
{
    RIPngInfo *info = getRipngInfo(entryAddr + bitmapIndex);

    const uint32_t directAndInterface = nextHopList[entryAddr[bitmapIndex]].directAndInterface;

    uint8_t metric;

    if (directAndInterface & 4)  // direct routes do not timeout
        metric = info->metric;
    else
    {
        if (info->timeout == 0)
        {
            metric = RIPNG_METRIC_INF;
            deleteEntry(entryAddr, popcount(*entryMask), bitmapIndex);
            *entryMask &= ~(1 << maskIndex);
            --entryCount;
        }
        else
        {
            metric = info->metric;
            info->timeout -= 1;
        }
    }

    return nextRTE(directAndInterface, metric);
}

static void generateRipngResponseDfs(BitmapNodeData *uAddr, int depth, Ipv6Address *prefix)
{
    static const uint8_t ADDR_PART[15] = {0, 2, 4, 6, 8, 10, 12, 14, 0, 4, 8, 12, 0, 8, 0};
    static const uint8_t LEN[15] = {3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 1, 1, 0};

    BitmapNode u = readNode(uAddr, depth);

    for (int p = u.entryMask; p; p &= p - 1)
    {
        const int i = ctz(p);
        volatile RIPngRTE *rte =
            generateRTE(u.entryAddr, &u.entryMask, i, getValIndexFromMask(u.entryMask, i));
        rte->prefix = *prefix;
        rte->prefix.addr8[depth / 2] =  // cannot use |= because rte is write-only
            prefix->addr8[depth / 2] | (depth & 1 ? ADDR_PART[i] : ADDR_PART[i] << 4);
        rte->len = depth * 4 + LEN[i];
    }

    for (int p = u.ceMask; p; p &= p - 1)
    {
        const int i = ctz(p);
        const int index = getValIndexFromMask(u.ceMask, i);
        prefix->addr8[depth / 2] |= depth & 1 ? i : i << 4;
        if (u.isLeaf)
        {
            volatile RIPngRTE *rte = generateRTE(u.ceAddr.extraEntryAddr, &u.ceMask, i, index);
            rte->prefix = *prefix;
            rte->len = (depth + 1) * 4;
        }
        else
            generateRipngResponseDfs(u.ceAddr.childAddr + index, depth + 1, prefix);
        prefix->addr8[depth / 2] = depth & 1 ? prefix->addr8[depth / 2] & 0xf0 : 0;
    }

    writeNode(uAddr, &u, depth);
}

void generateRipngResponse()
{
    Ipv6Address prefix;
    for (int i = 0; i < 4; ++i)
        prefix.addr32[i] = 0;

    generateRipngResponseDfs(root, 0, &prefix);

    for (int i = 0; i < ripngGcCount; ++i)
    {
        if (!queryExistence(&ripngGcList[i]))
        {
            volatile RIPngRTE *rte = nextRTE(0, RIPNG_METRIC_INF);
            rte->prefix = ripngGcList[i].addr;
            rte->len = ripngGcList[i].len;
        }
    }
    ripngGcCount = 0;

    flushRIPngResponse();

    printf("Forwarding table entry count: %d\r\n", entryCount);
}
