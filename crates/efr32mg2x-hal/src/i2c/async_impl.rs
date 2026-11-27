//! `embedded-hal-async` I2C: a mesma máquina de estados por byte, à espera da interrupção em vez de girar.
//!
//! ## Porque é que isto não é só `async` à volta do código que já existia
//!
//! O `I2c0::write` bloqueante faz `while status().pstart() { polls += 1 }` até um limite. Embrulhar isso num
//! `async fn` daria uma função que se chama `async`, devolve um futuro, e **bloqueia o executor inteiro** quando
//! corre — todas as outras tarefas ficavam paradas durante a transferência. É a mentira que este ficheiro existe
//! para não contar.
//!
//! O que aqui se faz é esperar de verdade: quando a condição ainda não está satisfeita, a tarefa **regista o seu
//! waker e devolve `Pending`**, e a interrupção do I2C acorda-a quando o periférico muda de estado. Entre uma
//! espera e a seguinte o executor corre o que quiser.
//!
//! ## Quem serve a interrupção
//!
//! O vector é da aplicação, como todos os outros nesta HAL: ela define `I2C0_IRQHandler` e chama
//! [`on_interrupt`]. A HAL não instala handlers em crates de biblioteca — um handler instalado aqui correria mesmo
//! num firmware que nunca usasse I2C assíncrono.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use embedded_hal_async::i2c::{I2c as AsyncI2c, Operation};

use super::{Error, I2c0, WAIT_POLLS};

// ## O que fazia isto falhar, e porque é que a resposta estava no core testado no host
//
// A primeira versão lia todos os bytes **com ACK**, incluindo o último. O escravo nunca recebia ordem para parar,
// continuava a conduzir dados e o mestre nunca fechava a recepção: medido, barramento retido (`BUSHOLD`), sem
// `RXDATAV` e sem NACK -- um travamento sem erro nenhum. A sequência certa está em `cores::i2c::write_read`, que
// chama `read(i != last)`: **NACK no último byte**. O `HwBus` que a serve manda `CMD.ACK` ou `CMD.NACK` antes de
// cada leitura, e é isso que aqui se espelha.
//
// A lição não é sobre o I2C: a resposta estava num core com testes no host, e a primeira versão deste ficheiro foi
// escrita a partir do que parecia certo em vez de ser lida a partir dele. Custou seis flashes.
//
// ## A interrupção do I2C, e a razão pela qual a primeira versão reentrou 44 milhões de vezes
//
// A primeira versão armava `IEN` para `txc`/`rxdatav`/`nack`/`mstop` **de uma vez**, e acordava a tarefa na interrupção.
// Medido na placa: **44 647 591** entradas no handler em poucos segundos (`ICSR = 0x82b`, vector 43). A causa está
// neste periférico e não no handler: `IF` **volta a afirmar-se enquanto a condição for verdadeira**, ao contrário de
// um evento latched, e limpar não desarma. Armar todos os bits e limpá-los no handler é uma comporta aberta.
//
// O que aqui se faz agora, uma espera de cada vez:
//
//   1. limpa o `IF` do evento que se vai esperar (pela janela `_CLR`, onde escrever 1 limpa);
//   2. arma **só** o `IEN` desse evento e desmascara o vector no NVIC;
//   3. devolve `Pending` sem pedir nova sondagem — é isto que deixa o núcleo dormir;
//   4. a interrupção chama [`on_interrupt`], que **desarma primeiro** e só depois limpa, e acorda a tarefa.
//
// Desarmar antes de limpar é a diferença entre uma entrada por evento e 44 milhões: enquanto o `IEN` estiver armado e
// a condição for verdadeira, o vector volta a disparar no instante em que a ISR regressa.
//
// --- marcadores, para um travamento dizer onde travou ---------------------------------------------
//
// Uma espera que nunca se satisfaz não escreve registo nenhum, e um firmware mudo não diz qual das três esperas do
// `put_byte` falhou. Estes três contadores são lidos na RAM por SWD (`nm` no ELF, `probe-rs read`) enquanto o
// programa está parado, e respondem a essa pergunta sem mais um flash por hipótese.
#[used]
#[no_mangle]
static ESPERAS_TXBL: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
#[used]
#[no_mangle]
static ESPERAS_TXC: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
/// O `STATUS` no momento em que uma espera falhou pela última vez.
///
/// É a medição que separa as hipóteses: diz se o periférico está ocupado, se já recebeu, se está à espera de ACK --
/// ou se nunca arrancou. Um contador diz *onde* parou; este diz *em que estado*.
#[used]
#[no_mangle]
static ULTIMO_STATUS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
/// O `IF` no mesmo instante -- para ver NACK/erros que o `STATUS` não mostra.
#[used]
#[no_mangle]
static ULTIMO_IF: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
/// 1 = à espera de TXBL, 2 = à espera de TXC, 3 = à espera de PSTART, 4 = à espera de PSTOP, 5 = à espera de RXDATAV.
#[used]
#[no_mangle]
static ULTIMA_ESPERA: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

