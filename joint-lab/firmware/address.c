#include "common.h"

#define IP_PART(a) ((a) >> 8), ((a)&0xff)

#define IP(a0, a1, a2, a3, a4, a5, a6, a7) \
    {                                      \
        .addr8 = {                         \
            IP_PART(a0),                   \
            IP_PART(a1),                   \
            IP_PART(a2),                   \
            IP_PART(a3),                   \
            IP_PART(a4),                   \
            IP_PART(a5),                   \
            IP_PART(a6),                   \
            IP_PART(a7),                   \
        }                                  \
    }

const MACAddress
#ifdef RV32
    __attribute__((section(".addresses.mac")))
#endif
    interfaceMacAddr[4] = {
        {.addr8 = {0x8c, 0x1f, 0x64, 0x69, 0x10, BASE_MAC5 + 0}},
        {.addr8 = {0x8c, 0x1f, 0x64, 0x69, 0x10, BASE_MAC5 + 1}},
        {.addr8 = {0x8c, 0x1f, 0x64, 0x69, 0x10, BASE_MAC5 + 2}},
        {.addr8 = {0x8c, 0x1f, 0x64, 0x69, 0x10, BASE_MAC5 + 3}},
};

const Ipv6Address
#ifdef RV32
    __attribute__((section(".addresses.linklocal")))
#endif
    interfaceLinkLocalAddr[4] = {
#ifdef BASE_LL7
        IP(0xfe80, 0, 0, 0, 0x8e1f, 0x64ff, 0xfe69, BASE_LL7 + 0),
        IP(0xfe80, 0, 0, 0, 0x8e1f, 0x64ff, 0xfe69, BASE_LL7 + 1),
        IP(0xfe80, 0, 0, 0, 0x8e1f, 0x64ff, 0xfe69, BASE_LL7 + 2),
        IP(0xfe80, 0, 0, 0, 0x8e1f, 0x64ff, 0xfe69, BASE_LL7 + 3),
#else
        IP(0xfe80, 0, 0, 0, 0, 0, 0, 1),
        IP(0xfe80, 0, 0, 0, 0, 0, 0, 1),
        IP(0xfe80, 0, 0, 0, 0, 0, 0, 1),
        IP(0xfe80, 0, 0, 0, 0, 0, 0, 1),
#endif
};

const Ipv6Address directRoutes[4] = {
    IP(0x2a0e, 0xaa06, DIRECT2, BASE_DIRECT3 + 0, 0, 0, 0, 0),
    IP(0x2a0e, 0xaa06, DIRECT2, BASE_DIRECT3 + 1, 0, 0, 0, 0),
    IP(0x2a0e, 0xaa06, DIRECT2, BASE_DIRECT3 + 2, 0, 0, 0, 0),
    IP(0x2a0e, 0xaa06, DIRECT2, BASE_DIRECT3 + 3, 0, 0, 0, 0),
};
