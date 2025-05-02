#include "../include/forwarding_table.h"

struct trie_node
{
  int nxt[2];
  int flag;
  ForwardingTableEntry entry;
} T[2000000];

int tot;

void initForwardingTable() {
  tot = 1;
}

int entryCmp(ForwardingTableEntry* a, ForwardingTableEntry* b) {
  return addrCmp(&a->dst, &b->dst) || (!a->direct && !b->direct && addrCmp(&a->nextHop, &b->nextHop)) || (a->direct != b->direct || a->interface != b->interface || a->len != b->len);
}

int insertForwardingTableEntry(ForwardingTableEntry entryInfo) {
  if (entryInfo.len == 0) {
    T[1].flag = 1;
    T[1].entry = entryInfo;
    return 0;
  }
  int now = 1;
  for (int i = 0; i < entryInfo.len; ++i) {
    int v = (entryInfo.dst.addr8[i / 8] >> (7 - (i & 7))) & 1;
    if (T[now].nxt[v] == 0) {
      T[now].nxt[v] = ++tot;
      T[tot].nxt[0] = T[tot].nxt[1] = T[tot].flag = 0;
    }
    now = T[now].nxt[v];
  }
  T[now].flag = 1;
  T[now].entry = entryInfo;
  return 0;
}

int deleteForwardingTableEntry(ForwardingTableEntry entryInfo) {
  int now = 1;
  for (int i = 0; i < entryInfo.len; ++i) {
    int v = (entryInfo.dst.addr8[i / 8] >> (7 - (i & 7))) & 1;
    if (T[now].nxt[v] == 0) return 1;
    now = T[now].nxt[v];
  }
  T[now].flag = 0;
  return 0;
}

PrefixQueryResult prefixQuery(Ipv6Address *dst) {
  PrefixQueryResult ret;
  ret.valid = 0;
  int now = 1;
  for (int i = 0; i < 128; ++i) {
    int v = (dst->addr8[i / 8] >> (7 - (i & 7))) & 1;
    if (T[now].nxt[v] == 0) break;
    now = T[now].nxt[v];
    if (T[now].flag) {
      ret.valid = 1;
      ret.direct = T[now].entry.direct;
      ret.interface = T[now].entry.interface;
      ret.nextHop = T[now].entry.nextHop;
    }
  }
  return ret;
}
