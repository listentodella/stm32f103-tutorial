#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use hardware::oled;

use defmt::println;
use defmt_rtt as _;
use panic_probe as _;

use cortex_m::peripheral::NVIC;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m_rt::entry;
use stm32f1xx_hal::gpio::{self, gpiob, Edge, ExtiPin, Input, PullUp};
use stm32f1xx_hal::pac::{self, interrupt};
use stm32f1xx_hal::prelude::{
    _stm32_hal_afio_AfioExt, _stm32_hal_flash_FlashExt, _stm32_hal_gpio_GpioExt,
};
use stm32f1xx_hal::rcc::RccExt;
use stm32f1xx_hal::timer::SysTimerExt;

/// 对射式红外传感器
/// 这个属于ISR所有。
/// main（）只能在初始化阶段访问它们，在初始化阶段中断尚未启用（即不能发生并发访问）。
/// 启用中断后，main（）可能不再对这些对象有任何引用。
/// 出于极简主义的考虑，我们在这里不使用RTIC，这将是更好的方式。
static mut INFRARED_SENSOR: MaybeUninit<gpiob::PB14<Input<PullUp>>> = MaybeUninit::uninit();

// 计数器
static mut SENSOR_COUNT: u32 = 0;

#[entry]
fn main() -> ! {
    // 获取对外设的访问对象
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let syst = cp.SYST;
    let mut afio = dp.AFIO.constrain();
    let mut exti = dp.EXTI;
    let mut nvic = cp.NVIC;

    let mut gpiob = dp.GPIOB.split();

    // 冻结系统中所有时钟的配置，并将冻结的频率存储在时钟中
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // 具有自定义精度的阻塞延迟函数
    let mut delay = syst.delay(&clocks);

    // 上电延时
    delay.delay_ms(20u16);

    // 初始化对射式红外传感器
    init_infrared_sensor(gpiob.pb14, &mut gpiob.crh, &mut afio, &mut exti);

    unsafe {
        // Enable EXTI15_10 interruptions
        NVIC::unmask(interrupt::EXTI15_10);
        // 将中断的“优先级”设置为prio
        nvic.set_priority(interrupt::EXTI15_10, 0x80);
    }

    // 初始化 OLED 显示屏
    println!("load oled...");
    let (mut scl, mut sda) = oled::simple::init_oled_pin(gpiob.pb8, gpiob.pb9, &mut gpiob.crh);
    let mut oled = oled::OLED::new(&mut scl, &mut sda);

    oled.show_string(1, 1, "Count:");
    loop {
        oled.show_num(1, 7, get_sensor_count(), 5);
    }
}

/// 中断调用函数
#[interrupt]
fn EXTI15_10() {
    let infrared_sensor = unsafe { &mut *INFRARED_SENSOR.as_mut_ptr() };

    if infrared_sensor.check_interrupt() {
        unsafe {
            SENSOR_COUNT += 1;
        }

        // if we don't clear this bit, the ISR would trigger indefinitely
        infrared_sensor.clear_interrupt_pending_bit();
    }
}

/// 初始化对射式红外传感器
fn init_infrared_sensor(
    pb14: gpio::Pin<'B', 14>,
    crh: &mut gpio::Cr<'B', true>,
    afio: &mut stm32f1xx_hal::afio::Parts,
    exti: &mut pac::EXTI,
) {
    // 配置上拉输入, 无需配置速度
    let mut pin = pb14.into_pull_up_input(crh);
    // 配置 AFIO 外部中断引脚选择
    pin.make_interrupt_source(afio);
    // 从该引脚启用外部中断
    pin.enable_interrupt(exti);
    // 在上升沿生成中断
    pin.trigger_on_edge(exti, Edge::Rising);

    let infrared_sensor = unsafe { &mut *INFRARED_SENSOR.as_mut_ptr() };
    *infrared_sensor = pin;
}

/// 获取传感器计数
fn get_sensor_count() -> u32 {
    unsafe { SENSOR_COUNT }
}
