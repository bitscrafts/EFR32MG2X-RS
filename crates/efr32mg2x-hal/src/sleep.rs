//! Entrar em EM2 — e sair.
//!
//! ## Porque é que isto é uma API e não um `wfi` no exemplo
//!
//! Entrar em EM2 não é dormir: é desligar o relógio de alta frequência e ficar dependente de uma
//! interrupção que já esteja armada. Um `wfi` solto com o `SLEEPDEEP` posto e sem fonte de acordar
//! deixa o aparelho a dormir até alguém lhe tocar no reset — e o sintoma, visto de fora, é uma placa
//! que morreu.
//!
//! Nesta série, o `EMU` tem `CTRL.EM2DBGEN` e **não** tem bloqueio de EM2: quem impede a entrada são
//! as condições do resto do sistema, não um bit. As duas que este módulo trata:
//!
//! - **o debugger**: com `EM2DBGEN = 0`, a ligação de debug morre em EM2 e não volta com a placa a
//!   correr — [`Sleep::keep_debugger`] liga-o antes de dormir;
//! - **o `SLEEPDEEP`**: fica posto durante o `wfi` e é limpo a seguir, para que o próximo `wfi` que
//!   alguém escreva à mão não entre em EM2 sem saber.
//!
//! ## O que este módulo não faz
//!
//! Não arma fontes de acordar. Quem chama tem de ter a interrupção ligada no periférico **e** no
//! `NVIC` antes de entrar, senão o `wfi` não regressa. É de propósito: escolher a fonte de acordar é
//! decisão do produto, e um `sleep_ms` que decidisse por nós seria a mesma classe de suposição que o
//! `19_000_000` escondia no driver do timer.

use crate::pac;

/// A entrada e a saída de EM2.
pub struct Sleep<'a> {
    emu: &'a pac::emu_ns::RegisterBlock,
}

impl<'a> Sleep<'a> {
    pub(crate) fn new(emu: &'a pac::emu_ns::RegisterBlock) -> Self {
        Self { emu }
    }

    /// Mantém a ligação de debug viva enquanto o núcleo está em EM2.
    ///
    /// Sem isto, `probe-rs` perde o alvo no primeiro sono e o que se observa é uma placa muda — que é
    /// indistinguível de uma placa avariada. Ligar custa microamperes e compra a possibilidade de ver.
    pub fn keep_debugger(&self) {
        self.emu.ctrl().modify(|_, w| w.em2dbgen().set_bit());
    }

    /// Dorme em EM2 até uma interrupção armada acordar o núcleo.
    ///
    /// Não regressa sem uma interrupção: quem chamar tem de a ter ligada no periférico e no `NVIC`.
    pub fn enter(&self) {
        // `cortex_m::Peripherals::steal` já é o que o `pwm` faz para o DWT: o `SCB` não vem no PAC do
        // chip, e passar um `&mut SCB` por toda a cadeia só para pôr um bit não melhora a segurança.
        // SAFETY: `Board` took `cortex_m::Peripherals` once and keeps it; the `SCB` is not part of this chip's PAC, so
        // it is borrowed here and only `SCR.SLEEPDEEP` is touched -- the one bit this crate writes on that block.
        let mut scb = unsafe { cortex_m::Peripherals::steal() }.SCB;
        scb.set_sleepdeep();
        cortex_m::asm::wfi();
        // Limpo a seguir: uma entrada acidental em EM2 mais tarde seria um defeito silencioso.
        scb.clear_sleepdeep();
    }
}
