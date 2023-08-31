#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]

mod hardware;
use hardware::oled;
use hardware::peripheral::Peripheral;

use defmt_rtt as _;
use panic_probe as _;

use cortex_m::prelude::_embedded_hal_adc_OneShot;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m_rt::entry;
use defmt::println;
use stm32f1xx_hal::adc;
use stm32f1xx_hal::adc::Adc;
use stm32f1xx_hal::adc::SampleTime;
use stm32f1xx_hal::dma::Event;
use stm32f1xx_hal::flash;
use stm32f1xx_hal::gpio::gpioa;
use stm32f1xx_hal::gpio::gpiob;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::pac::adc1;
use stm32f1xx_hal::prelude::_fugit_RateExtU32;
use stm32f1xx_hal::prelude::_stm32_hal_adc_ChannelTimeSequence;
use stm32f1xx_hal::prelude::_stm32_hal_dma_DmaExt;
use stm32f1xx_hal::prelude::_stm32_hal_flash_FlashExt;
use stm32f1xx_hal::prelude::_stm32_hal_gpio_GpioExt;
use stm32f1xx_hal::rcc;
use stm32f1xx_hal::rcc::RccExt;

#[entry]
fn main() -> ! {
    // 初始化外设
    // 获取对外设的访问对象
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash: flash::Parts = dp.FLASH.constrain();
    let rcc: rcc::Rcc = dp.RCC.constrain();
    let syst = cp.SYST;
    let adc1 = dp.ADC1;
    let mut dma_ch1 = dp.DMA1.split().1;

    let mut gpioa: gpioa::Parts = dp.GPIOA.split();
    let mut gpiob: gpiob::Parts = dp.GPIOB.split();

    // 配置ADC时钟默认值是最慢的ADC时钟：PCLK2/8。同时ADC时钟可配置。
    // 因此，它的频率可能会被调整以满足某些实际需求。
    // 使用支持的预分频器值2/4/6/8来近似用户指定的值。
    let clocks = rcc.cfgr.adcclk(72.MHz()).freeze(&mut flash.acr);

    // 封装具有自定义精度的阻塞延迟函数
    let mut delay = Peripheral::sys_delay(&mut flash, &clocks, syst);

    // 初始化 OLED 显示屏
    println!("load oled...");
    let (mut scl, mut sda) = oled::init_oled(gpiob.pb8, gpiob.pb9, &mut gpiob.crh);

    let mut data_a = [0x01, 0x02, 0x03, 0x04];
    let data_b = [0, 0, 0, 0];

    oled::show_hex_num(&mut scl, &mut sda, 1, 1, data_a[0], 2);
    oled::show_hex_num(&mut scl, &mut sda, 1, 4, data_a[1], 2);
    oled::show_hex_num(&mut scl, &mut sda, 1, 7, data_a[2], 2);
    oled::show_hex_num(&mut scl, &mut sda, 1, 10, data_a[3], 2);
    oled::show_hex_num(&mut scl, &mut sda, 2, 1, data_b[0], 2);
    oled::show_hex_num(&mut scl, &mut sda, 2, 4, data_b[1], 2);
    oled::show_hex_num(&mut scl, &mut sda, 2, 7, data_b[2], 2);
    oled::show_hex_num(&mut scl, &mut sda, 2, 10, data_b[3], 2);

    // 关联的外围设备地址
    // inc指示地址是否在每次字节传输后递增
    dma_ch1.set_peripheral_address(data_a.as_ptr() as u32, true);
    // address where from/to data will be read/write
    // inc指示地址是否在每次字节传输后递增
    // 数组间的转运需要自增
    dma_ch1.set_memory_address(data_b.as_ptr() as u32, true);
    // 要传输的字节数
    dma_ch1.set_transfer_length(data_a.len());

    // 数据传输方向
    // dma_ch1.ch().cr.write(|w| w.dir().set_bit());

    dma_ch1.ch().cr.modify(|_, w| {
        w.mem2mem()
            .clear_bit()
            .pl()
            .medium()
            .msize()
            .bits16()
            .psize()
            .bits16()
            .circ()
            .set_bit()
            .dir()
            .clear_bit()
    });

    // 启动DMA传输
    dma_ch1.start();

    // dma_ch1.listen(Event::TransferComplete);

    oled::show_hex_num(&mut scl, &mut sda, 3, 1, data_a[0], 2);
    oled::show_hex_num(&mut scl, &mut sda, 3, 4, data_a[1], 2);
    oled::show_hex_num(&mut scl, &mut sda, 3, 7, data_a[2], 2);
    oled::show_hex_num(&mut scl, &mut sda, 3, 10, data_a[3], 2);
    oled::show_hex_num(&mut scl, &mut sda, 4, 1, data_b[0], 2);
    oled::show_hex_num(&mut scl, &mut sda, 4, 4, data_b[1], 2);
    oled::show_hex_num(&mut scl, &mut sda, 4, 7, data_b[2], 2);
    oled::show_hex_num(&mut scl, &mut sda, 4, 10, data_b[3], 2);
    loop {
        delay.delay_ms(1000_u32);
        data_a = [0x01, 0x02, 0x03, 0x06];

        // 关联的外围设备地址
        // inc指示地址是否在每次字节传输后递增
        dma_ch1.set_peripheral_address(data_a.as_ptr() as u32, true);
        // address where from/to data will be read/write
        // inc指示地址是否在每次字节传输后递增
        // 数组间的转运需要自增
        dma_ch1.set_memory_address(data_b.as_ptr() as u32, true);
        // 要传输的字节数
        dma_ch1.set_transfer_length(data_a.len());

        // 数据传输方向
        // dma_ch1.ch().cr.write(|w| w.dir().set_bit());

        dma_ch1.ch().cr.modify(|_, w| {
            w.mem2mem()
                .clear_bit()
                .pl()
                .medium()
                .msize()
                .bits16()
                .psize()
                .bits16()
                .circ()
                .set_bit()
                .dir()
                .clear_bit()
        });

        // 启动DMA传输
        dma_ch1.start();
        println!("{:?}", data_b);
    }
}
