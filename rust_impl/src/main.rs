//! # Pico USB Serial Example
//!
//! Creates a USB Serial device on a Pico board, with the USB driver running in
//! the main thread.
//!
//! This will create a USB Serial device echoing anything it receives. Incoming
//! ASCII characters are converted to upercase, so you can tell it is working
//! and not just local-echo!
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

use hal::pwm::B;
// The macro for our start-up function
use rp_pico::entry;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use rp_pico::hal::pac;

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use rp_pico::hal;

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;




fn write_serial(serial: &mut SerialPort<hal::usb::UsbBus>, buf: &str, block: bool) {
    let write_ptr = buf.as_bytes();

    // Because the buffer is of constant size and initialized to zero (0) we here
    // add a test to determine the size that's really occupied by the str that we
    // wan't to send. From index zero to first byte that is as the zero byte value.
    let mut index = 0;
    while index < write_ptr.len() && write_ptr[index] != 0 {
        index += 1;
    }
    let mut write_ptr = &write_ptr[0..index];

    while !write_ptr.is_empty() {
        match serial.write(write_ptr) {
            Ok(len) => write_ptr = &write_ptr[len..],
            // Meaning the USB write buffer is full
            Err(UsbError::WouldBlock) => {
                if !block {
                    break;
                }
            }
            // On error, just drop unwritten data.
            Err(_) => break,
        }
    }
    // let _ = serial.write("\n".as_bytes());
    let _ = serial.flush();
}

fn process_usb_serial_buf(
    buf: &[u8;64], 
    serial: &mut SerialPort<hal::usb::UsbBus>) {
    
    // figure out length of command
    // let mut index = 0;
    // while index < buf.len() && buf[index] != 0 {
    //     index += 1;
    // }
    // let write_ptr = &buf[0..index];

    match buf[0] {
        // Read uptime command
        b'1' => {
            write_serial(serial, "read uptime request\r\n", false);
        }

        _ => {
            // do nothing if empty
    }
    }
}



/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
///
/// The function configures the RP2040 peripherals, then echoes any characters
/// received over USB Serial.
#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    #[cfg(feature = "rp2040-e5")]
    {
        let sio = hal::Sio::new(pac.SIO);
        let _pins = rp_pico::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
    }

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut said_hello = false;
    loop {
        // A welcome message at the beginning
        // if !said_hello && timer.get_counter() >= 2_000_000 {
        //     said_hello = true;
        //     let _ = serial.write(b"Hello, World!\r\n");
        // }

        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];

            match serial.read(&mut buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                    let _ = serial.flush();
                }
                Ok(count) => {
                    // Process the data we received
                    process_usb_serial_buf(&buf, &mut serial);
                }
            }
        }
    }
}



// End of file
