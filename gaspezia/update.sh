#!/usr/bin/env bash
# Integre upstream (Pumpkin-MC/Pumpkin) dans la branche gaspezia par MERGE, puis compile.
# S'arrete au premier conflit. Ne pousse rien.
set -euo pipefail

PR=3962   # PR portee par le fork, voir gaspezia/PATCHES.md
cd "$(git rev-parse --show-toplevel)"

[ "$(git branch --show-current)" = "gaspezia" ] || { echo "ERREUR: branche courante != gaspezia" >&2; exit 1; }
git diff --quiet && git diff --cached --quiet || { echo "ERREUR: arbre de travail non propre" >&2; exit 1; }
git remote get-url upstream >/dev/null 2>&1 || { echo "ERREUR: remote 'upstream' absent (git remote add upstream https://github.com/Pumpkin-MC/Pumpkin)" >&2; exit 1; }

start=$(date +%s)
before=$(git rev-parse HEAD)

git fetch upstream master "+refs/pull/${PR}/head:refs/remotes/upstream/pr-${PR}"

merge() {
  git merge-base --is-ancestor "$1" HEAD && { echo "$1 deja integre"; return 0; }
  if ! git merge --no-edit "$1"; then
    echo "CONFLIT en integrant $1. Resoudre, 'git commit', puis noter dans gaspezia/PATCHES.md (journal). Annuler: git merge --abort" >&2
    exit 2
  fi
}
merge upstream/master
merge "upstream/pr-${PR}"

echo "Integre: $(git rev-list --count "${before}..HEAD") commits. Build..."
cargo build --release -p pumpkin --features plugin-v0_2
echo "OK en $(( $(date +%s) - start )) s. HEAD=$(git rev-parse HEAD). Ajouter une ligne au journal de gaspezia/PATCHES.md, puis pousser a la main."
