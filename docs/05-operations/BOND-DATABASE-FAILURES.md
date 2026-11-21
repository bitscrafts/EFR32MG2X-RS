# A base de bonds — o estado que tranca o aparelho

Duas ocorrências, duas sessões perdidas. É o único estado do aparelho que sobrevive a um
reset e que o deixa num sítio de onde ele não sai sozinho.

## O sintoma, igual nas duas vezes

```
a placa esta' alimentada, o LED diz que esta' a anunciar
a app nao encontra nada
um scanner BLE tambem nao encontra nada
nao ha' erro em lado nenhum -- nem no telefone, nem no LED, nem no firmware
```

É o pior tipo de falha: **o aparelho parece saudável.** O LED vermelho do XIAO significa
"pronto a emparelhar", portanto a indicação que temos diz o contrário do que se passa.

## O que foi medido, e o que não foi

**2026-11-17** — a placa deixou de arrancar depois de uma sequência de tentativas de
emparelhamento. `RAILCb_AssertFailed`. O `module_nvm3_wipe` nasceu deste incidente.

**2026-11-21, na apresentação** — a placa anunciava (LED aceso), a app não ligava, o Mac
também não a via. Recuperada no lnx com `scripts/recover-bonds.sh`: wipe do NVM3 e reflash.
Ligou à primeira.

**O que NÃO foi isolado:** a causa. O wipe, o reflash e o reset foram feitos **juntos** —
três variáveis de uma vez. Qualquer uma delas pode ter sido a que resolveu:

| hipótese | como se distinguiria |
|---|---|
| base de bonds inconsistente | apagar só o NVM3 e reiniciar, sem reflashar |
| firmware diferente do que corria | reflashar sem apagar o NVM3 |
| stack preso em RAM | um reset simples, sem tocar em nada |
| bond órfão do lado do telefone | esquecer o dispositivo no telefone apenas |

**Nenhuma destas foi feita.** O registo fica com a hipótese principal identificada e com a
confissão de que não foi provada. Não repetir a frase "era a base de bonds" como se fosse
medição.

## O que mudou por causa disto (2026-11-21)

O firmware apaga os bonds **em cada arranque**, antes de anunciar. Com
`SECURITY_REQUIRED = false` a ligação não é encriptada, portanto os bonds não davam nada em
troca — eram estado a arder sem contrapartida, e o único sítio onde o aparelho se podia
trancar.

O efeito é o que interessa ao produto: **trocar ou reinstalar a app deixou de poder partir a
ligação**, e nenhum arranque herda estado de uma sessão anterior. Confirmado pelo operador:
a app liga depois de reinstalada, e de outro computador.

O que isto **não** garante: um stack que falhe no arranque por outra razão (o caso de
2026-11-17) continua a precisar do wipe. É por isso que a recuperação ficou em script
versionado, e não na cabeça de ninguém.

## Como se recupera

```bash
bash scripts/recover-bonds.sh              # apaga bonds + repoe o firmware
bash scripts/recover-bonds.sh --firmware   # so' repoe o firmware, bonds intactos
```

O `--firmware` existe para poder fazer o que não foi feito em 21/11: mudar **uma** variável
e voltar a tentar.

## O que fica por fazer

- **A app não diz porque não encontrou nada.** Não distingue "vi zero dispositivos" de "vi
  três e nenhum tinha o serviço". Hoje isso foi o que nos deixou sem diagnóstico.
- **Não há caminho na placa para esquecer bonds sem portátil.** Uma característica GATT, ou
  um toque longo, transformaria uma recuperação de meia hora num gesto de cinco segundos.