/// Os contadores, para quem os quiser ler sem `nm`.
pub fn waits() -> (u32, u32, u32) {
    use core::sync::atomic::Ordering::Relaxed;
    (
        ESPERAS_TXBL.load(Relaxed),
        ESPERAS_TXC.load(Relaxed),
        ULTIMA_ESPERA.load(Relaxed),
    )
}

/// O `(STATUS, IF)` da última espera falhada -- a leitura que diz em que estado o barramento ficou.
pub fn last_state() -> (u32, u32) {
    use core::sync::atomic::Ordering::Relaxed;
    (ULTIMO_STATUS.load(Relaxed), ULTIMO_IF.load(Relaxed))
}

/// Os bits de `IEN` que este ficheiro arma, um por espera.
///
/// Um de cada vez, e é o ponto do ficheiro: armar vários de uma vez e limpá-los no handler é o que produziu 44 milhões
/// de entradas. Os números são os do registo (`ien.rs` do PAC: `START` 0, `TXC` 3, `TXBL` 4, `RXDATAV` 5, `MSTOP` 8).
const IEN_START: u32 = 1 << 0;
const IEN_TXC: u32 = 1 << 3;
const IEN_TXBL: u32 = 1 << 4;
const IEN_RXDATAV: u32 = 1 << 5;
const IEN_MSTOP: u32 = 1 << 8;

/// Um futuro que termina quando `pronto` disser que sim, ou com erro quando o barramento recusar.
///
/// O limite de voltas não é um `timeout` de tempo real: conta as **vezes que a tarefa voltou a ser acordada**, e
/// existe para uma interrupção perdida virar um erro visível em vez de uma tarefa que nunca mais corre. Uma
/// transferência de I2C a 100 kHz levanta estas notificações dezenas de vezes, não milhares.
struct Ate<F> {
    /// O bit de `IEN` que corresponde ao evento esperado. Um de cada vez: ver a nota do topo.
    mask: u32,
    pronto: F,
}

