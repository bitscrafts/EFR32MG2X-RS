#!/usr/bin/env bash
# Um item da fila por invocação. Despacha o worker, corre o gate, commita só se verde.
#
# Porque é que isto existe, e porque é que é um script:
#   um despacho é a acção mais cara do projecto; tem de ser revísivel antes de correr e
#   re-executável depois de falhar. Um comando inline existe só no histórico do terminal,
#   e um shell em background não herda o ambiente (PATH, cargo, credenciais) -- por isso
#   tudo o que é preciso é fixado aqui dentro.
#
# Saídas:  0 item feito e commitado | 1 gate vermelho (item fica na fila) | 2 erro de ambiente
#          3 fila vazia | 5 três vermelhos seguidos, fila parada (BLOCKED)
set -uo pipefail

cd "$(dirname "$0")/.." || exit 2
REPO=$(pwd)

QUEUE="$REPO/docs/05-operations/QUEUE.md"
LOG="$REPO/docs/05-operations/QUEUE-RUN-LOG.md"
ESTADO="$REPO/_tmp/queue/state.json"
GATE_LOG="$REPO/_tmp/queue/gate.log"
WORKER_LOG="$REPO/_tmp/queue/worker.log"
# O modelo NAO tem valor por omissao. Um default foi o que escolheu o Gemini e gastou
# a quota antes de o worker sequer arrancar. Quem chama decide, e diz aqui.
MODELO="${QUEUE_MODEL:-}"
TECTO_TURNS="${QUEUE_MAX_TURNS:-60}"
MAX_VERMELHOS=3

mkdir -p "$REPO/_tmp/queue"
export PATH="/usr/local/cargo/bin:$HOME/.cargo/bin:$HOME/.local/bin:$PATH"


# O relatorio sai daqui, do que o gate provou -- nao de um agente a ler ficheiros.
# E por isso que este script pode correr com no_agent: nenhuma quota de modelo no caminho.
relatorio() {
    local id="$1" veredicto="$2" sha="$3" estado="$4"
    printf 'ITEM      %s -- %s\n' "$id" "$OBJECTIVO"
    printf 'GATE      %s\n' "$veredicto"
    if [ -n "$sha" ]; then
        printf 'COMMIT    %s\n' "$sha"
        printf 'FICHEIROS\n'
        git show --stat --format='' "$sha" | sed 's/^/          /' | head -12
    else
        printf 'COMMIT    nenhum -- o item continua na fila\n'
        grep -E '^  FAIL' "$GATE_LOG" | head -3 | sed 's/^ */FALHAS    /'
    fi
    printf 'WORKER    rc=%s, %ss -- %s\n' "$rc_worker" "$segundos" "${ultima_linha:-}"
    printf 'FILA      %s itens por fazer\n' "$(grep -c '^- \[ \] ' "$QUEUE" || echo 0)"
}

agora() { date -u +%Y-%m-%dT%H:%M:%SZ; }

nota() {
    printf '%s\n' "$*" >> "$LOG"
}

# --- a primeira linha nao marcada e o item da vez -----------------------------------------
linha=$(grep -n '^- \[ \] ' "$QUEUE" | head -1 || true)
if [ -z "$linha" ]; then
    nota "- $(agora) **QUEUE EMPTY** — nada por fazer; é hora de planear mais itens."
    echo "QUEUE EMPTY"
    exit 3
