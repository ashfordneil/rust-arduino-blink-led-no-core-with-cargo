use avr_core::prelude::v1::*;
use avr_core::intrinsics::volatile_store;

use arduino::*;

pub enum ClockSource {
    None,
    Prescale1,
    Prescale8,
    Prescale64,
    Prescale256,
    Prescale1024,
    ExternalFalling,
    ExternalRising,
}

impl ClockSource {
    pub fn bits(&self) -> u8 {
        use self::ClockSource::*;

        match *self {
            None            =>    0 |    0 |    0,
            Prescale1       =>    0 |    0 | CS00,
            Prescale8       =>    0 | CS01 |    0,
            Prescale64      =>    0 | CS01 | CS00,
            Prescale256     => CS02 |    0 |    0,
            Prescale1024    => CS02 |    0 | CS00,
            ExternalFalling => CS02 | CS01 |    0,
            ExternalRising  => CS02 | CS01 | CS00,
        }
    }

    pub fn mask() -> u8 {
        !(CS02 | CS01 | CS00)
    }
}

pub enum WaveformGenerationMode {
    Normal,
    PwmPhaseCorrect,
    ClearOnTimerMatchOutputCompare,
    FastPwm                       ,
    PwmPhaseCorrectOutputCompare,
    FastPwmOutputCompare,
}

impl WaveformGenerationMode {
    /// Returns bits for TCCR0A, TCCR0B
    pub fn bits(&self) -> (u8, u8) {
        use self::WaveformGenerationMode::*;

        // It makes more sense to return bytes (A,B), but the manual
        // lists the table as (B,A). We match the manual here for
        // inspection purposes and flip the values for sanity
        // purposes.
        let (b, a) = match *self {
            Normal                         => (    0,       0 |     0),
            PwmPhaseCorrect                => (    0,       0 | WGM00),
            ClearOnTimerMatchOutputCompare => (    0,   WGM01 |     0),
            FastPwm                        => (    0,   WGM01 | WGM00),
            // Reserved                    => (WGM02,       0 |     0),
            PwmPhaseCorrectOutputCompare   => (WGM02,       0 | WGM00),
            // Reserved                    => (WGM02,   WGM01 |     0),
            FastPwmOutputCompare           => (WGM02,   WGM01 | WGM00),
        };

        (a, b)
    }

    pub fn mask() -> (u8, u8) {
        (!(WGM00 | WGM01), !(WGM02))
    }
}

pub struct Timer {
    a: u8,
    b: u8,
    output_compare_1: Option<u8>,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            a: 0,
            b: 0,
            output_compare_1: None,
        }
    }

    pub fn clock_source(mut self, source: ClockSource) -> Self {
        self.b &= ClockSource::mask();
        self.b |= source.bits();
        self
    }

    pub fn waveform_generation_mode(mut self, mode: WaveformGenerationMode) -> Self {
        let (a, b) = WaveformGenerationMode::mask();
        self.a &= a;
        self.b &= b;

        let (a, b) = mode.bits();
        self.a |= a;
        self.b |= b;

        self
    }

    pub fn output_compare_1(mut self, value: Option<u8>) -> Self {
        self.output_compare_1 = value;
        self
    }

    pub fn configure(self) {
        unsafe {
            volatile_store(TCCR0A, self.a);
            volatile_store(TCCR0B, self.b);

            // Reset counter to zero
            volatile_store(TCNT0, 0);

            if let Some(v) = self.output_compare_1 {
                // Set the match
                volatile_store(OCR0A, v);

                // Enable compare interrupt
                volatile_store(TIMSK0, OCIE0A);
            }
        }
    }
}
