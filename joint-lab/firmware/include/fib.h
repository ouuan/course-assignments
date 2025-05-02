#pragma once

#include "common.h"

typedef struct
{
    Ipv6Address prefix;
    uint8_t len;
    uint8_t interface;
    uint8_t direct;
} FibEntry;
