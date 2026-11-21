//! O tick de 64 bits do SYSRTC, o comparador a acordar o núcleo, e um relógio independente a confirmá-lo.
//!
//! ## As duas fases, e porque é que são duas
//!
//! **Fase A -- dormir.** Dez prazos de 100 ms armados no comparador, com o núcleo em `wfi` entre eles. Prova que o
//! comparador dispara (a contagem de interrupções) e que o tick de 64 bits mede o total.
//!
//! **Fase B -- girar.** Os mesmos dez intervalos, mas à espera do tick largo **sem** `wfi`. Aqui o contador de
//! ciclos do DWT vale, porque o núcleo está acordado.
//!
//! A segunda fase existe por causa de uma medição, não por simetria. A primeira versão cronometrava a fase A com o
//! DWT e leu `wall_ms=1` para um segundo de sono: **o `CYCCNT` conta ciclos do núcleo, e com o núcleo em `wfi` o
//! relógio do núcleo está parado**. O instrumento não podia responder à pergunta, e um critério que o usasse estava
//! a medir a ferramenta em vez do driver. Na fase B o DWT mede um intervalo que contém o mesmo contador do SYSRTC,
//! e os dois números passam a poder discordar -- que é o que faz deles um controlo.
//!
//! ## O critério
//!
//! Dez disparos do comparador; o tick largo diz 1000 ms nas duas fases; o DWT, na fase em que vale, concorda com o
//! tick largo dentro de 5 %; nenhuma volta perdida. A tolerância cobre o cristal de 32 kHz e o custo de acordar.
//!
//! Lê-se com `scripts/read-mailbox.sh`.

#![no_std]
#![no_main]

#[path = "common/mailbox.rs"]
mod mailbox;

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m_rt::entry;
use efr32mg2x_hal::board::Board;
use efr32mg2x_hal::pin::Port;
// `panic-probe` e não `panic-halt`: o `defmt` que a HAL liga precisa de um transporte, e o panic sobre o mesmo
// cabo de SWD é o que transforma um panic mudo numa linha legível.
use defmt_rtt as _;
use panic_probe as _;

/// Quantas esperas de 100 ms cada fase faz.
const ESPERAS: u32 = 10;

/// Uma espera de 100 ms no contador do SYSRTC: 3276,8 ticks a 32768 Hz, arredondado para baixo.
const TICKS_100MS: u32 = 3_276;

/// Ticks do SYSRTC num segundo, para o relatório.
const TICKS_HZ: u64 = 32_768;

/// Quantas vezes o comparador acordou o núcleo.
///
/// O handler escreve aqui porque o `main` está dentro do `wfi`: não há como emprestar-lhe o `Sysrtc`, e sem um
/// sítio partilhado onde contar, "acordou dez vezes" e "acordou uma e ficou preso" davam o mesmo registo.
static ACORDARES: AtomicU32 = AtomicU32::new(0);

/// O handler forte que substitui o `DefaultHandler` do vector.
///
/// O PAC declara `PROVIDE(SYSRTC_APP_IRQHandler = DefaultHandler)` em `device.x`, portanto uma definição com este
/// nome exacto toma o lugar do handler que não regressa. O atributo `#[interrupt]` do `cortex-m-rt` precisa da
/// feature `device`, que a HAL não liga; o nome do vector é o que o linker procura.
#[no_mangle]
pub extern "C" fn SYSRTC_APP_IRQHandler() {
    efr32mg2x_hal::sysrtc::Sysrtc::acknowledge_compare_interrupt();
    ACORDARES.fetch_add(1, Ordering::Relaxed);
}

/// Um relatório só, com os números que decidem.
fn report(
    ok: bool,
    sleep_ms: u32,
    wakes: u32,
    spin_ms: u32,
    wall_b_ms: u32,
    cyc0: u32,
    cyc1: u32,
    wraps: u32,
) {
    let mut m = [0u8; 160];
    let mut n = 0usize;

    fn texto(m: &mut [u8], n: &mut usize, s: &[u8]) {
        for b in s {
            if *n < m.len() {
                m[*n] = *b;
                *n += 1;
            }
        }
    }
    fn numero(m: &mut [u8], n: &mut usize, v: u32) {
        let mut d = [0u8; 10];
        let mut k = 0usize;
        let mut v = v;
        loop {
            d[k] = b'0' + (v % 10) as u8;
            v /= 10;
            k += 1;
            if v == 0 || k == d.len() {
                break;
            }
        }
        while k > 0 {
            k -= 1;
            if *n < m.len() {
                m[*n] = d[k];
                *n += 1;
            }
        }
    }

    texto(&mut m, &mut n, b"wide: sleep_ms=");
    numero(&mut m, &mut n, sleep_ms);
    texto(&mut m, &mut n, b" wakes=");
    numero(&mut m, &mut n, wakes);
    texto(&mut m, &mut n, b" spin_ms=");
    numero(&mut m, &mut n, spin_ms);
    texto(&mut m, &mut n, b" spin_wall=");
    numero(&mut m, &mut n, wall_b_ms);
    texto(&mut m, &mut n, b" c0=");
    numero(&mut m, &mut n, cyc0);
    texto(&mut m, &mut n, b" c1=");
    numero(&mut m, &mut n, cyc1);
    texto(&mut m, &mut n, b" wraps=");
    numero(&mut m, &mut n, wraps);

    if ok {
        mailbox::write(mailbox::Status::Pass, &m[..n]);
    } else {
        mailbox::write(mailbox::Status::Fail, &m[..n]);
    }
}

