# Relatório e Paper — estrutura com evidência medida

**Projecto:** dosimeter-mg24 · **Curso:** CK0475 Aprendizagem de Máquina Probabilística
**Hardware:** XIAO MG24 Sense (microfone na placa) + telemóvel por BLE

> Regra deste documento: **cada número tem de vir de um teste que corre**. O que ainda não
> está medido aparece marcado `[A MEDIR]` — e não entra no paper até o estar.

---

# PARTE 1 — RELATÓRIO TÉCNICO

## 1. Enquadramento e decisão de hardware

O trabalho começou num par de microfones I2S para estimar direcção (ITD/ILD). Um dos
módulos avariou e, por estar num barramento partilhado, inutilizou o par: com ele ligado,
os dois canais liam zero. Ficou medido e documentado no projecto anterior:

```
só o modulo bom        picoR = 17 000-18 000     le'
só o modulo avariado   picoL = 0 e picoR = 0     apaga os dois canais
causa                  a linha DOUT e' um so' fio; o modulo avariado amarra-a ao GND
```

**A física decide o resto:** a direcção de um som vive na diferença de chegada a dois pontos
(`τ = d·sin θ / c`). Com um ponto de medida não existe τ. A reformulação não perdeu nenhum
dos pontos exigidos pelo enunciado — perdeu a direcção, que não é exigida.

**Decisão:** XIAO MG24 Sense, com microfone na placa. Elimina todo o hardware externo, e
portanto o modo de falha que dominou o trabalho anterior.

## 2. Aplicabilidade

**2.1 Dosímetro de ruído pessoal.** A NR-15 fixa 85 dB(A) para 8 h com taxa de troca de
5 dB. Um número sem incerteza não distingue dois casos que exigem decisões diferentes:
84 dB ± 1,5 dB (abaixo do limite, com confiança) e 88 dB ± 6 dB (pode estar acima ou abaixo;
precisa de mais medição). É essa distinção que o trabalho acrescenta.

**2.2 Anomalias em máquinas.** Um sensor acústico por máquina; a densidade do funcionamento
normal aprende-se e o que cai fora assinala-se. Uma máquina com dois microfones não se
instrumenta.

**2.3 Rede de nós urbanos.** Nós baratos que reportam nível + incerteza sem sincronismo
entre si — precisamente o que dois microfones no mesmo barramento exigem e uma rede não tem.

## 3. Hardware e o que ele mede

```
microfone   analogico na placa, PC9, alimentacao PC8, IADC 12 bits
BLE 5.3     advertising, conexoes, GATT read/write/notify — verificado com telemovel
MVP         Matrix Vector Processor: registos no PAC, driver no HAL [A MEDIR o ganho]
```

**Limite físico declarado:** 12 bits e taxa sustentada `[A MEDIR em E1-MIC.1]`. Não é um
sistema de alta fidelidade; é um sensor de energia acústica.

## 4. Método

```
1  janelas de 256 amostras
2  energia por banda por Goertzel (12 bandas entre 100 Hz e 3,8 kHz)
3  ponderacao A por banda (IEC 61672) -> nivel em dB(A)
4  features -> GMM (densidade) e regressao do nivel seguinte com variancia
5  dose NR-15 a partir do nivel, com taxa de troca de 5 dB
```

## 5. Resultados já medidos (testes que correm)

### 5.1 Dosimetria — 9 testes

```
85 dB(A) / 8 h    = 100 %          90 dB(A) / 4 h = 100 %     95 dB(A) / 2 h = 100 %
80 dB(A) / 8 h    = 50 %           metade do tempo = metade da dose
sensor a 0 dB nao "cura" a dose    piso de plausibilidade a 20 dB
```

### 5.2 Features — 6 testes, com dois controlos exigidos pelo método

```
tom puro a 1 kHz       a banda de 1 kHz domina as restantes por >= 6 dB
ruido branco           nenhuma banda domina (espalhamento < 25 dB)
controlo negativo      um tom de 250 Hz domina a banda de 250 Hz, NAO a de 1 kHz
silencio               potencia exactamente zero
ponderacao A           100 Hz fica abaixo de 1 kHz com a mesma amplitude
log10 e exp2           ida-e-volta com erro < 1e-3
```

**Porque os controlos importam:** uma feature que «dá números» não prova nada. O tom puro
prova que a banda certa acende; o ruído branco prova que não é uma banda fixa a disparar
sempre. Sem o par, não se distingue medição de artefacto.

### 5.3 Pendente, com critério já escrito

```
[A MEDIR] taxa sustentada do IADC            E1-MIC.1
[A MEDIR] GMM bate o baseline de covariancia unica em log-verosimilhanca   E1-MODEL.1
[A MEDIR] cobertura do intervalo a 90 % entre 85 % e 95 %                  E1-MODEL.2
[A MEDIR] 60 s de notificacoes BLE a 1 Hz sem quedas                       E1-BLE.1
[A MEDIR] ganho do MVP (ou fecho como "nao compensa")                      E1-MVP.1
```

