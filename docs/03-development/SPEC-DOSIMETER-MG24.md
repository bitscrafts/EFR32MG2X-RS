# SPEC — dosimeter-mg24: dosímetro de ruído com incerteza

**Identificador:** `E1-APP.DOSIMETER` · **Data:** 2026-11-17 · **Estado:** especificado, por implementar
**Projecto:** CK0475 — Aprendizagem de Máquina Probabilística (UFC 2026.2)
**Hardware:** XIAO MG24 Sense (EFR32MG24B220F1536IM48-B) — microfone analógico na placa, BLE 5.3, MVP

---

## 1. Objectivo

Um dispositivo autónomo que mede exposição ao ruído e comunica por BLE a um telemóvel
**o nível com intervalo de credibilidade**, não um número seco.

```
o que mede       nivel de pressao sonora equivalente em janelas curtas (Leq) e a dose acumulada
o que acrescenta a incerteza de cada leitura, medida por calibracao (cobertura e nitidez)
o que corre      no MCU: extracao de features + densidade GMM + regressao do nivel
o que mostra     telemovel, por Web Bluetooth (sem instalar app): nivel, intervalo, historico, dose
```

---

## 2. Aplicabilidade (entra na apresentação, slides 2 e 10)

### 2.1 Dosímetro de ruído pessoal

No Brasil, a **NR-15** fixa 85 dB(A) para 8 h com taxa de troca de 5 dB: cada +5 dB reduz
para metade o tempo admissível. Dosímetros comerciais dão um número **sem incerteza**.
Dois casos diferentes exigem decisões diferentes:

```
84 dB +-1,5 dB   -> abaixo do limite com confianca
88 dB +-6 dB     -> pode estar acima ou abaixo: precisa de medicao mais longa antes de decidir
```

**Só um sistema probabilístico distingue estes dois casos.** E um dosímetro é, por
natureza, um dispositivo de **um** microfone: dois exigiriam espaço no colete e alinhamento
que ninguém mantém.

### 2.2 Anomalias em máquinas, um sensor por máquina

A densidade da paisagem acústica normal aprende-se com GMM sobre bandas. O que cai fora
(rolamento, correia, cavitação) assinala-se. **Uma máquina com dois microfones não se
instrumenta; com um, sim.**

### 2.3 Rede de nós de ruído urbano

Cada nó é barato, autónomo, reporta **nível + incerteza** quando um telemóvel passa. A
agregação não precisa de sincronismo entre nós — que é exactamente o que dois microfones no
mesmo barramento exigem e uma rede de nós não tem.

### 2.4 Porque este hardware, e não o par I2S

```
sem fiacao externa        o microfone esta' na placa: nao ha' modulo, solda nem barramento
BLE verificado            advertising, conexoes e GATT notify confirmados com telemovel
acelerador de ML          MVP (Matrix Vector Processor) no EFR32MG24
precedente publico        detecao de ronco no mesmo hardware com o microfone da placa
```

**Risco de montagem: zero.** O projecto anterior falhou por um módulo I2S avariado que
amarrou o barramento partilhado ao terra — esse modo de falha não existe aqui.

---

## 3. Estado actual verificado neste repositório (2026-11-17)

```
EXISTE E FUNCIONA (verificado em hardware ou por exemplo compilado)
  crates/efr32mg2x-hal/src/microphone.rs     Microphone::sample() -> Option<u16>
                                             Microphone::level(window) -> Level
                                             power-up por PC8, sinal em PC9, IADC 12 bits
  crates/xiao-mg24-bsp/examples/module_mic_level.rs
                                             LED segue o nivel; uma palma levanta RMS e pico
  BLE 5.3                                    advertising, scanning, conexoes,
                                             GATT read/write/notify — confirmado de telemovel
  BSP Module                                 Module::microphone(), Module::led(), etc.
  PAC                                        registos MVP_NS e MVP_S presentes

NAO EXISTE
  captura continua com taxa fixa            o driver faz conversoes SINGLE, uma a uma
  features por banda                        nao ha' extracao de bandas nem log-mel
  inferencia de modelo                      nao ha' GMM, regressao nem calibracao
  servico GATT do dosimetro                nao ha' servico de nivel/intervalo/dose
  driver do MVP                             registos existem no PAC, driver no HAL nao
  aplicacao dosimeter-mg24                  nao existe
```

