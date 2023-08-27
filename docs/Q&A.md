# Q&A

## defmt-test 编译异常

### 报错信息

```shell
  = note: rust-lld: error: memory.x:4: region 'FLASH' already defined
          >>>   FLASH : ORIGIN = 0x08000000, LENGTH = 64K
          >>>                                         ^

          flip-link: the native linker failed to link the program normally; please check your project configuration and linker scripts
```

### 解决方案

在 .cargo/config.toml 文件中包含了两个目标都生效了, 导致 `memory.x` 被读取了两次。

参考资料:

- [region 'FLASH' already defined](https://github.com/rust-embedded/cortex-m/issues/413)

```toml
[target.thumbv7m-none-eabi]
# runner = "qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel"
# runner = 'arm-none-eabi-gdb'
rustflags = [
    "-C",
    "linker=flip-link",
    "-C",
    "link-arg=-Tlink.x",
    # This is needed if your flash or ram addresses are not aligned to 0x10000 in memory.x
    # See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
    "-C",
    "link-arg=--nmagic",
    "-C",
    "link-arg=-Tdefmt.x",
]

# 该配置只能写一份, 否则 memory.x 会导致异常
# [target.'cfg(all(target_arch = "arm", target_os = "none"))']
# # TODO: replace `$CHIP` with your chip's name (see `probe-run --list-chips` output)
# runner = "probe-run --chip STM32F103C8"
# rustflags = [
#     "-C",
#     "linker=flip-link",
#     "-C",
#     "link-arg=-Tlink.x",
#     "-C",
#     "link-arg=-Tdefmt.x",
#     # This is needed if your flash or ram addresses are not aligned to 0x10000 in memory.x
#     # See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
#     "-C",
#     "link-arg=--nmagic",
# ]

```

## cargo test --target thumbv7m-none-eabi -p testsuite 单元测试异常

### 问题

```shell
    Finished test [optimized + debuginfo] target(s) in 0.04s
     Running unittests src/lib.rs (target/thumbv7m-none-eabi/debug/deps/testsuite-5c8e8671ba35a44f)
error: test failed, to rerun pass `-p testsuite --lib`

Caused by:
  could not execute process `/home/one/Documents/code/RustEmbedProject/stm32f103-tutorial/target/thumbv7m-none-eabi/debug/deps/testsuite-5c8e8671ba35a44f` (never executed)

Caused by:
  Exec format error (os error 8)
```

### 解决

修改 .cargo/config.toml 文件，添加 runner 配置。

```toml
[target.thumbv7m-none-eabi]
# QEUM
# runner = "qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel"
# GDB
# runner = 'arm-none-eabi-gdb'
# 真机测试
runner = "probe-run --chip STM32F103C8"  # <--- 取消注释
rustflags = [
    "-C",
    "linker=flip-link",
    "-C",
    "link-arg=-Tlink.x",
    # This is needed if your flash or ram addresses are not aligned to 0x10000 in memory.x
    # See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
    "-C",
    "link-arg=--nmagic",
    "-C",
    "link-arg=-Tdefmt.x",
]
```

### 解决 2

执行以下指令进行替换。

```shell
cargo test --target thumbv7m-none-eabi -p testsuite probe-run -- --chip STM32F103C8
```

## cannot find linker script defmt.x

### 错误

```shell
  = note: rust-lld: error: cannot find linker script defmt.x

          flip-link: the native linker failed to link the program normally; please check your project configuration and linker scripts


error: aborting due to previous error

       Error Failed to run cargo build: exit code = Some(101).
```

### 解决

defmt 与 embed 的 crate 存在冲突，因此需要注释掉 defmt 的配置。

```shell
[target.thumbv7m-none-eabi]
# QEUM
# runner = "qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -semihosting-config enable=on,target=native -kernel"
# GDB
# runner = 'arm-none-eabi-gdb'
# 真机测试
# runner = "probe-run --chip STM32F103C8"
rustflags = [
    "-C",
    "linker=flip-link",
    "-C",
    "link-arg=-Tlink.x",
    # This is needed if your flash or ram addresses are not aligned to 0x10000 in memory.x
    # See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
    "-C",
    "link-arg=--nmagic",
    # "-C",  # <--- 注释
    # "link-arg=-Tdefmt.x",  # <--- 注释
]


```
