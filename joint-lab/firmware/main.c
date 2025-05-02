#include <stdio.h>

#include "address.h"
#include "forwarding_table.h"
#include "ripng.h"

extern uint32_t _bss_begin[];
extern uint32_t _bss_end[];

void start(void)
{
    for (uint32_t *p = _bss_begin; p != _bss_end; ++p)
    {
        *p = 0;
    }

    init_uart();
    initForwardingTable();
    initDMA();

    // add direct routes
    for (int i = 0; i < 4; ++i)
    {
        RoutingTableEntry rte;
        rte.prefix = &directRoutes[i];
        rte.nextHop = &interfaceLinkLocalAddr[i];  // actually not used
        rte.len = 64;
        rte.interface = i;
        rte.metricWithCost = 1;
        rte.direct = 1;
        updateForwardingTable(rte);
    }

    printf("\r\nInitialization finished.\r\n");

    dmaDaemon();
}
