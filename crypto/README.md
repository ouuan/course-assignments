## 现代密码学（2024）

鉴于：

1.  都是一些经典算法，并不是什么课程独创的任务
2.  一般人都会写 C++ 而不是 Rust，如果你也写 Rust，我们可能会都给助教留下深刻的印象（

感觉应该能公开放上来（，应该没法抄吧，或许可以用来对答案之类的（

### enigma

别的就图一乐，没必要写得像我这么认真（？

主要看这一行，感觉 Enigma 原理描述都不说人话，我拟合了半天才拟合出来（

```rust
self.permutation[input + self.position - self.ring_setting] - self.position + self.ring_setting
```

另外 double-stepping 我也拟合了一下扩展到了不一定 3 个 rotor，但谁知道不是 3 个 rotor 应该怎么样（

```rust
let mut should_turn = true;
let mut iter = self.rotors.iter_mut().rev().peekable();
while let Some(rotor) = iter.next() {
    let next_should_turn = iter.peek().is_some() && rotor.about_to_turnover();
    if should_turn || next_should_turn {
        rotor.turn();
    }
    should_turn = next_should_turn;
}
```

使用 `-h` 查看命令行参数帮助。示例：

-   `RUST_LOG=debug cargo run -r -- encrypt --rotor-types 231 --ring-settings des --initial-positions aaa --plugboard bx,gk,wy,ef,pq,sn ABC`
-   `cargo run -r -- crack ELCONWDIAPKSZHFBQTJYRGVXMU MRWJFDVSQEXUCONHBIPLTGAYZK WADFRPOLNTVCHMYBJQIGEUSKZX`

### baby_crypto

一些密码学算法的 Rust 实现，包括：

-   block cipher: AES-{128,192,256}
-   cipher mode: CTR, GCM
-   hash: SHA256, SHA3-{224,256,384,512}
-   HMAC
-   HKDF

最后要 link 到提供的 C++ 框架，我写了个 binding，但正常人哪怕熟悉 Rust 也会写 C++ 吧（

但是这些算法真的很适合写 trait（以及 const generic）啊（
