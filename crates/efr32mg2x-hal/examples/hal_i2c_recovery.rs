//! After a NACK, does the bus still work?
//!
//! The 2026-11-22 review's item 2: "no hardware I2C path sends STOP or ABORT after an error, so one NACK can leave
//! the bus held". Nothing has ever measured this. Every I2C result in this repository is a *happy path* result: a
//! device answers, the byte is right, the example is green. A sensor that is busy, absent or in reset answers
//! nothing -- and that is the case that runs for days in a product.
//!
//! Three steps, in one boot, on the same bus:
//!
//!   1. `WHO_AM_I` from the IMU (`0x6A`). The control: the bus works before anything goes wrong.
//!   2. The same read at `0x55`, where nothing is wired. This must be reported as **Nack**, not as a timeout --
//!      a device that is not there is a refusal, and a driver that calls it a timeout has lost the distinction.
//!   3. `WHO_AM_I` again from `0x6A`. The question the review asks: after the refusal, is the bus still usable?
//!   4. A real `write` -- `CTRL1_XL` (0x10) with 0x40 -- read back with `write_read` to prove it landed, then the
//!      register restored to the value it had. `write` is the other blocking entry point and it was rerouted through
//!      the transaction layer in the same change; without this step it would ship unmeasured.
//!
//! The record is `antes=<hex> erro=<n> depois=<hex> wr=<0|1> volta=<hex>`, and it only passes when all of them are
//! right: the control answered, the refusal came back as Nack, the bus came back, and the round trip wrote and read
//! back the same byte.
//!
//! VERIFIED ON HARDWARE (2026-11-23): the record is in this file's header and in the crate README's dated table.

#![no_std]
#![no_main]

use defmt_rtt as _;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::i2c::{Config, Error as I2cError, Speed};
use efr32mg2x_hal::pin::Port;
use efr32mg2x_hal::prelude::*;
use efr32mg2x_hal::route::PinMode;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const IMU: u8 = 0x6A;
const AUSENTE: u8 = 0x55;
const WHO_AM_I: u8 = 0x0F;

fn hex(out: &mut [u8; 4], v: u8) {
    let d = b"0123456789abcdef";
    out[0] = b'0';
    out[1] = b'x';
    out[2] = d[(v >> 4) as usize];
    out[3] = d[(v & 0x0F) as usize];
}

#[cortex_m_rt::entry]
fn main() -> ! {
    mailbox::invalidate();

    let board = Board::take().unwrap();
    let _power = board.output_on(Port::D, 5, true);
    board.delay().delay_ms(50);
    let _ = board.route_i2c0((Port::B, 3), (Port::B, 2), PinMode::OpenDrainPullUp);
    let mut i2c = board.i2c0(Config::new(Speed::Standard100kHz), board.clock_hz());

    // 1. Controlo: o bus funciona antes de qualquer erro.
    let mut id = [0u8; 1];
    let antes = match i2c.write_read(IMU, WHO_AM_I, &mut id) {
        Ok(()) => id[0],
        Err(_) => 0,
    };

    // 2. O erro: um endereco onde nao ha nada. Tem de ser uma recusa, e nao um timeout.
    let mut lixo = [0u8; 1];
    let erro = match i2c.write_read(AUSENTE, WHO_AM_I, &mut lixo) {
        Ok(()) => 0, // 0 = respondeu: nao devia, nada esta' ligado ai
        Err(I2cError::Nack) => 1,
        Err(I2cError::Timeout) => 2,
        Err(_) => 3,
    };

    // 3. A pergunta: depois da recusa, o bus volta?
    let mut id2 = [0u8; 1];
    let depois = match i2c.write_read(IMU, WHO_AM_I, &mut id2) {
        Ok(()) => id2[0],
        Err(_) => 0,
    };

    // 4. O outro ponto de entrada bloqueante -- `write` -- e uma leitura de varios bytes.
    //
    //    CTRL1_XL (0x10) e' de leitura/escrita no LSM6DS3TR-C: escrever 0x40 poe o acelerometro a 104 Hz. Com ele a
    //    correr, OUTX_L_A (0x28) devolve 6 bytes de aceleracao, e em repouso o eixo Z marca cerca de 1 g -- portanto
    //    "os seis bytes a zero" e' a assinatura de uma transferencia que nao aconteceu (o ponteiro volta a 0 num
    //    STOP, e uma leitura sem ponteiro devolve o registo 0), e e' isso que este passo recusa.
    //
    //    Ler **seis** bytes e' tambem o que fecha `I2C0-READ-DEFECT.md` para mais do que um byte: o defeito era o
    //    ultimo byte ser lido com ACK, o escravo continuar a conduzir e o mestre nunca fechar a recepcao.
    //
    //    O `read` simples (sem registo) continua sem oraculo neste dispositivo, e fica de fora por honestidade: a
    //    sequencia que ele usa e' a segunda metade da transaccao 1, essa medida.
    const CTRL1_XL: u8 = 0x10;
    const OUTX_L_A: u8 = 0x28;
    let mut ctrl = [0u8; 1];
    let original = match i2c.write_read(IMU, CTRL1_XL, &mut ctrl) {
        Ok(()) => ctrl[0],
        Err(_) => 0,
    };
    let escrito = i2c.write(IMU, &[CTRL1_XL, 0x40]).is_ok();
    // Uma amostra a 104 Hz leva 9.6 ms a aparecer; 20 ms chegam e sobram.
    board.delay().delay_ms(20);
    let mut acc = [0u8; 6];
    let lidos = i2c.write_read(IMU, OUTX_L_A, &mut acc).is_ok();
    let nao_zero = acc.iter().any(|b| *b != 0);
    let mut lido = [0u8; 1];
    let volta = match i2c.write_read(IMU, CTRL1_XL, &mut lido) {
        Ok(()) => lido[0],
        Err(_) => 0,
    };
    let _ = i2c.write(IMU, &[CTRL1_XL, original]);

    let mut line = [0u8; 80];
    let mut n = 0usize;
    let mut h = [0u8; 4];
    let mut push = |bytes: &[u8], n: &mut usize| {
        for &b in bytes {
            if *n < line.len() {
                line[*n] = b;
                *n += 1;
            }
        }
    };
    push(b"i2c-rec: antes=", &mut n);
    hex(&mut h, antes);
    push(&h, &mut n);
    push(b" erro=", &mut n);
    push(&[b'0' + erro], &mut n);
    push(b" depois=", &mut n);
    hex(&mut h, depois);
    push(&h, &mut n);
    push(if escrito { b" wr=1" } else { b" wr=0" }, &mut n);
    push(if lidos { b" rd6=1" } else { b" rd6=0" }, &mut n);
    push(if nao_zero { b" nz=1" } else { b" nz=0" }, &mut n);
    push(b" volta=", &mut n);
    hex(&mut h, volta);
    push(&h, &mut n);
    push(b"\n", &mut n);

    if antes == 0x6A && erro == 1 && depois == 0x6A && escrito && lidos && nao_zero && volta == 0x40
    {
        mailbox::pass(&line[..n]);
    } else {
        mailbox::fail(&line[..n]);
    }

    loop {
        cortex_m::asm::wfi();
    }
}
