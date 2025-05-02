#include <stdio.h>
#include <time.h>
#include <stdlib.h>
#include <stdint.h>
#include <arpa/inet.h>
#include <string.h>

typedef union {
    uint32_t addr32[4];
    uint16_t addr16[8];
    uint8_t addr8[16];
} Ipv6Address;

Ipv6Address randomAddr[10000];
int randomAddrCount;

typedef struct {
    Ipv6Address dst;
    uint8_t len;
    uint8_t interface;
    uint8_t direct;  // 1 if direct routing, 0 otherwise
    Ipv6Address nextHop;
} ForwardingTableEntry;

void outputAddr(Ipv6Address* addr) {
  static struct in6_addr dst;
  static char dstStr[50];
  memcpy(&dst, addr, 16);
  inet_ntop(AF_INET6, &dst, dstStr, 50);
  printf("%s ", dstStr);
}

void outputEntry(ForwardingTableEntry* entry) {
  outputAddr(&entry->dst);
  printf("%u ", entry->len);
  printf("%u ", entry->direct);
  outputAddr(&entry->nextHop);
  printf("%u ", entry->interface);
}

void genrandomAddr(Ipv6Address* addr, int len) {
    *addr = randomAddr[rand() % randomAddrCount];
    for (int i = len; i < 128; ++i)
        addr->addr8[i / 8] &= ~(1 << (i % 8));
}

void genrandomEntry(ForwardingTableEntry* entry) {
  entry->direct = rand() & 1;
  entry->len = (rand() % 16) + (rand() % 16 ? 32 : 16);
  entry->interface = rand() & 3;
  genrandomAddr(&entry->dst, entry->len);
  entry->nextHop = randomAddr[rand() % 7];
}

ForwardingTableEntry table[10000];

int main() {
  srand((unsigned)time(NULL));

  int q = rand() % 1000;

  randomAddrCount = rand() % 10000;

  for (int i = 1; i < randomAddrCount; ++i) {
      randomAddr[i] = randomAddr[rand() % i];
      randomAddr[i].addr16[rand() % 2 + 1] ^= 1 << (rand() % 16);
  }

  ForwardingTableEntry entry;
  int cnt = 0;
  Ipv6Address addr;

  printf("%d\n", q);
  while (q --) {
    int op = rand() % 3 + 1;
    printf("%d ", op);
    if (op == 1) { // insert
      genrandomEntry(&entry);
      table[cnt++] = entry;
      outputEntry(&entry);
    } else if (op == 2) { // erase
      if (cnt == 0 || (rand() & 1)) {
        genrandomEntry(&entry);
        outputEntry(&entry);
      } else {
        int i = rand() % cnt;
        outputEntry(&table[i]);
      }
    } else if (op == 3) { // query
      if (cnt == 0 || rand() & 1) {
        genrandomAddr(&addr, 128);
      } else {
        addr = table[rand() % cnt].dst;
      }
      outputAddr(&addr);
    }
    printf("\n");
  }
  return 0;
}