#[entry]
fn main() -> ! {
    mailbox::invalidate();
    let board = Board::take().unwrap();
    let mut led = board.output_on(Port::A, 7, false);
    let rtc = board.sysrtc();

    // A interrupção do comparador tem de estar ligada no periférico antes do primeiro `wfi`.
    rtc.unmask_interrupt();

    // `Hertz` é um tipo com a unidade dentro: converter para inteiro aqui é explícito, não um `as` disfarçado.
    let hw_hz: u32 = board.clock_hz().into();
    let _ = unsafe { efr32mg2x_hal::dwt::start() };

    // ## Fase A: dormir em cada prazo
    let ticks_a0 = rtc.now_wide();
    for _ in 0..ESPERAS {
        // O prazo sai do tick LARGO e só a truncagem para 32 bits é feita no fim: é essa a aritmética que
        // sobrevive a uma volta do contador, e é o que `schedule_at` recebe.
        let alvo = rtc.now_wide() + TICKS_100MS as u64;
        let antes = ACORDARES.load(Ordering::Relaxed);
        rtc.schedule_at(alvo as u32);

        // Espera pelo disparo do comparador. O limite de iterações existe para que um comparador mudo dê um
        // vermelho em vez de um exemplo pendurado -- um `wfi` regressa com qualquer interrupção, não só a nossa.
        let mut voltas = 0u32;
        while ACORDARES.load(Ordering::Relaxed) == antes {
            cortex_m::asm::wfi();
            voltas += 1;
            if voltas > 4_000_000 {
                break;
            }
        }
    }
    let ticks_a = (rtc.now_wide() - ticks_a0) as u32;
    let wakes = ACORDARES.load(Ordering::Relaxed);

    // ## Fase B: os mesmos intervalos, com o núcleo acordado
    //
    // Sem `wfi`, e à espera do MESMO tick largo. É a fase em que o DWT mede tempo de parede; na fase A o contador
    // do núcleo pára quando o núcleo dorme.
    // **O DWT não sobrevive ao sono.** Medido duas vezes, de duas maneiras: com o contador a cronometrar a fase A
    // deu 1 ms para um segundo (o núcleo dorme e o relógio do núcleo pára), e com os valores em bruto no registo
    // viu-se `c0=27297 c1=27297` -- congelado no primeiro `wfi`, sem voltar depois de acordar. Portanto ele é
    // reiniciado AQUI, depois do sono, e só mede a fase em que o núcleo não dorme. Um instrumento que não
    // responde à pergunta não entra no critério.
    let _ = unsafe { efr32mg2x_hal::dwt::start() };
    let ticks_b0 = rtc.now_wide();
    let ciclos_b0 = efr32mg2x_hal::dwt::cycles();
    for _ in 0..ESPERAS {
        let alvo = rtc.now_wide() + TICKS_100MS as u64;
        // `is_future` é a comparação em 32 bits que sobrevive à volta do contador; a decisão vive no núcleo
        // testado, não aqui.
        while efr32mg2x_hal::cores::sysrtc::is_future(rtc.now(), alvo as u32) {
            cortex_m::asm::nop();
        }
    }
    let ticks_b = (rtc.now_wide() - ticks_b0) as u32;
    let ciclos_b1 = efr32mg2x_hal::dwt::cycles();
    let ciclos = ciclos_b1.wrapping_sub(ciclos_b0);

    let sleep_ms = (ticks_a as u64 * 1000 / TICKS_HZ) as u32;
    let spin_ms = (ticks_b as u64 * 1000 / TICKS_HZ) as u32;
    let wall_b_ms = if hw_hz == 0 {
        0
    } else {
        (ciclos as u64 * 1000 / u64::from(hw_hz)) as u32
    };

    let esperado = ESPERAS * 100;
    let margem = esperado / 20;
    let perto = |v: u32| v >= esperado - margem && v <= esperado + margem;
    // O DWT e o SYSRTC medem o mesmo intervalo na fase B: um factor de dois, uma volta perdida ou um relógio
    // trocado aparecem aqui como uma diferença que a margem não cobre.
    let concordam = wall_b_ms.abs_diff(spin_ms) <= margem;
    let ok = wakes == ESPERAS && perto(sleep_ms) && perto(spin_ms) && concordam;

    report(
        ok,
        sleep_ms,
        wakes,
        spin_ms,
        wall_b_ms,
        ciclos_b0,
        ciclos_b1,
        rtc.wraps(),
    );
    led.on();

    loop {
        cortex_m::asm::wfi();
    }
}