fi
numero_linha=${linha%%:*}
corpo=${linha#*:}
ID=$(printf '%s' "$corpo" | sed 's/^- \[ \] //' | cut -d'|' -f1 | tr -d ' ')
SPEC=$(printf '%s' "$corpo" | cut -d'|' -f2 | sed 's/^ *//;s/ *$//')
OBJECTIVO=$(printf '%s' "$corpo" | cut -d'|' -f3- | sed 's/^ *//')

if [ ! -f "$REPO/$SPEC" ]; then
    echo "ERRO: a spec do item $ID nao existe: $SPEC"
    nota "- $(agora) **$ID** sem spec ($SPEC) — fila parada, nada foi feito."
    exit 2
fi

if [ -z "$MODELO" ]; then
    echo "ERRO: QUEUE_MODEL nao definido. Passa o modelo na linha de comando:"
    echo "  QUEUE_MODEL=<provider>:<model> bash scripts/queue-run-once.sh"
    echo "  exemplos: nvidia:deepseek-ai/deepseek-v4-flash-0731  |  deepseek:deepseek-chat"
    echo "  proibido: qualquer rota ollama-cloud, e o gemini (quota esgotada, 429)."
    exit 2
fi

if [ "${1:-}" = "--dry-run" ]; then
    echo "item    : $ID"
    echo "spec    : $SPEC"
    echo "modelo  : ${MODELO:-(predefinicao do pi)}"
    echo "objectivo: $OBJECTIVO"
    exit 0
fi

echo "== $ID =="
nota ""
nota "## $(agora) — $ID"
nota "$OBJECTIVO"

# --- despacho ---------------------------------------------------------------------------
brief="$REPO/_tmp/queue/$ID-brief.txt"
{
    echo "Implementa esta spec neste repositório. Trabalha só nos ficheiros que o bloco Files dela nomeia."
    echo "Nao facas commit, nao publiques, nao flashes. Deixa o trabalho na arvore."
    echo "Quando acabares, escreve um paragrafo com o que fizeste e o que ficou por fazer."
    echo
    cat "$REPO/$SPEC"
} > "$brief"

# A sessao e retomada, nunca recriada: o contexto do trabalho (o que ja se leu, o que se
# decidiu, os erros que ja se pagaram) vive nela, e um item novo nao pode comecar do zero.
# `--session <ficheiro>` continua a sessao cujo .jsonl e este; e o mesmo ficheiro entre itens.
SESSAO="$REPO/_tmp/queue/pi-session.jsonl"
pi_args=(-p --yolo --session "$SESSAO")
[ -n "$MODELO" ] && pi_args+=(--model "$MODELO")
[ -n "${QUEUE_PROVIDER:-}" ] && pi_args+=(--provider "$QUEUE_PROVIDER")

inicio=$(date +%s)
timeout 3000 pi "${pi_args[@]}" "$(cat "$brief")" > "$WORKER_LOG" 2>&1
rc_worker=$?
ultima_linha=$(tail -3 "$WORKER_LOG" | tr "\n" " " | cut -c1-160)
segundos=$(( $(date +%s) - inicio ))
nota "- worker: rc=$rc_worker, ${segundos}s, log \`_tmp/queue/worker.log\`"

# --- o gate e a unica opiniao que conta --------------------------------------------------
bash "$REPO/scripts/gate.sh" > "$GATE_LOG" 2>&1
rc_gate=$?
veredicto=$(grep -E '^VERDICT' "$GATE_LOG" | tail -1)

if [ $rc_gate -ne 0 ]; then
    nota "- gate: ${veredicto:-sem veredicto} (rc=$rc_gate) — **nada foi commitado**, o item fica na fila"
    nota "- primeiras falhas:"
    grep -E '^  FAIL' "$GATE_LOG" | head -3 | sed 's/^/    - /' >> "$LOG"

    vermelhos=0
    [ -f "$ESTADO" ] && vermelhos=$(python3 -c "import json;print(json.load(open('$ESTADO')).get('vermelhos',0))" 2>/dev/null || echo 0)
    vermelhos=$((vermelhos + 1))
    python3 - "$ESTADO" "$vermelhos" <<'PY'
import json, sys
open(sys.argv[1], "w").write(json.dumps({"vermelhos": int(sys.argv[2])}))
PY
    if [ "$vermelhos" -ge "$MAX_VERMELHOS" ]; then
        nota "- **BLOCKED**: $vermelhos vermelhos seguidos. A fila para aqui e nao volta a tentar sozinha."
        echo "BLOCKED: $vermelhos vermelhos seguidos"
        exit 5
    fi
    relatorio "$ID" "${veredicto:-sem veredicto}" "" vermelho
    exit 1
fi

# --- verde: marca, commita, conta -------------------------------------------------------
python3 - "$QUEUE" "$numero_linha" <<'PY'
import sys, pathlib
f, n = pathlib.Path(sys.argv[1]), int(sys.argv[2])
linhas = f.read_text().splitlines()
linhas[n-1] = linhas[n-1].replace("- [ ] ", "- [x] ", 1)
f.write_text("\n".join(linhas) + "\n")
PY

# O hook de commit recusa uma arvore que o marcador nao certificou. O gate correu agora,
# mas o marcador e do commit anterior: refaz-se aqui, sobre o que vai ser commitado.
HOOK="$HOME/.config/git/hooks/gate-pass.sh"
if [ -x "$HOOK" ]; then
    bash "$HOOK" >/dev/null 2>&1 || { echo "ERRO: gate-pass recusou a arvore"; exit 2; }
fi

git add -A
if git diff --cached --quiet; then
    nota "- gate: ${veredicto:-PASS} — mas nada mudou na arvore; item marcado sem commit"
else
    git -c user.name="Marcelo Correa" -c user.email="mvcorrea@gmail.com" \
        commit -q -m "$ID: $OBJECTIVO" || { echo "ERRO: commit falhou"; exit 2; }
    sha=$(git rev-parse --short HEAD)
    nota "- gate: ${veredicto:-PASS} — commit \`$sha\`"
fi

python3 - "$ESTADO" <<'PY'
import json, sys
open(sys.argv[1], "w").write(json.dumps({"vermelhos": 0}))
PY

relatorio "$ID" "${veredicto:-PASS}" "$(git rev-parse --short HEAD)" feito
restantes=$(grep -c '^- \[ \] ' "$QUEUE" || true)
if [ "$restantes" -eq 0 ]; then
    echo
    echo "FILA VAZIA -- nada mais a fazer."
    echo "ITENS FECHADOS"
    grep '^- \[x\] ' "$QUEUE" | sed 's/^- \[x\] /          /'
    echo "BACKLOG (precisa de spec para entrar na fila)"
    grep '^- \[~\] ' "$QUEUE" | sed 's/^- \[~\] /          /'
fi
exit 0
