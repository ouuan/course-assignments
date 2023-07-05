/*
 * This solution uses segregated fits.
 *
 * Free block:
 * - header: 64 bits
 *   - block size: 62 bits
 *   - previous block allocated: 1 bit
 *   - current block allocated: 1 bit
 * - previous block in the free list: 64 bits
 * - next block in the free list: 64 bits
 * - blank
 * - footer (block size): 64 bits
 *
 * Allocated block: (no list pointers, no footer)
 * - header: 64 bits
 * - payload
 *
 * Beginning of the heap: sentinels of the free lists
 *
 * Yufan You,
 * 2023.1.12
 */

#include "mm.h"

#include <string.h>

#include "memlib.h"

/*********************************************************
 * NOTE TO STUDENTS: Before you do anything else, please
 * provide your team information in the following struct.
 ********************************************************/
team_t team = {
    /* Team name */
    "ouuan",
    /* First member's full name */
    "ouuan",
    /* First member's student ID */
    "ouuan",
    /* Second member's full name (leave blank if none) */
    "",
    /* Second member's student ID (leave blank if none) */
    ""};

// sizes
#define W_SIZE (sizeof(size_t))
#define D_SIZE (2 * W_SIZE)
// min block size: header, prev, next, footer
#define MIN_SIZE (4 * W_SIZE)
#define ALIGNMENT D_SIZE
#define ALIGN(size) (((size) + (ALIGNMENT - 1)) & ~(ALIGNMENT - 1))

// accessing memory
#define DEREF(p) (*(size_t *)(p))
#define BLOCK_SIZE(hp) (DEREF(hp) & ~7)
#define ALLOCATED(hp) (DEREF(hp) & 1)
#define PAYLOAD(hp) ((size_t *)(hp) + 1)
#define FOOTER(hp) ((char *)(hp) + BLOCK_SIZE(hp) - W_SIZE)
#define HEADER(bp) ((size_t *)(bp)-1)
#define PREV_ALLOCATED(hp) (DEREF(hp) & 2)
#define PREV_HEADER(hp) ((char *)(hp) - *((size_t *)(hp)-1))
#define NEXT_HEADER(hp) ((char *)(hp) + BLOCK_SIZE(hp))
#define LIST_PREV(hp) (*((void **)(hp) + 1))
#define LIST_NEXT(hp) (*((void **)(hp) + 2))

// lists
#define MIN_POWER 4
#define MAX_POWER 18
#define LOG2(size) (8 * sizeof(long long) - __builtin_clzll((size)-1))
#define SENTINEL(k)            \
    ((size_t *)mem_heap_lo() + \
     ((k) < MAX_POWER ? (k)*2 - MIN_POWER * 2 - 1 : MAX_POWER * 2 - MIN_POWER * 2 - 1))

static inline size_t min(size_t x, size_t y)
{
    return x < y ? x : y;
}

static inline size_t max(size_t x, size_t y)
{
    return x > y ? x : y;
}

// remove *hp* from the corresponding free list
static void list_remove(void *hp)
{
    LIST_NEXT(LIST_PREV(hp)) = LIST_NEXT(hp);
    LIST_PREV(LIST_NEXT(hp)) = LIST_PREV(hp);
}

// add *hp* in the tail of the corresponding free list
static void list_insert(void *hp)
{
    void *sentinel = SENTINEL(LOG2(BLOCK_SIZE(hp)));
    void *prev = LIST_PREV(sentinel);
    LIST_PREV(sentinel) = hp;
    LIST_NEXT(prev) = hp;
    LIST_PREV(hp) = prev;
    LIST_NEXT(hp) = sentinel;
}

/*
 * @brief coalesce *hp* with adjacent free blocks
 * @param hp header to a newly freed block, which is not in any list yet
 * @param insert whether to insert into list after coalescing
 * @returns the new header after coalescing
 * @note the allocated bit of the coalesced block will be cleared even if no coalescing occurred,
 *       so the allocated bit of *hp* could be unset when calling this function
 */
static void *coalesce(void *hp, int insert)
{
    size_t coalesced_size = BLOCK_SIZE(hp);
    void *newhp = hp;

    if (!PREV_ALLOCATED(hp))
    {
        newhp = PREV_HEADER(hp);
        coalesced_size += BLOCK_SIZE(newhp);
        list_remove(newhp);
    }

    void *next = NEXT_HEADER(hp);
    if (!ALLOCATED(next))
    {
        coalesced_size += BLOCK_SIZE(next);
        list_remove(next);
    }

    DEREF(newhp) = coalesced_size | PREV_ALLOCATED(newhp);
    DEREF(FOOTER(newhp)) = coalesced_size;

    if (insert)
        list_insert(newhp);

    return newhp;
}

/*
 * @brief get more heap space and create a free block from the new space
 * @returns the header of the new free block, or NULL on error
 * @note the new free block will not be inserted into list
 */
static void *extend(size_t size)
{
    size_t *hp = mem_sbrk(size);
    if (hp == (void *)-1)
        return NULL;

    hp -= 1;
    DEREF(hp) = size | PREV_ALLOCATED(hp);
    DEREF((char *)hp + size) = 1;

    return coalesce(hp, 0);
}

/*
 * @param hp the header to be split
 * @param size the minimum size of *hp* after spliting
 * @param higher whether to use the lower part or the higher part
 */
