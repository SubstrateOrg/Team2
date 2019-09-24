# kitties_count 溢出的解决
使用`u64`类型，并且按照逐个加1的方式计数，避免溢出。

# 设计加密猫

## 繁殖小猫

### 可调用函数

```rust
pub fn breed(origin, kitty1: u32, kitty2: u32)
```

### 生成小猫dna
```rust
variation = get_random_128bits();
for i in 0 .. 128 {
    r = get_random_under_100();
    if r < 45 {
        child[i] = parent1[i];
    } else if r < 90 {
        child[i] = parent2[i];
    } else {
        child[i] = variation[i];
    }
}
```