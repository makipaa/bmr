#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;
use riscv;
use longan_nano::hal::{timer::{Timer, Event},prelude::*};
use longan_nano::hal::pac::{ ECLIC, Interrupt, TIMER0};
use longan_nano::hal::eclic::{EclicExt, Level, LevelPriorityBits, Priority, TriggerType};
use longan_nano::led::{Led, RED};


static mut TIMER_TIM0: Option<Timer<TIMER0>> = None;
static mut LED: Option<RED> = None;
#[entry]
fn main() -> !{
    
    let dp = longan_nano::hal::pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();


    // Configure GPIOs
    let gpioc = dp.GPIOC.split(&mut rcu);
    let red : RED = RED::new(gpioc.pc13);

    unsafe { TIMER_TIM0 = Some(Timer::timer0(dp.TIMER0, 1.hz(), &mut rcu));}

    unsafe {
        LED = Some(red)
     }
    

    ECLIC::reset();
    ECLIC::set_threshold_level(Level::L0);
    ECLIC::set_level_priority_bits(LevelPriorityBits::L3P1);

    ECLIC::setup(
        Interrupt::TIMER0_UP,
        TriggerType::Level,
        Level::L0,
        Priority::P0,
    );

    unsafe { ECLIC::unmask(Interrupt::TIMER0_UP) };
    unsafe {
        riscv::interrupt::enable();
        TIMER_TIM0.as_mut().unwrap().listen(Event::Update);
    }

    unsafe{ riscv::asm::wfi()};

    loop{}
}

#[allow(non_snake_case)]
#[no_mangle]
fn TIMER0_UP () {
    unsafe { riscv::interrupt::disable()};
    
    unsafe {
        if LED.as_mut().unwrap().is_on() {
            LED.as_mut().unwrap().off();
        } else {
            LED.as_mut().unwrap().on();
        } 
    }
    
    unsafe{TIMER_TIM0.as_mut().unwrap().clear_update_interrupt_flag()};
    unsafe { riscv::interrupt::enable()};
}