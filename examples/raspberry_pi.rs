#![no_std]
#![no_main]
// extern crate panic_halt;
extern crate embedded_hal;
extern crate rp_pico;
use rp_pico::entry;
use panic_halt as _;
// extern crate cortex_m;

// use hal::pac;

// Some traits we need
// use embedded_hal::digital::OutputPin;
// use hal::clocks::Clock;

// #[link_section = ".boot2"]
// #[used]
// pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
// if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

// use cortex_m::delay::Delay;
// use embedded_hal::delay::DelayNs;
// use embedded_hal::spi::Operation::DelayNs as SpiDelayNs;
use {
    display_interface_spi::SpiInterface,
    embedded_graphics::{
        text::{
            Text,
            // DecorationColor,
        },
        mono_font::{
            MonoTextStyle,
            ascii,
        },
        pixelcolor::Gray4,
        prelude::*,
    },
//     hal::gpio::Gpio,
//     hal::spi::{Bus, Mode, SlaveSelect, Spi},
    ssd1327,
};
// use embedded_hal::spi::Mode;
use rp_pico::hal as hal;
use rp_pico::hal::pac;
// use embedded_hal::spi::SpiDevice;

use hal::gpio::FunctionSpi;
use hal::Spi;
use fugit::RateExtU32;

use embedded_hal_bus::spi::ExclusiveDevice;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );










    // Configure gpio
    // let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0).unwrap();
    // let gpio = Gpio::new().unwrap();
    // let cs = gpio.get(8).unwrap().into_output();
    let dc = pins.gpio5.into_push_pull_output();
    let mut rst = pins.gpio6.into_push_pull_output();

    // Init SPI
    // let cs = pins.gpio1.into_function::<FunctionSpi>();
    let cs = pins.gpio1.into_push_pull_output();

    let sclk = pins.gpio2.into_function::<FunctionSpi>();
    let mosi = pins.gpio3.into_function::<FunctionSpi>();

    // let spiBus = SpiBus::new(pac.SPI0);
    let spi = Spi::</*hal::spi::Disabled*/ _, _, _, 8>::new(
        pac.SPI0, (mosi, sclk))
        .init(&mut pac.RESETS, 125u32.MHz(), 16u32.MHz(), embedded_hal::spi::MODE_0);
    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let spii = SpiInterface::new(spi_dev, dc);
    let mut disp = ssd1327::display::Ssd1327::new(spii);

    let mut timer = hal::timer::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    // Reset & init
    disp.reset(&mut rst, &mut timer).unwrap();
    disp.init().unwrap();

    // Clear the display
    disp.clear(Gray4::new(0)).unwrap();
    disp.flush().unwrap();

    // Write "Hello" to the display
    Text::new("Hello", Point::new(0, 0), MonoTextStyle::new(&ascii::FONT_10X20, Gray4::new(0x0f))
            // background_color : Some(Gray4::new(0x00)),
            // strikethrough_color: DecorationColor::Custom(Gray4::new(0x80)),
            // underline_color: DecorationColor::Custom(Gray4::new(0xa0)),
    )
        .draw(&mut disp)
        .unwrap();
            // )
    disp.flush().unwrap();
    panic!()
}