**Consequência:** a única incerteza que pode mudar o plano é a **taxa de amostragem
sustentada** do IADC neste driver. Mede-se na tarefa `E1-MIC.1` antes de tudo o resto.

---

## 4. Arquitectura

```
microfone analogico (PC9)
   |
   v
IADC 12 bits  --(taxa a medir: alvo >= 4 kHz)-->  anel de amostras
   |
   v
remocao de offset DC  ->  janelas de 256 amostras (16 ms a 16 kHz, 64 ms a 4 kHz)
   |
   v
bandas de energia (8 a 16 bandas, Goertzel ou FFT de 256)  ->  vector de features
   |
   +--> GMM de densidade  ->  log-verosimilhanca (estado acustico, anomalia)
   |
   +--> regressao do nivel ->  nivel previsto + variancia
   |
   v
Leq por janela longa (1 s) + dose acumulada (NR-15)
   |
   v
BLE GATT notify  ->  telemovel (Web Bluetooth) mostra nivel, intervalo, dose, historico
```

**MVP:** acelera as multiplicacoes do GMM (responsabilidade `E1-MVP.1`). **A aplicação tem
de correr sem ele**, com o caminho em CPU como referência — o MVP é optimização, não
requisito de correcção.

---

## 5. Trabalho a implementar, por tarefa

Cada tarefa tem identificador, entrada, saída e **critério de aceitação medido**. A ordem
é a de dependência: as primeiras não dependem de nada e desbloqueiam o resto.

### E1-MIC.1 — Medir a taxa sustentada do IADC

```
entrada    driver actual (Microphone::sample)
saida      numero medido: amostras por segundo em conversao continua
metodo     contar amostras durante 10 s com SysTick; imprimir por USART; repetir 3x
aceitacao  taxa medida com desvio < 5 % entre as 3 medicoes, e registada no relatorio
           decide: >= 8 kHz -> features espectrais completas
                   2-8 kHz  -> bandas largas e envelope (plano B, ver secao 8)
```

### E1-MIC.2 — Captura continua com anel

```
entrada    IADC0 + LDMA (crates/efr32mg2x-hal/src/dma)
saida      stream_dma(buf, n) preenche um anel circular sem intervencao da CPU
aceitacao  10 s de captura com ZERO amostras perdidas (contador de overruns = 0)
           e leitura estavel do RMS em 3 ensaios consecutivos
```

### E1-MIC.3 — Remoção de offset e escala

```
entrada    amostras de 12 bits (0..4095, referencia 1,2 V)
saida      amostra centrada em zero (i16) e escala em dBFS relativa
aceitacao  em silencio absoluto (sala sem ventilador) o valor medio fica < 2 LSB;
           uma palma a 30 cm produz pico > 20x o RMS de repouso
```

### E1-FEAT.1 — Bandas de energia

```
entrada    janela de 256 amostras
saida      8 bandas (plano B) ou 16-24 bandas tipo mel, em escala log
aceitacao  controle: tom puro a 1 kHz -> a banda correspondente domina por >= 15 dB
           controle: ruido branco -> energia repartida, nenhuma banda domina mais de 6 dB
```

### E1-MODEL.1 — Densidade GMM

```
entrada    features de N janelas (dados reais gravados pelo dispositivo)
saida      coeficientes (pesos, medias, covariancias diagonais) em modelo.rs, const
aceitacao  log-verosimilhanca por janela em dados NAO vistos >= a de um modelo de
           covariancia unica (o GMM tem de ganhar ao baseline, senao nao se usa)
           controle: janelas do mesmo estado (ventilador) tem verosimilhanca maior
           que janelas de outro estado (palmas) em >= 90 % dos casos
```

