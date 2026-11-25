//! Shout at the board and the light gets brighter — with hardware PWM.
//!
//! This is the demo Seeed flashes into every XIAO MG24 **Sense** at the factory: *"the louder you
//! shout, the brighter the light will be."* In Arduino it is `analogWrite(LED_PIN, 0..255)`; here
//! it is a TIMER0 compare channel whose duty cycle follows the microphone level.
//!
//! Why PWM and not a software loop: a GPIO has two brightnesses, and a loop that toggles it
//! spends the CPU doing it — the same CPU that has to sample the microphone. A compare channel
//! keeps the LED lit *while the sampler runs*, which is the difference between a demo and
//! something a user can build on.
//!
//! What the example uses, and why each part is there:
//!
//! - **PC8 enables the microphone**, PC9 is the analog signal, and `IADC0` converts it. The mic
//!   is analog MEMS, not PDM: there is no I2S and no digital microphone bus on this board.
//! - **PA7 is the user LED, active low**, and it is the same pin the blink examples use.
//! - **the level is an RMS window of 128 samples**, mapped to 0..=100 % duty. Silence is a small
//!   number, not zero samples; zero samples means the converter never delivered.
//!
//! Verification without eyes: the mailbox reports RMS, peak and the duty being applied. The
//! brightness itself needs a human — the numbers do not.
//!
//! VERIFIED ON HARDWARE (2026-11-24): the record is in this file's header and in the crate README's dated table.

#![no_std]
#![no_main]

use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::entry;
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::prelude::*;

use panic_probe as _;

#[path = "common/mailbox.rs"]
mod mailbox;

use defmt_rtt as _;

/// PWM frequency for the LED. 1 kHz is fast enough that the eye integrates the pulses into a
/// brightness instead of seeing a flicker. The timer's 16-bit TOP sets the *floor* at
/// 19 MHz / 65535 ≈ 290 Hz, not the ceiling, so this is comfortably inside the range.

/// Samples per level window.
const WINDOW: usize = 128;

/// Map an RMS level (ADC counts around mid-scale) to a duty cycle.
///
/// The scale is deliberately not linear in the ear: a quiet room reads about 10 counts and a
/// raised voice a few hundred, so a linear map would leave the LED dark until the room is loud.
/// 0..=400 counts across 0..=100 % puts normal speech in the middle, which is what makes the
/// demo feel responsive.
fn push_dec(out: &mut [u8], mut at: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    loop {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
        if value == 0 || n == digits.len() {
            break;
        }
    }
    for i in (0..n).rev() {
        out[at] = digits[i];
        at += 1;
    }
    at
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = Board::take().expect("peripherals taken twice");
    let mut delay = board.delay();

    // The microphone: PC8 enable (active high), then the converter.
    board.output_on(Port::C, 8, false).on();
    delay.delay_ms(5);
    let converter = board.converter().expect("the converter is handed out once");
    let mic = match converter.microphone() {
        Ok(mic) => mic,
        Err(_) => {
            mailbox::write(mailbox::Status::Fail, b"mic: converter will not start");
            loop {
                delay.delay_ms(500);
            }
        }
    };

    // ## Aqui não há PWM no LED, de propósito: TIMER0 é o pace do microfone
    //
    // Este exemplo tinha os dois -- o LED num canal PWM do TIMER0 e o microfone a usar o mesmo TIMER0 como pace do
    // IADC -- e o segundo a configurar o timer desfazia o primeiro. Medido 2026-11-21 na placa: a verificação do
    // PWM dizia `TIMER0 is not counting (clock?)`, e dizia a verdade: o timer já não estava no modo que o PWM
    // deixou. O exemplo pedia um periférico que a HAL só pode dar uma vez, e o aviso certo era o que aparecia.
    //
    // O microfone é o assunto aqui; o LED era decoração sobre o mesmo recurso. Para ver o PWM do LED, há
    // `hal_timer0_pwm`, onde ele é o assunto e nada mais o disputa.

    // ## Dez leituras, uma por segundo
    //
    // Uma leitura so' nao distingue "o IADC esta' a converter som" de "le sempre o mesmo registo": um valor fixo
    // passa por boa medida. Com dez valores separados no tempo, um sinal vivo varia e um registo congelado nao --
    // e' a mesma diferenca que os dez acordares do EM2 fizeram, entre "o contador anda" e "o contador esteve parado".
    for leitura in 0..10u32 {
        let level = mic.level(WINDOW);

        {
            let mut buf = [0u8; 64];
            buf[..4].copy_from_slice(b"mic ");
            let mut at = 4;
            at = push_dec(&mut buf, at, leitura);
            buf[at] = b' ';
            at += 1;
            buf[at..at + b"rms=".len()].copy_from_slice(b"rms=");
            at += 4;
            at = push_dec(&mut buf, at, level.rms);
            buf[at..at + b" pk=".len()].copy_from_slice(b" pk=");
            at += 4;
            at = push_dec(&mut buf, at, level.peak);
            buf[at..at + b" n=".len()].copy_from_slice(b" n=");
            at += 3;
            at = push_dec(&mut buf, at, level.samples);
            mailbox::write(
                if level.samples > 0 {
                    mailbox::Status::Pass
                } else {
                    mailbox::Status::Fail
                },
                &buf[..at],
            );
        }

        // Uma pausa entre janelas: o que se quer aqui e' uma leitura por segundo, nao sessenta. Uma leitura
        // por segundo tambem deixa o valor mudar entre linhas, que e' o que prova que ha' um sinal e nao um
        // registo congelado.
        delay.delay_ms(1000);
    }

    mailbox::write(mailbox::Status::Pass, b"mic: dez leituras de um segundo");

    loop {
        delay.delay_ms(1000);
    }
}
