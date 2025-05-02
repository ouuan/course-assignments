#pragma once

#include "common.h"
#include "ripng.h"

typedef struct
{
    const Ipv6Address *prefix;
    const Ipv6Address *nextHop;
    uint8_t len;
    uint8_t interface;
    uint8_t metricWithCost;
    uint8_t direct;  // 1 if direct routing, 0 otherwise
} RoutingTableEntry;

/*
 * Storage layout [130:0]:
 *         direct
 * [1:0]   interface
 * [127:0] nextHop
 */
typedef struct
{
    Ipv6Address nextHop;
    uint32_t directAndInterface;
    uint32_t _padding[3];
} NextHopData;

void initForwardingTable();

void updateForwardingTable(RoutingTableEntry rte);

void generateRipngResponse();

typedef uint8_t NextHopId;

typedef struct
{
    //        isLeaf
    // [14:0] entryMask
    // [15:0] childMask
    uint32_t masks;
    // [13:0] unused
    // [17:0] ceAddr
    uint32_t ceAddr;
    // [13:0] unused
    // [17:0] entryAddr
    uint32_t entryAddr;
    uint32_t _padding;
} BitmapNodeData;

/*
 * Storage layout [67:0]:
 * [17:0] entryAddr
 * [17:0] ceAddr
 *        isLeaf
 * [14:0] entryMask
 * [15:0] childMask
 */
typedef struct
{
    // A node is temporarily invalid during updating to avoid reading wrong data.
    uint8_t isLeaf;
    NextHopId *entryAddr;
    union
    {
        BitmapNodeData *childAddr;  // when isLeaf == 0
        NextHopId *extraEntryAddr;  // when isLeaf == 1
    } ceAddr;
    uint32_t entryMask;  // 000, 001, 010, 011, 100, 101, 110, 111, 00, 01, 10, 11, 0, 1, *
    uint32_t ceMask;
} BitmapNode;

typedef uint16_t RIPngData;

typedef struct
{
    uint8_t metric;
    uint8_t timeout;
} RIPngInfo;
