#!/usr/bin/env bash
# Exercita `queue-run-once.sh` num repositório descartável, com um worker que não é o pi.
#
# ## Porque é que isto existe
#
# Dois itens ficaram "feitos" sem trabalho nenhum em 2026-11-18: o script marcava o item antes de
# commitar, e aceitava um gate PASS sobre uma árvore inalterada como se fosse trabalho. Uma regra
# dessas não se corrige por leitura -- corrige-se exercendo o script, que é o que este ficheiro faz.
#
# ## Os quatro casos, e o que cada um apanha
#
#   1) worker que não produz nada   -> item NÃO marcado, saída 4 (e não 0)
#   2) worker que produz trabalho   -> item marcado E commit, com o trabalho dentro
#   3) commit que falha             -> marcação reposta, nenhum commit, saída 2
#   4) worker que só mexe na fila   -> a árvore mudou, mas o commit não traz trabalho: desfeito, saída 4
#
# O caso 4 é o que prova a segunda guarda. Sem ele, a guarda que compara o conteúdo do commit
# seria código que ninguém correu -- e uma guarda que nunca corre é uma guarda por provar.
#
# O worker é substituído por `QUEUE_PI`, que o script aceita pela mesma razão que aceita
# `QUEUE_MODEL`: um script que só corre com o worker real não se pode testar.
set -uo pipefail

REPO=$(cd "$(dirname "$0")/.." && pwd) || exit 1
T=$(mktemp -d -t qtest.XXXXXX) || exit 1
L="$T-logs"
mkdir -p "$L"
falhas=0

limpar() {
    # Só apaga o que este script criou com mktemp, e só se o nome for mesmo o dele. Uma guarda
    # que corre com a variável vazia, ou com um caminho que não é o esperado, apaga o diretório
    # errado -- já aconteceu neste workspace. Aqui a guarda falha para o lado de não apagar.
    case "${T:-}" in
        */qtest.*) [ -d "$T" ] && rm -rf "$T" ;;
    esac
    case "${L:-}" in
        */qtest.*) [ -d "$L" ] && rm -rf "$L" ;;
    esac
}
trap limpar EXIT

mkdir -p "$T/scripts" "$T/docs/05-operations"
cp "$REPO/scripts/queue-run-once.sh" "$T/scripts/queue-run-once.sh"

# O gate do repositório de teste devolve PASS sempre: aqui mede-se a fila, não o código.
printf '#!/usr/bin/env bash\necho "VERDICT: PASS"\nexit 0\n' > "$T/scripts/gate.sh"
chmod +x "$T/scripts/gate.sh"

printf '# Spec de teste\n\nUm item que existe para exercitar o script.\n' \
    > "$T/docs/05-operations/SPEC-B99.md"

cd "$T" || exit 1
git init -q
git config user.name "Teste"
git config user.email "teste@local"
git config --local core.hooksPath .git/hooks

# O modelo é obrigatório por desenho; o stub ignora-o.
export QUEUE_MODEL="teste:stub"

printf '#!/bin/sh\nexit 0\n' > "$T/noop"
printf '#!/bin/sh\necho "trabalho do worker" > WORKER-OUTPUT.txt\nexit 0\n' > "$T/work"
printf '#!/bin/sh\necho "nota do worker na propria fila" >> docs/05-operations/QUEUE.md\nexit 0\n' \
    > "$T/na-fila"
chmod +x "$T/noop" "$T/work" "$T/na-fila"

estado_limpo() {
    cat > "$T/docs/05-operations/QUEUE.md" <<'QUEUE'
# QUEUE

- [ ] B99 | docs/05-operations/SPEC-B99.md | item de teste
QUEUE
    rm -f "$T/WORKER-OUTPUT.txt"
    git add -A >/dev/null 2>&1
    git commit -q -m "fila limpa" >/dev/null 2>&1
    base=$(git rev-parse HEAD)
}

marcado() { grep -c '^- \[x\] B99' "$T/docs/05-operations/QUEUE.md" 2>/dev/null || true; }
commits() { git rev-list --count "$base"..HEAD 2>/dev/null || echo 0; }

resultado() {
    if [ "$1" -eq 1 ]; then
        printf '  PASS  queue-run-once: %s\n' "$2"
    else
        printf '  FAIL  queue-run-once: %s\n' "$2"
        falhas=1
    fi
}

# --- caso 1: worker que não faz nada ------------------------------------------------------
estado_limpo
QUEUE_PI="$T/noop" bash "$T/scripts/queue-run-once.sh" > "$L/caso1.log" 2>&1
rc1=$?; m1=$(marcado); c1=$(commits)
if [ "$rc1" -eq 4 ] && [ "$m1" -eq 0 ] && [ "$c1" -eq 0 ]; then
    resultado 1 "caso 1 (worker sem trabalho): rc=4, item não marcado, nenhum commit"
else
    resultado 0 "caso 1: rc=$rc1 (esperado 4), marcado=$m1 (0), commits=$c1 (0) -- ver $L/caso1.log"
fi

# --- caso 2: worker que produz trabalho ---------------------------------------------------
estado_limpo
QUEUE_PI="$T/work" bash "$T/scripts/queue-run-once.sh" > "$L/caso2.log" 2>&1
rc2=$?; m2=$(marcado); c2=$(commits)
no_commit=$(git show --stat --format='' HEAD 2>/dev/null | grep -c 'WORKER-OUTPUT.txt' || true)
if [ "$rc2" -eq 0 ] && [ "$m2" -eq 1 ] && [ "$c2" -eq 1 ] && [ "$no_commit" -ge 1 ]; then
    resultado 1 "caso 2 (trabalho real): item marcado e commit com o ficheiro do worker"
else
    resultado 0 "caso 2: rc=$rc2 (0), marcado=$m2 (1), commits=$c2 (1), ficheiro=$no_commit (>=1) -- ver $L/caso2.log"
fi

# --- caso 3: o commit falha ---------------------------------------------------------------
estado_limpo
printf '#!/bin/sh\necho "hook de teste recusa" >&2\nexit 1\n' > "$T/.git/hooks/pre-commit"
chmod +x "$T/.git/hooks/pre-commit"
QUEUE_PI="$T/work" bash "$T/scripts/queue-run-once.sh" > "$L/caso3.log" 2>&1
rc3=$?; m3=$(marcado); c3=$(commits)
if [ "$rc3" -eq 2 ] && [ "$m3" -eq 0 ] && [ "$c3" -eq 0 ]; then
    resultado 1 "caso 3 (commit falha): marcação reposta, nenhum commit"
else
    resultado 0 "caso 3: rc=$rc3 (esperado 2), marcado=$m3 (0), commits=$c3 (0) -- ver $L/caso3.log"
fi
rm -f "$T/.git/hooks/pre-commit"

# --- caso 4: o worker só mexe na fila -----------------------------------------------------
estado_limpo
QUEUE_PI="$T/na-fila" bash "$T/scripts/queue-run-once.sh" > "$L/caso4.log" 2>&1
rc4=$?; m4=$(marcado); c4=$(commits)
if [ "$rc4" -eq 4 ] && [ "$m4" -eq 0 ] && [ "$c4" -eq 0 ]; then
    resultado 1 "caso 4 (só a fila mudou): commit desfeito, item não marcado"
else
    resultado 0 "caso 4: rc=$rc4 (esperado 4), marcado=$m4 (0), commits=$c4 (0) -- ver $L/caso4.log"
fi

exit "$falhas"