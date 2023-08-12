//! "Blinky" using delays instead of a timer

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use cortex_m_rt::entry;
use stm32f1xx_hal::gpio::IOPinSpeed;
use stm32f1xx_hal::gpio::OutputSpeed;
use stm32f1xx_hal::pac;
use stm32f1xx_hal::prelude::_fugit_ExtU32;
use stm32f1xx_hal::prelude::_stm32_hal_flash_FlashExt;
use stm32f1xx_hal::prelude::_stm32_hal_gpio_GpioExt;
use stm32f1xx_hal::rcc::RccExt;
use stm32f1xx_hal::timer::SysTimerExt;

use panic_halt as _;

#[entry]
fn main() -> ! {
    // 获取对外设的访问对象
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // 获得原始flash和rcc设备的所有权，并将它们转换为相应的HAL结构
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // 冻结系统中所有时钟的配置，并将冻结的频率存储在时钟中
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // 获取GPIO外围设备
    let mut gpioa = dp.GPIOA.split();

    // 将 PA0 引脚配置为推挽式输出。
    let mut led = gpioa.pa0.into_push_pull_output(&mut gpioa.crl);
    // 设置其输出速度（50 MHz）。
    // 然后在接下来的代码中，我们将使用该引脚来控制 LED 的状态。
    led.set_speed(&mut gpioa.crl, IOPinSpeed::Mhz50);

    // 具有自定义精度的阻塞延迟
    let mut delay = cp.SYST.delay(&clocks);

    // 等待计时器触发更新并更改LED的状态
    loop {
        led.set_high();
        // Use `embedded_hal::DelayMs` trait
        delay.delay_ms(1_000_u16);
        led.set_low();
        // or use `fugit` duration units
        delay.delay(1.secs());
    }
}