### E1-MODEL.2 — Regressão do nível com incerteza

```
entrada    nivel das K janelas anteriores
saida      nivel previsto + variancia (regressao bayesiana ou GP aproximado)
aceitacao  cobertura do intervalo a 90 %: entre 85 % e 95 % em dados de teste
           nitidez: largura media do intervalo < 6 dB
           controle horizonte 0 -> erro < 1 dB; horizonte 20 janelas -> erro maior E
           intervalo mais largo (se nao alargar, o modelo mente confianca)
```

### E1-DOSE.1 — Dosimetria NR-15

```
entrada    Leq por segundo
saida      dose acumulada (%), tempo admissivel restante
aceitacao  controle numerico: 85 dB(A) durante 8 h -> dose 100 %;
           90 dB(A) durante 4 h -> dose 100 % (taxa de troca 5 dB);
           teste unitario na biblioteca, sem hardware
```

### E1-BLE.1 — Serviço GATT do dosímetro

```
entrada    XiaoBle (GATT server ja' funcional)
saida      servico com 4 caracteristicas:
             nivel_leq      f32 em dB(A)            notify 1 Hz
             intervalo      (min, max) em dB        notify 1 Hz
             dose           f32 em %                notify 0,1 Hz
             comando        write: reset da dose, periodo de notificacao
aceitacao  telemovel (nRF Connect) le as 3 notificacoes a 1 Hz durante 60 s sem quedas;
           o write de reset repoe a dose a 0 e a notificacao seguinte confirma
```

### E1-BLE.2 — Ligação telemóvel (Web Bluetooth)

```
entrada    pagina HTML unica servida localmente (ou ficheiro aberto no browser)
saida      nivel, intervalo e dose em tempo real, mais um grafico das ultimas 5 min
aceitacao  abrir no telemovel, emparelhar, ver valores a actualizar; palmas levantam o
           nivel em menos de 1 s
```

### E1-MVP.1 — Aceleração do GMM pelo MVP (opcional)

```
entrada    registos MVP_NS/MVP_S do PAC (sem driver no HAL)
saida      multiplicacao matriz-vetor acelerada para a inferencia do GMM
aceitacao  (a) resultado identico ao caminho CPU (|delta log-ver| < 1e-3);
           (b) tempo de inferencia medido e comparado; se nao houver ganho >= 2x,
           a tarefa fecha como "nao compensa" e o CPU fica como caminho oficial
NOTA       o MVP e' optimizacao. Nenhuma outra tarefa pode depender dele.
```

### E1-APP.1 — Aplicação dosimeter-mg24

```
entrada    todas as tarefas acima
saida      /projects/dosimeter-mg24 (projecto proprio) + a app que corre na placa
aceitacao  ligar a placa, abrir a pagina no telemovel, fazer 3 palmas e ver o nivel e o
           intervalo a reagir; desligar o telemovel e a dose continua a acumular no
           dispositivo (autonomia local)
```

### E1-HOST.1 — Treino e geração de coeficientes

```
entrada    gravações reais do dispositivo (USART ou BLE), 10-20 clipes de 10 s
saida      python/treina_gmm.py -> modelo.rs com const arrays; python/calibra_gp.py
aceitacao  o ficheiro gerado compila no workspace; inferencia em Rust reproduz a
           verosimilhanca calculada em Python com |delta| < 1e-3 (teste de ida-e-volta)
```

---

## 6. Ordem de execução e dependências

