# BKP 断电恢复

这是一个使用备用电池供电，主电源断电后 BKP 恢复数据的示例。

演示主电源掉电后再上电显示的依旧是掉电前最后一次写入的数据。

## 执行指令

```shell
cargo rp rtc_bkp
```

## 学习目标

- 了解 BKP

## 接线图

![](../../../images/wiring_diagram/11-2%20硬件SPI读写W25Q64.jpg)
