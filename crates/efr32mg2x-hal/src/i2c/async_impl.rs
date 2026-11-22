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

// ## Porque é que isto não usa a interrupção do I2C
//
// A primeira versão deste ficheiro armava `IEN` para `txc`/`rxdatav`/`nack`/`mstop` e acordava a tarefa na
// interrupção. Medido na placa: **44 647 591** entradas no handler em poucos segundos (`ICSR = 0x82b`, vector 43),
// porque neste periférico o flag de `IF` volta a afirmar-se enquanto a condição for verdadeira -- limpar não o
// desarma, ao contrário de um evento latched. Uma versão futura tem de desarmar o bit certo por espera, e não
// armar todos de uma vez.
//
// Enquanto isso, a espera é esta: o futuro devolve `Pending` e **pede para ser sondado outra vez**. Não bloqueia
// nada -- o executor corre outras tarefas entre sondagens --, mas também não deixa o núcleo dormir enquanto uma
// transferência está em curso, que a 100 kHz são cerca de um milissegundo por dezena de bytes.

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

/// Um futuro que termina quando `pronto` disser que sim, ou com erro quando o barramento recusar.
///
/// O limite de voltas não é um `timeout` de tempo real: conta as **vezes que a tarefa voltou a ser acordada**, e
/// existe para uma interrupção perdida virar um erro visível em vez de uma tarefa que nunca mais corre. Uma
/// transferência de I2C a 100 kHz levanta estas notificações dezenas de vezes, não milhares.
struct Ate<F> {
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

        // Sondar outra vez, sem interrupção. Ver a nota no topo do ficheiro.
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

impl I2c0 {
    /// O estado do barramento, para o predicado de [`Ate`].
    fn espera<F>(&self, pronto: F) -> Ate<F>
    where
        F: FnMut(&efr32mg2x_pac::i2c0_ns::RegisterBlock) -> bool,
    {
        Ate { pronto }
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
        self.espera(|r| r.status().read().txbl().bit_is_set())
            .await?;
        self.i2c.txdata().write(|w| unsafe { w.bits(byte as u32) });
        ULTIMA_ESPERA.store(2, core::sync::atomic::Ordering::Relaxed);
        ESPERAS_TXC.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        self.espera(|r| r.status().read().txc().bit_is_set())
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
            self.espera(|r| !r.status().read().pstart().bit_is_set())
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
                    for slot in buffer.iter_mut() {
                        // O flag de **evento**, não o bit de estado: medido neste periférico, no instante em que a
                        // tarefa ficou presa o `IF` tinha `RXDATAV` posto e o `STATUS` não, com `BUSHOLD` em cima
                        // (`IF=0x879`, `STATUS=0xc2`). Quem responde à pergunta "chegou um byte" é o flag.
                        self.espera(|r| r.if_().read().rxdatav().bit_is_set())
                            .await?;
                        *slot = self.i2c.rxdata().read().bits() as u8;
                        // O flag é latched: sem o limpar, a espera seguinte vê o mesmo byte outra vez. Escrever 1
                        // limpa, como em todos os `IF` desta família.
                        // SAFETY: a mesma janela segura e a mesma escrita de 1-para-limpar usadas no caminho
                        // bloqueante; a limpeza de um flag não tem outro efeito.
                        let seguro =
                            unsafe { crate::alias::clear_window(efr32mg2x_pac::I2c0S::ptr()) };
                        seguro.if_().write(|w| w.rxdatav().set_bit());
                    }
                }
            }
        }

        self.i2c.cmd().write(|w| w.stop().set_bit());
        self.espera(|r| !r.status().read().pstop().bit_is_set())
            .await?;
        Ok(())
    }
}

// `WAIT_POLLS` é o limite do caminho bloqueante; aqui não se usa, e mantê-lo importado seria ruído. A constante
// fica referida nesta nota para quem procurar o paralelo entre os dois caminhos.
const _: u32 = WAIT_POLLS;