static void *split(void *hp, size_t size, int higher)
{
    size_t free_size = BLOCK_SIZE(hp) - size;
    if (free_size >= MIN_SIZE)
    {
        if (higher)
        {
            DEREF(hp) = free_size | PREV_ALLOCATED(hp);
            DEREF(FOOTER(hp)) = free_size;
            list_insert(hp);
            hp = (char *)hp + free_size;
            DEREF(hp) = size | 1;
            DEREF(NEXT_HEADER(hp)) |= 2;
        }
        else
        {
            DEREF(hp) = size | PREV_ALLOCATED(hp) | 1;
            void *hq = (char *)hp + size;
            DEREF(hq) = free_size | 2;
            DEREF(FOOTER(hq)) = free_size;
            DEREF(NEXT_HEADER(hq)) &= ~2;
            list_insert(hq);
        }
    }
    return hp;
}

int mm_init(void)
{
    size_t end = ALIGN((size_t)mem_heap_lo() + ((MAX_POWER - MIN_POWER) * 2 + 3) * W_SIZE);
    if (mem_sbrk(end - (size_t)mem_heap_lo()) == (void *)-1)
        return -1;

    // initialize sentinels
    for (int k = MIN_POWER; k <= MAX_POWER; ++k)
    {
        void *sentinel = SENTINEL(k);
        LIST_NEXT(sentinel) = sentinel;
        LIST_PREV(sentinel) = sentinel;
    }

    // set epilogue
    DEREF(end - W_SIZE) = 3;

    return 0;
}

void *mm_malloc(size_t size)
{
    size_t block_size = max(ALIGN(size + W_SIZE), MIN_SIZE);

    void *hp;
    int found = 0;

    // find segregated fit
    for (int k = min(LOG2(block_size), MAX_POWER); k <= MAX_POWER && !found; ++k)
    {
        void *sentinel = SENTINEL(k);
        for (hp = LIST_NEXT(sentinel); hp != sentinel; hp = LIST_NEXT(hp))
        {
            if (BLOCK_SIZE(hp) >= block_size)
            {
                list_remove(hp);
                found = 1;
                break;
            }
        }
    }

    // request new heap memory when no fit found
    if (!found)
    {
        size_t extend_size = block_size;
        size_t *epilogue = (size_t *)((char *)mem_heap_hi() - W_SIZE + 1);
        if ((*epilogue & 2) == 0)
            extend_size -= *(epilogue - 1);  // minus the size of the last free block
        hp = extend(max(extend_size, 4096));
        if (hp == NULL)
            return NULL;
    }

    // set allocated and prev_allocated
    DEREF(hp) |= 1;
    DEREF(NEXT_HEADER(hp)) |= 2;

    // small blocks use the lower part, large blocks use the higher part
    return PAYLOAD(split(hp, block_size, block_size > 96));
}

void mm_free(void *ptr)
{
    if (ptr == NULL)
        return;

    // the allocated bit will be cleared in the `coalesce` function
    void *hp = coalesce(HEADER(ptr), 1);
    // set prev_allocated
    DEREF(NEXT_HEADER(hp)) &= ~2;
}

void *mm_realloc(void *ptr, size_t size)
{
    if (size == 0)
    {
        mm_free(ptr);
        return NULL;
    }
    if (ptr == NULL)
        return mm_malloc(size);

    size_t required_size = max(ALIGN(size + W_SIZE), MIN_SIZE);
    void *hp = HEADER(ptr);

    size_t coalesced_size = BLOCK_SIZE(hp);

    // use while instead of if to use break
    while (coalesced_size < required_size)
    {
        void *next = NEXT_HEADER(hp);
        if (!ALLOCATED(next))
        {
            coalesced_size += BLOCK_SIZE(next);
            list_remove(next);
        }

        if (coalesced_size >= required_size)
        {
            // coalesce into only the next block
            DEREF(NEXT_HEADER(next)) |= 2;
            DEREF(hp) += BLOCK_SIZE(next);
            break;
        }

        void *oldhp = hp;

        // coalesce into the previous block
        if (!PREV_ALLOCATED(hp))
        {
            hp = PREV_HEADER(hp);
            coalesced_size += BLOCK_SIZE(hp);
            list_remove(hp);
        }

        if (coalesced_size >= required_size)
        {
            // enough space after coalescing
            DEREF(hp) = coalesced_size | PREV_ALLOCATED(hp) | 1;
            DEREF(NEXT_HEADER(hp)) |= 2;
            memmove(PAYLOAD(hp), PAYLOAD(oldhp), BLOCK_SIZE(oldhp) - W_SIZE);
            break;
        }

        // space not enough after coalescing

        // pretend to be allocated to keep it untouched by malloc
        DEREF(hp) = coalesced_size | PREV_ALLOCATED(hp) | 1;
        DEREF(NEXT_HEADER(hp)) |= 2;
        // malloc and copy data
        void *bp = mm_malloc(size);
        if (bp == NULL)
            return NULL;
        memcpy(bp, PAYLOAD(oldhp), BLOCK_SIZE(oldhp) - W_SIZE);
        // free coalesced block
        DEREF(hp) &= ~1;
        DEREF(FOOTER(hp)) = coalesced_size;
        DEREF(NEXT_HEADER(hp)) &= ~2;
        list_insert(hp);
        return bp;
    }

    return PAYLOAD(split(hp, required_size, 0));
}
