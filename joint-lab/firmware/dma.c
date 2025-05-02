#include "ra.h"
#include "ripng.h"
#include "stdio.h"

DMAStatus __attribute__((section(".dma.status"))) dmaStatus;

static uint8_t __attribute__((section(".dma.router2cpu"))) inputBuffer[1 << 20];
uint8_t *currentInput;

volatile uint8_t __attribute__((section(".dma.cpu2router")))
outputBuffer[MTU + sizeof(EthernetHeader)];
static volatile uint8_t *previousOutputEnd;

uint16_t ntohs(uint16_t x)
{
    return ((x & 0xff) << 8) | (x >> 8);
}

uint16_t htons(uint16_t x)
{
    return ntohs(x);
}

void initDMA()
{
    currentInput = inputBuffer;

    // reset does not clear BRAM and checksum
    previousOutputEnd = outputBuffer + MTU + sizeof(EthernetHeader);

    initRIPng();
    initRA();
}

uint32_t getTime()
{
    return *(volatile uint32_t *)0x200bff8;
}

void clearOutputSuffix(volatile void *outputEnd)
{
    for (volatile uint32_t *p = outputEnd; (uint8_t *)p < previousOutputEnd; ++p)
        *p = 0;
    previousOutputEnd = outputEnd;
}

static void receivePacket(uint16_t len)
{
    if (len < sizeof(Ipv6Header) + sizeof(EthernetHeader))
        return;

    EthernetHeader *ether = (void *)currentInput;
    const uint8_t iface = ether->dst.addr8[5];

    Ipv6Header *ip6 = (void *)(ether + 1);

    const uint16_t plen = ntohs(ip6->payloadLength);
    if (plen + sizeof(Ipv6Header) + sizeof(EthernetHeader) != len)
        return;

    switch (ip6->nextHeader)
    {
        case UDP_NEXT_HEADER:
        {
            if (plen < sizeof(UDPHeader))
                return;

            UDPHeader *udp = (void *)(ip6 + 1);

            if (udp->length != ip6->payloadLength)
                return;
            if (udp->checksum == 0)
                return;  // IPv6 receivers must discard UDP packets containing a zero checksum

            if (udp->dstPort == RIPNG_PORT_BIG_ENDIAN)
                receiveRIPngResponse(plen, iface);

            break;
        }
    }
}

void dmaDaemon()
{
    int receivedCount = 0;

    while (1)
    {
        runRIPng();
        runRA();

        volatile uint16_t *lenp = (uint16_t *)currentInput;
        const uint16_t len = *lenp;
        if (len)
        {
            ++receivedCount;
            if (receivedCount % 256 == 0)
                printf("Received %d packets\r\n", receivedCount);
            receivePacket(len);
            *lenp = 0;
            currentInput += 1 << 11;
            if (currentInput == inputBuffer + (1 << 20))
                currentInput = inputBuffer;
        }
    }
}
