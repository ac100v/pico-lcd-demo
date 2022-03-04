use panic_probe as _;
use rp_pico as bsp;

pub struct MyDelay {
    tmr: bsp::hal::Timer,
}

impl MyDelay {
    pub fn new(tmr: bsp::hal::Timer) -> Self {
        Self { tmr }
    }
}

impl embedded_hal::blocking::delay::DelayUs<u32> for MyDelay {
    fn delay_us(&mut self, us: u32) {
        let t0 = self.tmr.get_counter();
        loop {
            let t1 = self.tmr.get_counter();
            if (t1 - t0) >= us.into() {
                break;
            }
        }
    }
}
