//! Rotozoomer with sound player
//!
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_rt::exception; // for SysTick Handler
use defmt_rtt as _;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;
use embedded_time::rate::*;
use panic_probe as _;
use rp_pico as bsp;

use bsp::hal::{
    clocks::init_clocks_and_plls,
    gpio::{bank0::Gpio25, FunctionSpi, PushPullOutput},
    pac,
    pwm::{FreeRunning, Slices},
    sio::Sio,
    spi::Spi,
    watchdog::Watchdog,
};

mod affine_matrix;
mod uc1701;
mod util;

// static variables for sharing hardware control between main() and exception handlers
// Mutex<RefCell<Option<shared_variable>>> = Mutex::new(RefCell::new(None));
static mut G_LED: Option<bsp::hal::gpio::Pin<Gpio25, PushPullOutput>> = None;
static mut G_PWM_SPEAKER: Option<
    bsp::hal::pwm::Channel<bsp::hal::pwm::Pwm6, bsp::hal::pwm::FreeRunning, bsp::hal::pwm::A>,
> = None;
static mut G_BEAT_COUNT: u32 = 0;

// sampling freq. of a wav file
// It should be a divisor of 1MHz
const AUDIO_FS: u32 = 25_000; // 25kHz

struct AnimationController {
    rot: i32, // angle of rotation (rad)
    scl: i32, // rate of magnification (100 => 1x)
    loopend: bool,
}

impl AnimationController {
    fn new() -> Self {
        Self {
            rot: 0,
            scl: 100,
            loopend: false,
        }
    }
    fn next(&mut self, beat_count: u32) {
        // The animation loops A->B->C->D->E->A->...
        if self.loopend {
            // part E: zoom
            self.rot = 0;
            if self.scl < 100 {
                self.scl += 4;
            } else {
                // transit E->A when rate of magnification is x1
                self.scl = 100;
                if beat_count <= 49 {
                    self.loopend = false;
                }
            }
        } else if beat_count <= 20 {
            // A: still
            self.rot = 0;
            self.scl = 100;
            self.loopend = false;
        } else if beat_count <= 36 {
            // 'I am not an atomic playboy'
            // B: rotate
            self.rot += 1;
        } else if beat_count <= 49 {
            // C: rotate + unzoom
            self.rot += 1;
            self.scl += 2;
        } else if self.scl > 0 {
            // D: rotate + zoom
            self.rot += 1;
            self.scl -= 16;
        } else {
            self.loopend = true;
        }
    }
}

// SysTick handler
#[exception]
fn SysTick() {
    // WAV file (8bit, mono)
    const AUDIO: &[u8] = include_bytes!("../asset/HumanHead25k.wav");
    const AUDIO_BPM: u32 = 130;
    const WAV_HDR_SIZE: usize = 44;
    static mut SAMPLE_POS: usize = WAV_HDR_SIZE;
    unsafe {
        if let Some(ref mut led) = G_LED {
            let phase = (SAMPLE_POS - WAV_HDR_SIZE) as u32 % (AUDIO_FS * 60 / AUDIO_BPM) as u32;
            if phase == 0 {
                G_BEAT_COUNT += 1;
            }
            // heartbeat by LED
            if phase < AUDIO_FS / 20 {
                let _ = led.set_high();
            } else {
                let _ = led.set_low();
            }
        }
        if let Some(ref mut pwm) = G_PWM_SPEAKER {
            // output a sample as duty of PWM (pseudo DAC)
            pwm.set_duty((AUDIO[SAMPLE_POS]) as u16);
            SAMPLE_POS += 1;
            if SAMPLE_POS >= AUDIO.len() {
                SAMPLE_POS = WAV_HDR_SIZE;
                G_BEAT_COUNT = 0;
            }
        }
    }
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let _clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // configure SPI
    let _ = pins.gpio1.into_mode::<FunctionSpi>(); // GPIO1 -> CS
    let _ = pins.gpio2.into_mode::<FunctionSpi>(); // GPIO2 -> SCK
    let _ = pins.gpio3.into_mode::<FunctionSpi>(); // GPIO3 -> MOSI
    let spi = Spi::<_, _, 8>::new(pac.SPI0).init(
        &mut pac.RESETS,
        125_000_000u32.Hz(),
        4_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );
    let lcd_cd_pin = pins.gpio4.into_push_pull_output(); // GPIO4 -> CD
    let mut lcd = uc1701::UC1701::new(spi, lcd_cd_pin);

    // The app controls SysTick by itself, so we can't use cortex_m::delay::Delay.
    // 'MyDelay' is a alternative implimententation of DelayUs function.
    // It generates a delay by Timer instead of SysTick.
    let timer = bsp::hal::timer::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut mydelay = util::MyDelay::new(timer);

    lcd.init(&mut mydelay);

    mydelay.delay_us(1_000_000);

    unsafe {
        G_LED = Some(pins.led.into_push_pull_output());
    }

    // configure GPIO 28 as PWM output (pseudo DAC)
    //
    let pwm_slices = Slices::new(pac.PWM, &mut pac.RESETS);
    // channel A of PWM6 corresponds to GPIO28
    let mut pwm = pwm_slices.pwm6;
    pwm.set_ph_correct();
    pwm.enable();

    // set the PWM into free-run mode.
    // PWM frequency is 125 MHz / 256 = 488 kHz. (beyond audible range)
    // Its duty is variable between 0/256 to 255/256.
    let mut pwm = pwm.into_mode::<FreeRunning>();
    pwm.set_top(255);
    let mut channel_a = pwm.channel_a;
    let _channel_pin_a = channel_a.output_to(pins.gpio28);

    unsafe {
        G_PWM_SPEAKER = Some(channel_a);
    }

    // configure SysTick
    core.SYST.disable_counter();
    core.SYST.clear_current();
    core.SYST.set_reload(1_000_000 / AUDIO_FS - 1);
    core.SYST.enable_interrupt();
    core.SYST.enable_counter();

    const IMAGE: &[u8] = include_bytes!("../asset/rust-logo.pbm");
    let mut imgbmp: [u8; 128 * 128] = [0u8; 128 * 128];
    for i in 0..imgbmp.len() {
        if IMAGE[57 + (i >> 3)] & (128 >> (i & 7)) != 0 {
            imgbmp[i] = 1;
        }
    }

    let mut ani = AnimationController::new();

    let mut time_counter = mydelay.get_counter();
    loop {
        let beat_count;
        unsafe {
            beat_count = G_BEAT_COUNT;
        }
        ani.next(beat_count);

        // calculate Affine transformation matrix
        let mut mat = affine_matrix::AffineMatrix::new();
        mat.translate(64.0, 64.0);
        mat.rotate(ani.rot as f32 * 0.01);
        mat.scale(ani.scl as f32 * 0.01);
        mat.translate(-64.0, -32.0);

        // draw the Rust logo to LCD
        for p in 0..8 {
            let mut d: [u8; 128] = [0; 128];
            for (x, item) in d.iter_mut().enumerate() {
                let mut dat = 0;
                for dy in 0..8 {
                    let y = ((p * 8 + dy) & 0x7f) as usize;
                    let (x1, y1) = mat.transform(x as u32, y as u32);

                    if imgbmp[(y1 * 128 + x1) as usize] != 0 {
                        dat |= 1 << dy;
                    }
                }
                *item = dat;
            }
            lcd.set_page(p);
            lcd.write_data(&d);
        }
        time_counter += 1_000_000 / 60; // 60 fps
        mydelay.wait_counter(time_counter);
    }
}
