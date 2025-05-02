#pragma once

#include "dma.h"

#define RIPNG_METRIC_INF 16

typedef struct
{
    Ipv6Address prefix;
    uint16_t tag;
    uint8_t len;
    uint8_t metric;
} RIPngRTE;

typedef struct
{
    uint16_t srcPort;
    uint16_t dstPort;
    uint16_t length;
    uint16_t checksum;
} UDPHeader;

typedef struct
{
    uint8_t command;
    uint8_t version;
    uint16_t zero;
} RIPngHeader;

void receiveRIPngResponse(uint16_t len, uint8_t iface);

// should write prefix and len into the returned pointer, no need to write tag and metric
volatile RIPngRTE *nextRTE(int interface, uint8_t metric);
void flushRIPngResponse();

void initRIPng();
void runRIPng();