## 6. Limitações (secção obrigatória, não rodapé)

1. **Valor absoluto em dB SPL não calibrado.** Sem calibrador acústico, reporta-se dBFS
   relativo com a curva A aplicada por software. Declarado como aproximação.
2. **12 bits** limitam a profundidade dinâmica face a um I2S de 24 bits.
3. **Um microfone:** sem direcção. É a limitação que motivou a reformulação, e está medida.
4. **Dados de treino limitados** — o GMM é o modelo adequado a esse tamanho; uma rede
   sobreajustaria, e isso é dito no paper.
5. **Ponderação A por banda** é uma discretização da curva contínua.

## 7. Trabalho futuro

```
- calibracao absoluta com calibrador (converte dBFS em dB SPL)
- MVP com ganho medido, ou registo de que nao compensa
- mais dados: 20+ clipes por estado acustico
- extensao a direccao se houver segundo microfone: ITD/ILD, a fisica ja' esta' escrita
```

---

# PARTE 2 — PAPER (estrutura NeurIPS/ICML)

## Título (proposto)

*Dosimetria de ruído com incerteza calibrada num microcontrolador com um único microfone*

## Abstract (a completar com os números finais)

Medimos exposição ao ruído com um microfone único integrado num MCU, e reportamos **nível
com intervalo de credibilidade** em vez de um valor pontual. O modelo de densidade acústica
(GMM sobre energias por banda) e a regressão do nível seguinte dão [COBERTURA]% de cobertura
a 90 % nominal, com nitidez de [LARGURA] dB. A dose NR-15 é calculada no dispositivo e
comunicada por BLE. Mostramos que a incerteza distingue casos que o valor pontual confunde:
84 ± 1,5 dB e 88 ± 6 dB implicam decisões diferentes sobre exposição.

## 1. Introdução

- Exposição ao ruído é uma decisão com incerteza (NR-15: 85 dB(A)/8 h, troca de 5 dB).
- Dosímetros comerciais dão um número sem incerteza.
- Contribuição: (a) densidade acústica + regressão com incerteza num MCU de 78 MHz com um
  microfone; (b) calibração empírica da incerteza por cobertura; (c) dose normativa no
  dispositivo, sem nuvem.

## 2. Fundamentação teórica

- Densidade: GMM diagonal; porque não uma rede (tamanho dos dados).
- Regressão: linear bayesiana / GP; intervalo preditivo e o que significa «calibrado».
- Ponderação A (IEC 61672) e por que a dose é em dB(A).
- Física do sensor: pressão → diafragma → ADC; limite de Nyquist; um ponto não mede direção.

## 3. Metodologia

- Hardware e taxa (medida).
- Extração de features: Goertzel por banda, 12 bandas, controlos.
- Treino no host, coeficientes em `const`, inferência no MCU.
- Dose: fórmula e verificação numérica.

## 4. Experimentos

### 4.1 Controlos das features (feito)

Tabela com os 6 testes: tom puro, ruído branco, controlo negativo, silêncio, ponderação A,
precisão numérica.

### 4.2 Densidade (a medir)

Log-verosimilhança por janela: GMM vs covariância única, em dados não vistos. Controlo:
janelas do mesmo estado acústico com verosimilhança maior em ≥ 90 % dos casos.

### 4.3 Incerteza (a medir)

Cobertura observada vs nominal (85–95 % para 90 %), nitidez, NLL. Controlo de horizonte:
0 → erro < 1 dB; 20 janelas → erro maior **e** intervalo mais largo.

### 4.4 Dose (feito)

Verificação numérica (85/8h, 90/4h, 95/2h = 100 %) e teste de robustez a leituras inválidas.

### 4.5 Otimização bayesiana (a medir)

q do PPCA, componentes do GMM, largura do kernel — curva de convergência.

## 5. Resultados e discussão

- O que a incerteza acrescenta à decisão (tabela de casos).
- Porque as perdas em rajada são piores que aleatórias (resultado do projecto anterior,
  medido: 25,08 dB vs 14,68 dB a 5 %) — relevante para firmware de áudio.
- Onde o modelo não ganha: em ruído de banda larga o PPCA não bate a interpolação linear
  (medido no projecto anterior: 16,74 vs 25,08 dB). Resultado negativo declarado.

## 6. Conclusão

Um microfone, com incerteza honesta, vale mais do que dois mal ligados — e a incerteza é
medível, não decorativa.

## Apêndices

- A. Verificação da fórmula da dose.
- B. Controlos das features (com os sinais usados).
- C. Protocolo de calibração absoluta (trabalho futuro).