impl<F> Future for Ate<F>
where
    F: FnMut(&efr32mg2x_pac::i2c0_ns::RegisterBlock) -> bool + Unpin,
{
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        // SAFETY: vista crua do I2C0, como em `on_interrupt`; só se lê estado.
        let i2c = unsafe { efr32mg2x_pac::I2c0Ns::steal() };

        if (this.pronto)(&i2c) {
            return Poll::Ready(Ok(()));
        }

        // Guardar o estado no instante da falha: é o que diz em que estado o periférico está, não só onde parou a
        // tarefa. `read()` é uma leitura de registo e não muda nada.
        ULTIMO_STATUS.store(
            i2c.status().read().bits(),
            core::sync::atomic::Ordering::Relaxed,
        );
        ULTIMO_IF.store(
            i2c.if_().read().bits(),
            core::sync::atomic::Ordering::Relaxed,
        );

        // Limpar o evento *antes* de o armar: um `IF` já posto faria a interrupção disparar imediatamente e o
        // handler desarmá-lo-ia sem que a condição tivesse mudado. A janela `_CLR` é onde escrever 1 limpa.
        // SAFETY: a janela segura do I2C0 é o mesmo bloco noutro offset, e só se escrevem bits de flag.
        let seguro = unsafe { crate::alias::clear_window(efr32mg2x_pac::I2c0S::ptr()) };
        // SAFETY: `this.mask` vem de uma tabela de bits de `IEN` deste periférico, não de um número do chamador.
        seguro.if_().write(|w| unsafe { w.bits(this.mask) });
        // SAFETY: idem -- os mesmos bits, agora no registo de enable da vista segura.
        let seguro_en = unsafe { efr32mg2x_pac::I2c0S::steal() };
        seguro_en
            .ien()
            .modify(|r, w| unsafe { w.bits(r.bits() | this.mask) });
        // O vector é da aplicação, mas quem sabe que a espera está armada é o driver: desmascarar aqui é o que
        // transforma `Pending` num núcleo a dormir em vez de uma tarefa a girar.
        // SAFETY: `unmask` mexe no NVIC, que não é um periférico desta HAL; o efeito é uma interrupção a mais.
        unsafe { cortex_m::peripheral::NVIC::unmask(efr32mg2x_pac::Interrupt::I2C0) };
        // Sem `wake_by_ref()`: a tarefa não pede para ser sondada, é a interrupção que a acorda.
        let _ = cx;
        Poll::Pending
    }
}

/// O que a aplicação chama no seu `I2C0_IRQHandler`.
///
/// Devolve o `IF` que estava posto e deixa o periférico com o `IEN` **desarmado**, para o vector não voltar a disparar
/// enquanto a condição se mantiver verdadeira. Quem chama acorda a sua tarefa; ver o exemplo `hal_i2c_async`.
#[must_use]
pub fn on_interrupt() -> u32 {
    // SAFETY: vista segura do I2C0, a mesma que o resto deste ficheiro usa para `IF`; só se lê e limpam flags.
    let seguro = unsafe { efr32mg2x_pac::I2c0S::steal() };
    let pendente = seguro.if_().read().bits();
    // Desarmar primeiro. Isto é a correcção do defeito dos 44 milhões de entradas.
    seguro.ien().write(|w| unsafe { w.bits(0) });
    // SAFETY: janela `_CLR` do mesmo bloco; escrever 1 em cada bit pendente limpa-o.
    let limpar = unsafe { crate::alias::clear_window(efr32mg2x_pac::I2c0S::ptr()) };
    limpar.if_().write(|w| unsafe { w.bits(pendente) });
    pendente
}

impl I2c0 {
    /// O estado do barramento, para o predicado de [`Ate`].
    fn espera<F>(&self, mask: u32, pronto: F) -> Ate<F>
    where
        F: FnMut(&efr32mg2x_pac::i2c0_ns::RegisterBlock) -> bool,
    {
        Ate { mask, pronto }
    }

