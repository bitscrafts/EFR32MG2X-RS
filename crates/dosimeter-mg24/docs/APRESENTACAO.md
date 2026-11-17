# Apresentação — guião de 10 minutos

**dosimeter-mg24 · CK0475** · aplicabilidade primeiro, modelo a seguir, hardware como instrumento

> Regra: cada slide diz **uma** coisa. O texto abaixo é o que se diz, não o que se lê no ecrã.
> Onde diz `[NÚMERO]`, entra o valor medido — não se inventa.

---

## Slide 1 — A pergunta (45 s)

*Título:* Um número de ruído sem incerteza não é uma resposta.

*Diz-se:* «A NR-15 diz: 85 dB(A) durante 8 horas. Um dosímetro comercial devolve um número.
Mas há duas situações que exigem decisões diferentes: 84 dB com ±1,5 dB — está abaixo do
limite, com confiança; e 88 dB com ±6 dB — pode estar acima ou abaixo, e nesse caso o que
falta não é o número, é **medição**. Este trabalho faz um dosímetro que devolve o número
**com** a incerteza.»

## Slide 2 — Aplicabilidade 1: dosímetro pessoal (45 s)

*Diz-se:* «O caso principal. Um dispositivo no bolso ou no colete, microfone na própria
placa, telemóvel a mostrar nível, intervalo e dose acumulada. Um dosímetro é, por natureza,
um dispositivo de **um** microfone: dois exigiriam espaço no colete e um alinhamento que
ninguém mantém.»

## Slide 3 — Aplicabilidade 2: máquinas e rede urbana (45 s)

*Diz-se:* «O mesmo modelo serve para anomalias em máquinas — um sensor por máquina, porque
uma máquina com dois microfones não se instrumenta. E serve para nós de ruído urbano: cada
nó reporta nível **e** incerteza sem precisar de sincronismo com os outros, que é
exactamente o que dois microfones no mesmo barramento exigem e uma rede não tem.»

## Slide 4 — Porque este hardware (45 s)

*Título:* O sensor está na placa.

*Diz-se:* «A versão anterior usava dois módulos I2S. Um avariou e, por partilharem a linha de
dados, apagou os dois canais: `picoL = 0` e `picoR = 0`. Com o módulo bom sozinho: 17 000 a
18 000. Foi medido, não foi suposto. Aqui não há fiação: o microfone está na placa, e o BLE
já foi verificado com telemóvel.»

## Slide 5 — O que o sensor mede, e o que não mede (45 s)

*Diz-se:* «Mede pressão sonora num ponto: 12 bits, taxa medida `[TAXA]`. **Não mede
direção**: a direção vive na diferença de chegada entre dois pontos, `τ = d·sin θ / c`. Com
um ponto não há τ. Esta é a fronteira física do trabalho, e está declarada na primeira
página.»

## Slide 6 — P1: a paisagem acústica é uma densidade (90 s)

*Diz-se:* «12 bandas por Goertzel entre 100 Hz e 3,8 kHz, com ponderação A. Um GMM aprende a
densidade dos estados acústicos. Dois controlos provam que isto mede o que diz medir: um tom
puro a 1 kHz domina a banda de 1 kHz por 6 dB ou mais; ruído branco não tem banda dominante.
Sem os dois, uma feature que dá números não prova nada.»

## Slide 7 — P2: o nível seguinte, com incerteza (90 s)

*Diz-se:* «Regressão bayesiana do nível do instante seguinte, com variância preditiva.
A parte que quase ninguém mede: **calibração**. A cobertura a 90 % nominal tem de dar entre
85 % e 95 % na realidade. E há um controlo que denuncia modelos mentirosos: com horizonte
zero, o erro tem de ser quase nulo; com horizonte longo, o erro cresce **e o intervalo tem
de abrir**. Se não abrir, o modelo está a fingir confiança.»

## Slide 8 — Dose, verificada contra a norma (45 s)

*Diz-se:* «A dose usa a taxa de troca de 5 dB: cada +5 dB permite metade do tempo.
Verificação numérica, que corre em testes: 85 dB/8 h = 100 %, 90 dB/4 h = 100 %,
95 dB/2 h = 100 %, 80 dB/8 h = 50 %. E um detalhe que evita um erro real: um sensor
desligado lê 0 dB e não pode «curar» a dose já acumulada.»

## Slide 9 — Demonstração (60 s)

*Diz-se:* «Telemóvel no browser, sem instalar nada, ligado por Web Bluetooth: nível,
intervalo e dose. Três palmas levantam o nível em menos de um segundo; o ventilador da sala
mantém o nível estável e o intervalo estreito. E a dose acumula no dispositivo mesmo com o
telemóvel desligado.»

*Plano B, se o BLE falhar ao vivo:* série com os mesmos valores, gravada minutos antes.

## Slide 10 — Limitações e fecho (45 s)

*Diz-se:* «Três limitações, todas declaradas: sem calibrador acústico o valor é relativo, em
dBFS com curva A; 12 bits limitam a profundidade; e um microfone não dá direção — essa
extensão está escrita, com a física pronta, se houver segundo sensor. Fecho: um ponto de
medida com incerteza honesta vale mais do que dois mal ligados.»

---

# Perguntas prováveis, com resposta pronta

```
"Porque nao usaram dois microfones?"
    Estava montado. Um modulo avariou e, por a linha de dados ser partilhada, apagou os dois
    canais — esta' medido: com o bom, 17-18k; com o avariado, zero nos dois.
    A direcao exige dois pontos (tau = d sin/c); com um ponto nao ha' tau.

"O GMM nao e' fraco para audio?"
    Como classificador de fala, sim. Aqui e' um MODELO DE DENSIDADE sobre 12 bandas de uma
    paisagem acustica, comparado com covariancia unica. Com os dados que temos, uma rede
    sobreajustaria — e isso e' uma limitacao declarada, nao uma escolha estetica.

"Os intervalos sao honestos?"
    Foi o que medimos por cobertura: [COBERTURA] % observado contra 90 % nominal. Se nao
    coincidisse, o defeito seria do modelo e diriamos isso.

"Porque nao corre tudo no telemovel?"
    A dose tem de acumular sem telemovel: o dispositivo e' o instrumento, o telemovel e' o
    ecra. Alem disso a inferencia no MCU mostra que o modelo cabe em KB.

"O que isto tem de machine learning probabilistico?"
    P1: densidade (GMM) sobre features reais. P2: regressao com distribuicao preditiva e
    calibracao. P3: otimizacao bayesiana dos hiperparametros. Os tres pontos aparecem.

"E se o microfone avariar, como aconteceu ao modulo?"
    Uma leitura de 0 dB nao pode reduzir a dose; ha' um piso de plausibilidade a 20 dB, com
    teste. Uma avaria aparece como ausencia de leitura, nao como silencio.
```

---

# Ordem de construção dos slides (para não deixar para o fim)

```
1  slides 1-5    ja' tem conteudo (fisica, aplicabilidade, hardware medido)
2  slide 6       depende dos controlos das features  — JA' FEITO (6 testes)
3  slide 7       depende da cobertura medida         — falta o modelo
4  slide 8       JA' FEITO (9 testes da dose)
5  slide 9       depende do BLE e da pagina          — janela de infra
6  slide 10      escreve-se por ultimo, com as limitacoes reais
```
