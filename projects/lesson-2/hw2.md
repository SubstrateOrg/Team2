# 数据结构
```rust
pub struct Kitty {
    dna: [u8; 16]
}
```

# 存储定义：
```rust
Kitties: map u64 => Kitty
KittiesCount: u64
OwnedKitties: map (AccountId, u64) => u64
OwnedKittiesCount: map AccountId => u64
```

# 可调用函数
```rust
create()
```

# dna 生成伪代码
```rust
payload = sender_addr + transaction_index
for i in 1 .. 80
    payload += block_hash(current_height - i)
dna = first_16_bytes(hash(payload))
```
