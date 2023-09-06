#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]

use hardware::oled;

use defmt::println;
use defmt_rtt as _;
use panic_probe as _;

use cortex_m::prelude::_embedded_hal_Qei;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m_rt::entry;
use stm32f1xx_hal::gpio;
use stm32f1xx_hal::gpio::OutputSpeed;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::pac::TIM3;
use stm32f1xx_hal::prelude::_stm32_hal_afio_AfioExt;
use stm32f1xx_hal::prelude::_stm32_hal_flash_FlashExt;
use stm32f1xx_hal::prelude::_stm32_hal_gpio_GpioExt;
use stm32f1xx_hal::qei::Qei;
use stm32f1xx_hal::qei::QeiOptions;
use stm32f1xx_hal::rcc::RccExt;
use stm32f1xx_hal::timer::pwm_input;
use stm32f1xx_hal::timer::SysDelay;
use stm32f1xx_hal::timer::SysTimerExt;
use stm32f1xx_hal::timer::Tim3NoRemap;
use stm32f1xx_hal::timer::Timer;

#[entry]
fn main() -> ! {
    // 获取对外设的访问对象
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let syst = cp.SYST;
    let mut afio = dp.AFIO.constrain();
    let tim3 = dp.TIM3;

    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();

    // 冻结系统中所有时钟的配置，并将冻结的频率存储在时钟中
    let clocks = rcc
        .cfgr
        // .use_hse(8.MHz())
        // 设置系统时钟
        // .sysclk(72.MHz())
        // .pclk1(36.MHz())
        // .hclk(72.MHz())
        .freeze(&mut flash.acr);

    // 具有自定义精度的阻塞延迟函数
    let mut delay = syst.delay(&clocks);

    // 初始化 OLED 显示屏
    println!("load oled...");
    let (mut scl, mut sda) = init_oled(gpiob.pb8, gpiob.pb9, &mut gpiob.crh);

    // 旋转编码器
    // 配置上拉输入
    println!("load rotary encoder ...");
    let pa6 = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
    let pa7 = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);
    let mut qei = Timer::new(tim3, &clocks).qei((pa6, pa7), &mut afio.mapr, QeiOptions::default());

    oled::show_string(&mut scl, &mut sda, 1, 1, "Cnt:");
    oled::show_string(&mut scl, &mut sda, 2, 1, "Speed:");
    println!("loop ...");
    loop {
        // 获取当前编码器计数
        let tim3_cnt = get_tim3_cnt(&mut qei);
        let tim3_speed = get_tim3_speed(&mut qei, &mut delay);
        println!("cnt={:?} speed={:?}", tim3_cnt, tim3_speed);
        oled::show_signed_num(&mut scl, &mut sda, 1, 5, tim3_cnt as i32, 5);
        oled::show_signed_num(&mut scl, &mut sda, 2, 7, tim3_speed as i32, 5);
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

/// 获取当前编码器计数
fn get_tim3_speed<PINS>(qei: &mut Qei<TIM3, Tim3NoRemap, PINS>, delay: &mut SysDelay) -> i16
where
    PINS: pwm_input::Pins<Tim3NoRemap>,
{
    let before = qei.count();
    // 模拟闸门时间
    // 不建议在主循环中加入过长的延时，会阻塞主函数的执行
    delay.delay_ms(1_000_u16);
    let after = qei.count();

    after.wrapping_sub(before) as i16
}

/// 初始化 OLED 显示屏
pub fn init_oled(
    pb8: gpio::Pin<'B', 8>,
    pb9: gpio::Pin<'B', 9>,
    crh: &mut gpio::Cr<'B', true>,
) -> (
    gpio::PB8<gpio::Output<gpio::OpenDrain>>,
    gpio::PB9<gpio::Output<gpio::OpenDrain>>,
) {
    // 将引脚配置为作为开漏输出模式
    let mut scl = pb8.into_open_drain_output(crh);
    let mut sda = pb9.into_open_drain_output(crh);
    scl.set_speed(crh, gpio::IOPinSpeed::Mhz50);
    sda.set_speed(crh, gpio::IOPinSpeed::Mhz50);

    // 始化 OLED 配置
    hardware::oled::init_oled_config(&mut scl, &mut sda);
    (scl, sda)
}
