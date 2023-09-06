#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]

use core::cell::RefCell;

use hardware::oled;

use defmt::println;
use defmt_rtt as _;
use panic_probe as _;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use stm32f1xx_hal::device::TIM2;
use stm32f1xx_hal::gpio::{self, OutputSpeed};
use stm32f1xx_hal::pac::{self, interrupt};
use stm32f1xx_hal::prelude::{_stm32_hal_flash_FlashExt, _stm32_hal_gpio_GpioExt};
use stm32f1xx_hal::rcc::RccExt;
use stm32f1xx_hal::timer::CounterHz;
use stm32f1xx_hal::timer::{Event, TimerExt};

static G_TIM: Mutex<RefCell<Option<CounterHz<TIM2>>>> = Mutex::new(RefCell::new(None));

// 计数器
static mut NUM: u32 = 0;
static mut ARR: u32 = 0;

#[entry]
fn main() -> ! {
    // 获取对外设的访问对象
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut nvic = cp.NVIC;
    let tim2 = dp.TIM2;

    let mut gpiob = dp.GPIOB.split();

    // 冻结系统中所有时钟的配置，并将冻结的频率存储在时钟中
    let clocks = rcc
        .cfgr
        // .use_hse(8.MHz())
        // 设置系统时钟
        // .sysclk(72.MHz())
        // .pclk1(36.MHz())
        .freeze(&mut flash.acr);

    unsafe {
        // Enable interruptions
        NVIC::unmask(interrupt::TIM2);
        // 设置中断的优先级
        nvic.set_priority(interrupt::TIM2, 2);
    }

    // 配置系统定时器，每秒触发一次更新，并启用中断
    let mut timer: CounterHz<TIM2> = tim2.counter_hz(&clocks);

    // 设置一个1秒后过期的计时器
    // 定时时间计算公式：CK_CNT_OV = CK_CNT/(ARR+1) = CK_PSC/(PSC+1)/(ARR+1)
    // 定时 1s 即定时频率为 1Hz；
    // timer.start(1.Hz()).unwrap();

    // 使用用户定义的预分频器和自动重新加载寄存器以倒计时模式重新启动计时器
    timer.start_raw(100, 2500);
    // timer.start_raw(100, 5000);

    // 当计时器到期时产生中断
    timer.listen(Event::Update);

    // 将计时器移动到全局存储中
    cortex_m::interrupt::free(|cs| G_TIM.borrow(cs).replace(Some(timer)));

    // 初始化 OLED 显示屏
    println!("load oled...");
    let (mut scl, mut sda) = init_oled(gpiob.pb8, gpiob.pb9, &mut gpiob.crh);

    oled::show_string(&mut scl, &mut sda, 1, 1, "Num:");
    oled::show_string(&mut scl, &mut sda, 2, 1, "Arr:");
    oled::show_string(&mut scl, &mut sda, 3, 1, "Cnt:");
    loop {
        oled::show_num(&mut scl, &mut sda, 1, 5, get_num(), 8);
        oled::show_num(&mut scl, &mut sda, 2, 5, get_arr(), 8);
        oled::show_num(&mut scl, &mut sda, 3, 5, get_counter(), 8);
    }
}

/// 中断调用函数
#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(tim) = G_TIM.borrow(cs).borrow_mut().as_mut() {
            unsafe {
                NUM += 1;
                ARR = tim.arr() as u32;
            }
            tim.wait().unwrap();
        }
    });
}

/// 获取计数
fn get_num() -> u32 {
    unsafe { NUM }
}

/// 获取自动重新加载寄存器
fn get_arr() -> u32 {
    unsafe { ARR }
}

/// 获取计数
fn get_counter() -> u32 {
    let mut count = 0;
    cortex_m::interrupt::free(|cs| {
        if let Some(tim) = G_TIM.borrow(cs).borrow_mut().as_mut() {
            count = tim.now().ticks();
        }
    });
    count
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
