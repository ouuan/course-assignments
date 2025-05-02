#pragma once

#include "common.h"

#define MTU 1500
#define IP6_ETHERTYPE_BIG 0xdd86
#define UDP_NEXT_HEADER 17
#define RIPNG_PORT_BIG_ENDIAN 0x0902

typedef struct
{
    uint16_t padding;
    MACAddress dst;  // interface ID is stored in dst.addr8[5] in input
    MACAddress src;
    uint16_t ethertype;
} EthernetHeader;

typedef struct
{
    uint32_t version;
    uint16_t payloadLength;
    uint8_t nextHeader;
    uint8_t hopLimit;
    Ipv6Address src;
    Ipv6Address dst;
} Ipv6Header;

typedef struct
{
    volatile uint16_t cpuToRouterLength;
    const volatile uint16_t cpuToRouterChecksum;
} DMAStatus;

extern DMAStatus dmaStatus;
extern uint8_t *currentInput;

extern volatile uint8_t __attribute__((section(".dma.cpu2router")))
outputBuffer[MTU + sizeof(EthernetHeader)];

uint16_t ntohs(uint16_t x);
uint16_t htons(uint16_t x);

void initDMA();

uint32_t getTime();

void clearOutputSuffix(volatile void *outputEnd);

void dmaDaemon();