```
1  E1-MIC.1   medir taxa            (bloqueia a decisao de features)      30 min
2  E1-MIC.2   captura continua      (depende de 1)                       2-3 h
3  E1-MIC.3   offset e escala       (depende de 2)                       1 h
4  E1-FEAT.1  bandas                (depende de 3)                       2 h
5  E1-HOST.1  treino + modelo.rs    (depende de 4 e de gravacoes)        3 h
6  E1-MODEL.1 GMM no device         (depende de 5)                       2 h
7  E1-MODEL.2 nivel + incerteza     (depende de 5)                       3 h
8  E1-DOSE.1  dosimetria            (independente, teste unitario)       1 h
9  E1-BLE.1   GATT do dosimetro     (independente)                       2 h
10 E1-BLE.2   pagina telemovel      (depende de 9)                       2 h
11 E1-APP.1   aplicacao             (depende de 6,7,8,9)                 2 h
12 E1-MVP.1   MVP                   (opcional, no fim)                   4 h
```

---

## 7. Riscos e plano B

```
R1 taxa do IADC < 2 kHz
   plano B: features de envelope (RMS e maximo por janela de 64 ms) e bandas muito largas
   (2 a 4), mantendo P1 e P2 intactos — a densidade passa a ser sobre envelope, nao espectro.
   O projecto continua valido; muda a resolucao em frequencia, nao o metodo.

R2 LDMA com IADC nao fechar (scan precisa de trigger)
   plano B: captura por interrupcao do IADC com buffer duplo; perde-se taxa maxima mas
   garante-se continuidade. Medir antes de decidir (tarefa E1-MIC.2).

R3 modelo nao cabe ou nao corre a 1 Hz
   plano B: GMM diagonal com 8 componentes x 8 bandas = 64 medias + 64 variancias;
   inferencia O(K*D) = 512 multiplicacoes por janela — trivial para 78 MHz.

R4 calibracao da cobertura falha (intervalos mal calibrados)
   plano B e' metodologico: reportar a falha e recalibrar por escala de variancia
   (temperatura), que e' uma correcao padrao e mensuravel.
```

---

## 8. Critérios globais de aceitação (o que o relatório tem de mostrar)

```
1  taxa de amostragem medida, com desvio entre ensaios
2  features validadas por dois controles (tom puro e ruido branco)
3  GMM bate o baseline de covariancia unica em log-verosimilhanca
4  cobertura do intervalo a 90 % entre 85 e 95 %, com nitidez < 6 dB
5  dose conferida contra calculo a mao (85 dB/8 h e 90 dB/4 h)
6  BLE: 60 s de notificacoes a 1 Hz sem quedas
7  demo no telemovel a reagir a palmas em < 1 s
8  (opcional) MVP igual ao CPU e ganho medido, ou fecho como "nao compensa"
```

---

## 9. Apresentação — onde entram os tópicos de aplicabilidade

```
slide 1   problema: exposicao ao ruido e' uma decisao com incerteza (NR-15)
slide 2   APLICABILIDADE 1: dosimetro pessoal — 84+-1,5 dB vs 88+-6 dB sao decisoes diferentes
slide 3   APLICABILIDADE 2: anomalias em maquinas com um sensor por maquina
slide 4   APLICABILIDADE 3: rede de nos urbanos sem sincronismo
slide 5   hardware: microfone na placa, sem fiacao; BLE verificado
slide 6   features e densidade (P1), com os dois controles
slide 7   nivel com incerteza (P2), cobertura e nitidez
slide 8   dose e demonstracao no telemovel
slide 9   otimizacao bayesiana dos hiperparametros (P3)
slide 10  limitacoes e trabalho futuro: MVP, mais dados, calibracao absoluta em dB SPL
```

**Frase de fecho prevista:** *um número sem incerteza não distingue «está abaixo do limite»
de «não sei se está»; e é essa diferença que faz um dosímetro ser útil.*

---

## 10. O que este documento NÃO decide

```
- o valor absoluto em dB SPL: requer calibrador acustico. Reporta-se dBFS relativo e
  declara-se a limitacao. A conversao para dB(A) e' feita com uma curva de ponderacao
  aplicada por software, documentada como aproximacao.
- a taxa final de amostragem: depende da medicao E1-MIC.1
- se o MVP compensa: depende de E1-MVP.1
```
