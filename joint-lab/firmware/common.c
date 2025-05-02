#include "common.h"

int addrCmp(const Ipv6Address *lhs, const Ipv6Address *rhs)
{
    for (int i = 0; i < 4; ++i)
    {
        if (lhs->addr32[i] != rhs->addr32[i])
            return 1;
    }
    return 0;
}

int isLinkLocalAddr(Ipv6Address *addr)
{
    return addr->addr32[0] == 0x80fe && addr->addr32[1] == 0;
}

int isMulticastAddr(Ipv6Address *addr)
{
    return addr->addr8[0] == 0xff;
}
