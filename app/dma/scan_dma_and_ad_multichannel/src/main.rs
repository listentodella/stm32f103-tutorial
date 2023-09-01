#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]

use cortex_m::asm::wfi;
use cortex_m::singleton;
use defmt_rtt as _;
use panic_probe as _;

mod hardware;
use hardware::oled;
use hardware::peripheral::Peripheral;

use cortex_m_rt::entry;
use defmt::println;
use stm32f1xx_hal::adc;
use stm32f1xx_hal::adc::Adc;
use stm32f1xx_hal::adc::SetChannels;
use stm32f1xx_hal::flash;
use stm32f1xx_hal::gpio::gpioa;
use stm32f1xx_hal::gpio::gpiob;
use stm32f1xx_hal::gpio::Analog;
use stm32f1xx_hal::gpio::PA0;
use stm32f1xx_hal::gpio::PA1;
use stm32f1xx_hal::gpio::PA2;
use stm32f1xx_hal::gpio::PA3;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::pac::adc1;
use stm32f1xx_hal::pac::ADC1;
use stm32f1xx_hal::prelude::_fugit_RateExtU32;
use stm32f1xx_hal::prelude::_stm32_hal_adc_ChannelTimeSequence;
use stm32f1xx_hal::prelude::_stm32_hal_dma_DmaExt;
use stm32f1xx_hal::prelude::_stm32_hal_dma_ReadDma;
use stm32f1xx_hal::prelude::_stm32_hal_flash_FlashExt;
use stm32f1xx_hal::prelude::_stm32_hal_gpio_GpioExt;
use stm32f1xx_hal::rcc;
use stm32f1xx_hal::rcc::RccExt;

pub struct AdcPins(PA0<Analog>, PA1<Analog>, PA2<Analog>, PA3<Analog>);

impl SetChannels<AdcPins> for Adc<ADC1> {
    fn set_samples(&mut self) {
        // 为特定通道设置ADC采样时间
        self.set_channel_sample_time(0, adc::SampleTime::T_55);
        self.set_channel_sample_time(1, adc::SampleTime::T_55);
        self.set_channel_sample_time(2, adc::SampleTime::T_55);
        self.set_channel_sample_time(3, adc::SampleTime::T_55);
    }

    fn set_sequence(&mut self) {
        // ADC设置常规通道转换序列
        // 定义要转换为常规组的通道序列。
        self.set_regular_sequence(&[0, 1, 2, 3]);
        // 我们可以选择设置连续扫描模式
        self.set_continuous_mode(true);
        // 我们还可以使用不连续转换（每次转换4个通道）
        // self.set_discontinuous_mode(Some(4));
    }
}

#[entry]
fn main() -> ! {
    // 初始化外设
    // 获取对外设的访问对象
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash: flash::Parts = dp.FLASH.constrain();
    let rcc: rcc::Rcc = dp.RCC.constrain();
    let syst = cp.SYST;
    let dma_ch1 = dp.DMA1.split().1;

    let mut gpioa: gpioa::Parts = dp.GPIOA.split();
    let mut gpiob: gpiob::Parts = dp.GPIOB.split();

    // 配置ADC时钟默认值是最慢的ADC时钟：PCLK2/8。同时ADC时钟可配置。
    // 因此，它的频率可能会被调整以满足某些实际需求。
    // 使用支持的预分频器值2/4/6/8来近似用户指定的值。
    let clocks = rcc.cfgr.adcclk(72.MHz()).freeze(&mut flash.acr);

    // 封装具有自定义精度的阻塞延迟函数
    let mut _delay = Peripheral::sys_delay(&mut flash, &clocks, syst);

    // 初始化 OLED 显示屏
    println!("load oled...");
    let (mut scl, mut sda) = oled::init_oled(gpiob.pb8, gpiob.pb9, &mut gpiob.crh);

    // 初始化 ADC
    let adc1 = dp.ADC1;
    // Setup ADC
    let mut adc1 = Adc::adc1(adc1, clocks);
    // 设置Adc结果对齐
    adc1.set_align(adc::Align::Right);
    // 外部触发源，软件触发
    adc1.set_external_trigger(adc1::cr2::EXTSEL_A::Swstart);
    // 设置ADC单次转换
    adc1.set_continuous_mode(true);

    // Configure analog input
    let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);
    let adc_ch1 = gpioa.pa1.into_analog(&mut gpioa.crl);
    let adc_ch2 = gpioa.pa2.into_analog(&mut gpioa.crl);
    let adc_ch3 = gpioa.pa3.into_analog(&mut gpioa.crl);
    // 使用Channels类型来包装pins参数
    let pins = AdcPins(adc_ch0, adc_ch1, adc_ch2, adc_ch3);
    let adc_dma = adc1.with_scan_dma(pins, dma_ch1);

    // 指定引脚
    // let adc_dma = adc1.with_dma(adc_ch0, dma_ch1);

    let ad_value = singleton!(: [u16; 8] = [0; 8]).unwrap();

    // read方法消耗buf和self，启动adc和dma传输并返回RxDma结构。
    // wait方法使用RxDma结构，等待整个传输完成，然后返回更新的buf和底层adc_dma结构。
    // 对于非阻塞，可以调用RxDma的is_done方法，并且只在该方法返回true后调用wait。
    let (buf, _adc_dma) = adc_dma.read(ad_value).wait();
    // 消耗AdcDma结构体，将adc配置恢复到以前的状态，并在正常模式下返回adc结构体。
    // let (_adc1, _adc_ch0, _dma_ch1) = adc_dma.split();

    oled::show_string(&mut scl, &mut sda, 1, 1, "AD0:");
    oled::show_string(&mut scl, &mut sda, 2, 1, "AD1:");
    oled::show_string(&mut scl, &mut sda, 3, 1, "AD2:");
    oled::show_string(&mut scl, &mut sda, 4, 1, "AD3:");

    oled::show_num(&mut scl, &mut sda, 1, 5, buf[1].into(), 4);
    oled::show_num(&mut scl, &mut sda, 2, 5, buf[2].into(), 4);
    oled::show_num(&mut scl, &mut sda, 3, 5, buf[3].into(), 4);
    oled::show_num(&mut scl, &mut sda, 4, 5, buf[4].into(), 4);

    println!("loop");
    loop {
        wfi();
    }
}
