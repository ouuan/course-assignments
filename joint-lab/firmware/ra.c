#include "ra.h"

#include <printf.h>

#include "address.h"
#include "dma.h"

#define ICMP6_NEXT_HEADER 58
#define RA_TYPE 134

#define MIN_RA_INTERVAL 3000000
#define RA_INTERVAL_RANGE (1 << 22)

static uint32_t randState;

// https://gist.github.com/tommyettinger/46a874533244883189143505d203312c
static uint32_t rand()
{
    uint32_t z = (randState += 0x6D2B79F5UL);
    z = (z ^ (z >> 15)) * (z | 1UL);
    z ^= z + (z ^ (z >> 7)) * (z | 61UL);
    return z ^ (z >> 14);
}

static uint32_t previousTime;
static uint32_t currentInterval;

void initRA()
{
    randState = interfaceLinkLocalAddr[0].addr32[3];
    previousTime = getTime();
}

static const Ipv6Address ALL_NODES_MULTICAST_IP = {
    .addr8 = {0xff, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x01}};
static const MACAddress ALL_NODES_MULTICAST_MAC = {.addr8 = {0x33, 0x33, 0, 0, 0, 0x01}};

static void sendRA()
{
    while (dmaStatus.cpuToRouterLength)
        ;

    volatile EthernetHeader *ether = (void *)outputBuffer;
    ether->ethertype = IP6_ETHERTYPE_BIG;
    ether->dst = ALL_NODES_MULTICAST_MAC;

    volatile Ipv6Header *ip6 = (void *)(ether + 1);
    ip6->version = 0x60;
    ip6->nextHeader = ICMP6_NEXT_HEADER;
    ip6->hopLimit = 255;
    ip6->dst = ALL_NODES_MULTICAST_IP;
    ip6->payloadLength = htons(sizeof(RouterAdvertisement));

    volatile RouterAdvertisement *ra = (void *)(ip6 + 1);
    ra->type = RA_TYPE;
    ra->code = 0;
    ra->curHopLimit = 0;
    ra->flags = 0;
    ra->routerLifetime = 0x1000;
    ra->reachableTime = 0;
    ra->retransTimer = 0;
    ra->sourceLLAddr.type = 1;
    ra->sourceLLAddr.length = 1;
    ra->prefixInfo.type = 3;
    ra->prefixInfo.length = 4;
    ra->prefixInfo.prefixLength = 64;
    ra->prefixInfo.flags = 0xc0;
    ra->prefixInfo.validLifetime = 0x20000000;
    ra->prefixInfo.preferredLifetime = 0x10000000;
    ra->prefixInfo.reserved2 = 0;

    clearOutputSuffix(ra + 1);

    for (int iface = 0; iface < 4; ++iface)
    {
        while (dmaStatus.cpuToRouterLength)
            ;

        ether->src = interfaceMacAddr[iface];
        ip6->src = interfaceLinkLocalAddr[iface];
        ra->checksum = 0;
        ra->sourceLLAddr.addr = interfaceMacAddr[iface];
        ra->prefixInfo.prefix = directRoutes[iface];
        ra->checksum = ~dmaStatus.cpuToRouterChecksum;
        dmaStatus.cpuToRouterLength =
            sizeof(EthernetHeader) + sizeof(Ipv6Header) + sizeof(RouterAdvertisement);
    }
}

void runRA()
{
    if (getTime() - previousTime >= currentInterval)
    {
        previousTime = getTime();
        currentInterval = MIN_RA_INTERVAL + rand() % RA_INTERVAL_RANGE;
        sendRA();
        printf("Sent RA (%d).\r\n", currentInterval);
    }
}
