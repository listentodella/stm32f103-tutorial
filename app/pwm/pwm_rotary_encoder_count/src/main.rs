#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]

mod hardware;
use hardware::oled;
use hardware::peripheral::Peripheral;

use defmt_rtt as _;
use panic_probe as _;

use cortex_m::prelude::_embedded_hal_Qei;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m_rt::entry;
use defmt::println;
use stm32f1xx_hal::pac::TIM3;
use stm32f1xx_hal::qei::Qei;
use stm32f1xx_hal::qei::QeiOptions;
use stm32f1xx_hal::timer::pwm_input;
use stm32f1xx_hal::timer::Tim3NoRemap;
use stm32f1xx_hal::timer::Timer;

#[entry]
fn main() -> ! {
    // 初始化外设
    let Peripheral {
        mut flash,
        rcc,
        tim2: _,
        tim3,
        dbg: _,
        syst,
        mut afio,
        exti: _,
        nvic: _,
        mut gpioa,
        mut gpiob,
    } = Peripheral::new();

    // 冻结系统中所有时钟的配置，并将冻结的频率存储在时钟中
    let clocks = rcc
        .cfgr
        // .use_hse(8.MHz())
        // 设置系统时钟
        // .sysclk(72.MHz())
        // .pclk1(36.MHz())
        // .hclk(72.MHz())
        .freeze(&mut flash.acr);

    // 封装具有自定义精度的阻塞延迟函数
    let mut delay = Peripheral::sys_delay(&mut flash, &clocks, syst);

    // 初始化 OLED 显示屏
    println!("load oled...");
    let (mut scl, mut sda) = oled::init_oled(gpiob.pb8, gpiob.pb9, &mut gpiob.crh);

    // 旋转编码器
    // 配置上拉输入
    println!("load rotary encoder ...");
    let pa6 = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
    let pa7 = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);
    let mut qei = Timer::new(tim3, &clocks).qei((pa6, pa7), &mut afio.mapr, QeiOptions::default());

    oled::show_string(&mut scl, &mut sda, 1, 1, "Cnt:");
    println!("loop ...");
    loop {
        // 获取当前编码器计数
        let tim3_cnt = get_tim3_cnt(&mut qei);
        println!("tim3_cnt={:?}", tim3_cnt as i16);
        oled::show_signed_num(&mut scl, &mut sda, 1, 5, tim3_cnt as i32, 5);
        delay.delay_ms(1000_u16);
    }
}

/// 获取当前编码器计数
fn get_tim3_cnt<PINS>(qei: &mut Qei<TIM3, Tim3NoRemap, PINS>) -> i16
where
    PINS: pwm_input::Pins<Tim3NoRemap>,
{
    let tim3_cnt = qei.count(); // 编码器当前数值
    tim3_cnt as i16
}