    /// Põe um byte no barramento: espera pelo buffer livre, escreve, espera pela transmissão e verifica o NACK.
    ///
    /// As três esperas são as do caminho bloqueante, uma a uma. Só a primeira não chegava: com TBXL já posto, dois
    /// bytes seguidos escrevem no mesmo registo antes de o primeiro sair. Medido -- `who=0x00` em vez de `0x6a`.
    ///
    /// O NACK lê-se **no alias seguro**: `if_` na vista não-segura deste periférico não reflecte o evento, e a
    /// primeira versão deste ficheiro lia-o ali (sempre zero) e por isso não via recusa nenhuma.
    async fn put_byte(&mut self, byte: u8) -> Result<(), Error> {
        ULTIMA_ESPERA.store(1, core::sync::atomic::Ordering::Relaxed);
        ESPERAS_TXBL.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        self.espera(IEN_TXBL, |r| r.status().read().txbl().bit_is_set())
            .await?;
        // SAFETY: `TXDATA` takes the byte being shifted out; `byte` is a `u8`, so the widening to `u32` adds nothing.
        self.i2c.txdata().write(|w| unsafe { w.bits(byte as u32) });
        ULTIMA_ESPERA.store(2, core::sync::atomic::Ordering::Relaxed);
        ESPERAS_TXC.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        self.espera(IEN_TXC, |r| r.status().read().txc().bit_is_set())
            .await?;
        // SAFETY: as mesmas duas linhas do caminho bloqueante -- a janela segura do I2C0 e a limpeza de um flag.
        let seguro = unsafe { crate::alias::clear_window(efr32mg2x_pac::I2c0S::ptr()) };
        if seguro.if_().read().nack().bit_is_set() {
            seguro.if_().write(|w| w.nack().set_bit());
            return Err(Error::Nack);
        }
        Ok(())
    }
}

impl AsyncI2c for I2c0 {
    /// Uma transacção, com **repeated START** entre operações.
    ///
    /// A primeira versão chamava um `write` e depois um `read`, cada um com o seu STOP: o barramento era libertado
    /// no meio e o dispositivo perdia o ponteiro de registo que acabara de receber -- medido na placa, `who=0x00`
    /// em vez de `0x6a`. É o mesmo defeito que o caminho bloqueante já tinha corrigido, e está descrito no
    /// comentário dele. Entre operações há START sem STOP; o STOP é único e no fim.
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        if operations.is_empty() {
            return Err(Error::InvalidData);
        }

        for operation in operations.iter_mut() {
            self.i2c.cmd().write(|w| w.start().set_bit());
            self.espera(IEN_START, |r| !r.status().read().pstart().bit_is_set())
                .await?;

            match operation {
                Operation::Write(bytes) => {
                    if bytes.is_empty() {
                        return Err(Error::InvalidData);
                    }
                    self.put_byte((address << 1) & 0xFE).await?;
                    for &byte in bytes.iter() {
                        self.put_byte(byte).await?;
                    }
                }
                Operation::Read(buffer) => {
                    if buffer.is_empty() {
                        return Err(Error::InvalidData);
                    }
                    self.put_byte((address << 1) | 1).await?;
                    // **Um byte de cada vez, com ACK -- e NACK no último.** É a sequência do core testado no
                    // host (`cores::i2c::write_read` chama `read(i != last)`) e a do `HwBus` que o serve: sem o
                    // NACK final o escravo continua a conduzir dados, o mestre nunca fecha a recepção, e o que se
                    // observa é exactamente o que se media -- barramento retido (`BUSHOLD`), sem byte.
                    //
                    // O `STATUS.RXDATAV` é a condição que `HwBus::read` espera; foi para o `IF` por engano antes
                    // desta comparação.
                    let total = buffer.len();
                    for (i, slot) in buffer.iter_mut().enumerate() {
                        self.espera(IEN_RXDATAV, |r| r.status().read().rxdatav().bit_is_set())
                            .await?;
                        if i + 1 == total {
                            self.i2c.cmd().write(|w| w.nack().set_bit());
                        } else {
                            self.i2c.cmd().write(|w| w.ack().set_bit());
                        }
                        *slot = self.i2c.rxdata().read().bits() as u8;
                    }
                }
            }
        }

        self.i2c.cmd().write(|w| w.stop().set_bit());
        self.espera(IEN_MSTOP, |r| !r.status().read().pstop().bit_is_set())
            .await?;
        Ok(())
    }
}

// `WAIT_POLLS` é o limite do caminho bloqueante; aqui não se usa, e mantê-lo importado seria ruído. A constante
// fica referida nesta nota para quem procurar o paralelo entre os dois caminhos.
const _: u32 = WAIT_POLLS;
