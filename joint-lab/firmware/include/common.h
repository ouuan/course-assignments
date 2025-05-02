#pragma once

#include <stdint.h>

typedef union
{
    uint32_t addr32[4];
    uint16_t addr16[8];
    uint8_t addr8[16];
} Ipv6Address;

typedef struct
{
    Ipv6Address addr;
    uint8_t len;
} Ipv6Prefix;

typedef union
{
    uint16_t addr16[3];
    uint8_t addr8[6];
} MACAddress;

// @returns 0 if equal, 1 if unequal
int addrCmp(const Ipv6Address *lhs, const Ipv6Address *rhs);

// @returns 1 if link local, 0 if not
int isLinkLocalAddr(Ipv6Address *addr);

// @returns 1 if multicast, 0 if not
int isMulticastAddr(Ipv6Address *addr);
