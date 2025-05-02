#pragma once

#include "common.h"

typedef struct
{
    uint8_t type;
    uint8_t length;
    MACAddress addr;
} LinkLayerAddressOption;

typedef struct
{
    uint8_t type;
    uint8_t length;
    uint8_t prefixLength;
    uint8_t flags;
    uint32_t validLifetime;
    uint32_t preferredLifetime;
    uint32_t reserved2;
    Ipv6Address prefix;
} PrefixInfoOption;

typedef struct
{
    uint8_t type;
    uint8_t code;
    uint16_t checksum;
    uint8_t curHopLimit;
    uint8_t flags;
    uint16_t routerLifetime;
    uint32_t reachableTime;
    uint32_t retransTimer;
    LinkLayerAddressOption sourceLLAddr;
    PrefixInfoOption prefixInfo;
} RouterAdvertisement;

void initRA();
void runRA();
