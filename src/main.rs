#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU8, Ordering};
use esp32s3_hal::{clock::ClockControl, pac::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use esp_backtrace as _;

#[allow(clippy::declare_interior_mutable_const)]
const EMPTY_CELL: AtomicU8 = AtomicU8::new(0);

const N: usize = 8;
const MASK: u8 = N as u8 - 1;

pub struct MpMcQueue {
    buffer: [AtomicU8; N],
    dequeue_pos: AtomicU8,
    enqueue_pos: AtomicU8,
}

impl MpMcQueue {
    pub const fn new() -> Self {
        let mut cell_count = 0;
        let mut result_cells = [EMPTY_CELL; N];
        while cell_count != N {
            result_cells[cell_count] = AtomicU8::new(cell_count as u8);
            cell_count += 1;
        }

        Self {
            buffer: result_cells,
            dequeue_pos: AtomicU8::new(0),
            enqueue_pos: AtomicU8::new(0),
        }
    }

    pub fn dequeue(&self) -> bool {
        let mut pos = self.dequeue_pos.load(Ordering::Relaxed);
        let mut cell;
        loop {
            cell = &self.buffer[(pos & MASK) as usize];
            let seq = cell.load(Ordering::Acquire);
            match (seq as i8).wrapping_sub((pos.wrapping_add(1)) as i8) {
                0 => {
                    if self
                        .dequeue_pos
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
                }
                i8::MIN..=-1 => return false,
                _ => {
                    pos = self.dequeue_pos.load(Ordering::Relaxed);
                }
            }
        }
        cell.store(pos.wrapping_add(MASK).wrapping_add(1), Ordering::Release);
        true
    }

    pub fn enqueue(&self) -> bool {
        let mut pos = self.enqueue_pos.load(Ordering::Relaxed);
        let mut cell;
        loop {
            cell = &self.buffer[(pos & MASK) as usize];
            let seq = cell.load(Ordering::Acquire);
            match (seq as i8).wrapping_sub(pos as i8) {
                0 => {
                    if self
                        .enqueue_pos
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
                }
                i8::MIN..=-1 => return false,
                _ => {
                    pos = self.enqueue_pos.load(Ordering::Relaxed);
                }
            }
        }
        cell.store(pos.wrapping_add(1), Ordering::Release);
        true
    }
}

#[inline(never)]
fn inner() {
    let queue = MpMcQueue::new();
    loop {
        if !queue.enqueue() || !queue.dequeue() {
            break;
        }
    }
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

    inner();
    panic!("miscompilation")
}
