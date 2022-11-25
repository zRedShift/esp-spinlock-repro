#![no_std]
#![no_main]

use core::ops::IndexMut;
use core::sync::atomic::{AtomicU8, Ordering};
use esp32s3_hal::{clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use esp_backtrace as _;
#[cfg(feature = "esp-println")]
use esp_println::println;

#[cfg(not(feature = "esp-println"))]
macro_rules! println {
    ($($arg:tt)*) => {};
}

#[xtensa_lx_rt::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    const N: usize = 8;
    const MASK: u8 = N as u8 - 1;
    let mut cells: [_; N] = core::array::from_fn(|i| AtomicU8::new(i as u8));
    let dequeue_pos = AtomicU8::new(0);
    let enqueue_pos = AtomicU8::new(0);
    loop {
        let mut pos = enqueue_pos.load(Ordering::Relaxed);
        println!("enqueue pos at {pos}");
        let mut cell;
        loop {
            cell = cells.index_mut((pos & MASK) as usize);
            let seq = cell.load(Ordering::Acquire);
            println!("enqueue seq at {seq}");
            let dif = (seq as i8).wrapping_sub(pos as i8);
            println!("enqueue dif at {seq}");
            assert!(dif >= 0);
            if dif == 0 {
                if enqueue_pos
                    .compare_exchange_weak(
                        pos,
                        pos.wrapping_add(1),
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    break;
                }
            } else {
                pos = enqueue_pos.load(Ordering::Relaxed);
                println!("enqueue pos at {pos}");
            }
        }
        let new_seq = pos.wrapping_add(1);
        println!("enqueue new seq at {new_seq}");
        cell.store(new_seq, Ordering::Release);
        pos = dequeue_pos.load(Ordering::Relaxed);
        println!("dequeue pos at {pos}");
        loop {
            cell = cells.index_mut((pos & MASK) as usize);
            let seq = cell.load(Ordering::Acquire);
            println!("dequeue seq at {seq}");
            let dif = (seq as i8).wrapping_sub((pos.wrapping_add(1)) as i8);
            println!("dequeue dif at {seq}");
            assert!(dif >= 0);
            if dif == 0 {
                if dequeue_pos
                    .compare_exchange_weak(
                        pos,
                        pos.wrapping_add(1),
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    break;
                }
            } else {
                pos = dequeue_pos.load(Ordering::Relaxed);
                println!("dequeue pos at {pos}");
            }
        }
        let new_seq = pos.wrapping_add(MASK).wrapping_add(1);
        println!("dequeue new seq at {new_seq}");
        cell.store(new_seq, Ordering::Release);
    }
}
