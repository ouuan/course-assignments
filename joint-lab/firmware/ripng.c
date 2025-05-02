#include "ripng.h"

#include <printf.h>

#include "address.h"
#include "forwarding_table.h"

#define MAX_RTE_COUNT 72
#define RIPNG_RESPONSE 2

void receiveRIPngResponse(uint16_t plen, uint8_t iface)
{
    EthernetHeader *ether = (void *)currentInput;

    Ipv6Header *ip6 = (void *)(ether + 1);

    if (ip6->hopLimit != 255)
        return;
    if (!isLinkLocalAddr(&ip6->src))
        return;
    if (addrCmp(&ip6->src, &interfaceLinkLocalAddr[iface]) == 0)
        return;

    UDPHeader *udp = (void *)(ip6 + 1);

    if (udp->srcPort != RIPNG_PORT_BIG_ENDIAN)
        return;
    if (plen < sizeof(UDPHeader) + sizeof(RIPngHeader))
        return;

    RIPngHeader *ripng = (void *)(udp + 1);

    if (ripng->command != RIPNG_RESPONSE)
        return;
    if (ripng->version != 1)
        return;
    if (ripng->zero != 0)
        return;

    const uint16_t rteLen = plen - sizeof(UDPHeader) - sizeof(RIPngHeader);
    if (rteLen % sizeof(RIPngRTE) != 0)
        return;

    const uint16_t rteCount = rteLen / sizeof(RIPngRTE);

    RoutingTableEntry rteInfo;
    rteInfo.nextHop = &ip6->src;
    rteInfo.interface = iface;
    rteInfo.direct = 0;

    RIPngRTE *rte = (void *)(ripng + 1);
    for (uint16_t i = 0; i < rteCount; ++i, ++rte)
    {
        if (rte->metric == 0 || rte->metric > RIPNG_METRIC_INF || rte->len > 128 ||
            isLinkLocalAddr(&rte->prefix) || isMulticastAddr(&rte->prefix))
            continue;  // > If any check fails, ignore that entry and proceed to the next.
        rteInfo.prefix = &rte->prefix;
        rteInfo.len = rte->len;
        rteInfo.metricWithCost = rte->metric + 1;
        updateForwardingTable(rteInfo);
    }
}

static const Ipv6Address RIPNG_MULTICAST_IP = {
    .addr8 = {0xff, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x09}};
static const MACAddress RIPNG_MULTICAST_MAC = {.addr8 = {0x33, 0x33, 0, 0, 0, 0x09}};

static volatile uint8_t *outputPos;
static volatile void *const rteStart = outputBuffer + sizeof(EthernetHeader) + sizeof(Ipv6Header) +
                                       sizeof(UDPHeader) + sizeof(RIPngHeader);
static int splitHorizonInterface[MAX_RTE_COUNT];
static uint8_t outputRteMetric[MAX_RTE_COUNT];
static int outputRteCount;

static void sendRIPngResponse()
{
    const uint16_t length = outputPos - outputBuffer;
    const uint16_t plen = length - sizeof(EthernetHeader) - sizeof(Ipv6Header);
    const uint16_t plenBig = htons(plen);

    while (dmaStatus.cpuToRouterLength)
        ;

    volatile EthernetHeader *ether = (void *)outputBuffer;
    ether->ethertype = IP6_ETHERTYPE_BIG;
    ether->dst = RIPNG_MULTICAST_MAC;

    volatile Ipv6Header *ip6 = (void *)(ether + 1);
    ip6->version = 0x60;
    ip6->nextHeader = UDP_NEXT_HEADER;
    ip6->hopLimit = 255;
    ip6->dst = RIPNG_MULTICAST_IP;
    ip6->payloadLength = plenBig;

    volatile UDPHeader *udp = (void *)(ip6 + 1);
    udp->srcPort = RIPNG_PORT_BIG_ENDIAN;
    udp->dstPort = RIPNG_PORT_BIG_ENDIAN;
    udp->length = plenBig;

    RIPngHeader *ripng = (void *)(udp + 1);
    ripng->command = RIPNG_RESPONSE;
    ripng->version = 1;
    ripng->zero = 0;

    clearOutputSuffix(outputPos);

    for (int interface = 0; interface < 4; ++interface)
    {
        while (dmaStatus.cpuToRouterLength)
            ;

        ether->src = interfaceMacAddr[interface];
        ip6->src = interfaceLinkLocalAddr[interface];
        udp->checksum = 0;

        volatile RIPngRTE *rte = rteStart;
        for (int i = 0; i < outputRteCount; ++i, ++rte)
        {
            rte->metric =
                interface == splitHorizonInterface[i] ? RIPNG_METRIC_INF : outputRteMetric[i];
        }

        const uint16_t checksum = dmaStatus.cpuToRouterChecksum;
        udp->checksum = checksum == 0xffff ? checksum : ~checksum;

        dmaStatus.cpuToRouterLength = length;
    }

    outputPos = rteStart;
    outputRteCount = 0;
}

volatile RIPngRTE *nextRTE(int interface, uint8_t metric)
{
    if (outputRteCount >= MAX_RTE_COUNT)
        sendRIPngResponse();
    splitHorizonInterface[outputRteCount] = interface;
    outputRteMetric[outputRteCount] = metric;
    ++outputRteCount;
    volatile void *ret = outputPos;
    outputPos += sizeof(RIPngRTE);
    return ret;
}

void flushRIPngResponse()
{
    if (outputPos != rteStart)
        sendRIPngResponse();
}

static uint32_t previousTime;

void initRIPng()
{
    previousTime = getTime();
    outputPos = rteStart;
}

void runRIPng()
{
    if (getTime() - previousTime >= RIPNG_UPDATE_TIME)
    {
        previousTime += RIPNG_UPDATE_TIME;
        generateRipngResponse();
    }
}
