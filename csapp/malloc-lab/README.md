# CS:APP Malloc Lab

https://csapp.cs.cmu.edu/3e/labs.html

## 实现思路

主体上使用 segregated fits 来实现。

### 数据结构

按 log2 block size 分成若干个 free list，每个节点记录前驱后继，每个 free list 中有一个哨兵节点同时作为第一个节点的前驱和最后一个节点的后继。

### 内存布局

在 free block 中记录 header、footer 和链表的前驱后继，而在 allocated block 中只记录 header 来节约空间。在 header 中存放相邻的上一个 block 的 allocated bit，这样的话即使 allocated block 没有 footer 也可以进行相邻 free block 的合并。

free block:

-   header: 64 bits
    -   block size: 62 bits
    -   previous block allocated: 1 bit
    -   current block allocated: 1 bit
-   pointer to previous block in the free list: 64 bits
-   pointer to next block in the free list: 64 bits
-   blank
-   footer (block size): 64 bits

allocated block:

-   header: 64 bits
    -   block size: 62 bits
    -   previous block allocated: 1 bit
    -   current block allocated: 1 bit
-   payload
-   padding (optional)

在 heap 的开头存放每个 free list 的哨兵节点。

在 heap 的结尾放有一个 size 为 0、allocated 的 epilogue block，来避免边界情况的特殊处理。由于相邻上一个 block 的 allocated bit 存在 header 中，不需要 prologue block，只需将开头的第一个 block 设为上一个 block allocated 即可。

### `mm_init`

获取哨兵节点和 epilogue 所需的空间，初始化哨兵节点和 epilogue。

### `mm_malloc`

从所属 size 类别开始依次遍历每个 free list，直到找到大小足够的 block，如果没找到则申请新的 heap 空间。

找到合适的 free block 后，allocate 并 split。

### `mm_free`

设为 free，前后合并。

### `mm_realloc`

首先特判 `size == 0`、`ptr == NULL`。

然后找到新的 block 需要使用的 block：

1.  如果原来的 block 就足够大，则使用原来的 block；
2.  否则，如果和相邻的下一个 free block 合并后足够大，则合并；
3.  否则，如果和相邻两侧的 free block 合并后足够大，则合并后 `memmove`；
4.  否则，`mm_malloc`、`memcpy`、free。

最后 split。

### 细节优化

-   在 split 时，根据 block size 的大小来选择 allocate 到开头或结尾（前半或后半），这样就可以使大小相近的 block 位置相邻，减少 external fragmentation。（但是 realloc 时只 allocate 到开头。）
-   在获取更多 heap space 时：
    -   设置一个 chunk size，每次最少获取一个 chunk。
    -   在需要的 block size 超过 chunk size 时，若位于末尾的最后一个 block free，则只申请需要的 block size 减去最后一个 block 的 size。
-   不设置 64 个 free list，而是只设置 $2^4 \sim 2^{18}$，减少无用的枚举。

## 结果分析

```
Results for mm malloc:
trace            name     valid  util     ops      secs   Kops
 1     amptjp-bal.rep       yes   99%    5694  0.000239  23854
 2       cccp-bal.rep       yes   99%    5848  0.000213  27443
 3    cp-decl-bal.rep       yes   99%    6648  0.000258  25817
 4       expr-bal.rep       yes   99%    5380  0.000212  25318
 5 coalescing-bal.rep       yes   97%   14400  0.000257  55944
 6     random-bal.rep       yes   94%    4800  0.000488   9830
 7    random2-bal.rep       yes   94%    4800  0.000440  10912
 8     binary-bal.rep       yes   91%   12000  0.000397  30204
 9    binary2-bal.rep       yes   81%   24000  0.000794  30208
10    realloc-bal.rep       yes   94%   14401  0.000663  21711
11   realloc2-bal.rep       yes   97%   14401  0.000263  54819
Total                             95%  112372  0.004225  26596

Score = (57 (util) + 40 (thru)) * 11/11 (testcase) = 97/100
```

`binary2-bal.rep` 的内存利用率较低，是因为这个测试点的 payload size 较小，internal fragmentation 较为严重，实际上这个测试点的 external fragmentation 已经接近 0 了。

两个 random 测试点的内存利用率和吞吐量都相对较低，可能是因为不规则的 payload 造成了相对严重的 external fragmentation，并且 block size 大，内存访问的空间局部性也相对较差。
