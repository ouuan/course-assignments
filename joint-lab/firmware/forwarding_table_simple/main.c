#include <arpa/inet.h>
#include <stdio.h>
#include <string.h>

#include "../include/forwarding_table.h"

void inputAddr(Ipv6Address* addr) {
  static struct in6_addr dst;
  static char dstStr[50];
  scanf("%s", dstStr);
  inet_pton(AF_INET6, dstStr, &dst);
  memcpy(addr, &dst, 16);
}

void inputEntry(ForwardingTableEntry* entry) {
  inputAddr(&entry->dst);
  scanf("%hhu", &entry->len);
  scanf("%hhu", &entry->direct);
  inputAddr(&entry->nextHop);
  scanf("%hhu", &entry->interface);
}

void outputAddr(Ipv6Address* addr) {
  static struct in6_addr dst;
  static char dstStr[50];
  memcpy(&dst, addr, 16);
  inet_ntop(AF_INET6, &dst, dstStr, 50);
  printf("%s ", dstStr);
}

int main() {

  int n;
  scanf("%d", &n);

  initForwardingTable();

  ForwardingTableEntry entry;
  Ipv6Address dst;
  PrefixQueryResult result;

  while (n--) {
    int op;
    scanf("%d", &op);
    if (op == 1) { // insert
      inputEntry(&entry);
      if (insertForwardingTableEntry(entry))
        return 1;
    } else if (op == 2) { // delete
      inputEntry(&entry);
      deleteForwardingTableEntry(entry);
    } else if (op == 3) { // query
      inputAddr(&dst);
      result = prefixQuery(&dst);
      if (result.valid) {
        printf("%u %u %u ", result.valid, result.direct, result.interface);
        if (!result.direct) outputAddr(&result.nextHop);
      } else {
        printf("Not found");
      }
      printf("\n");
    }
  }

  return 0;

}
