# Ecarts du fork Gaspezia/pumpkin par rapport a Pumpkin-MC/Pumpkin

Branche `gaspezia` = `upstream/master` + les lignes ci-dessous + ce dossier `gaspezia/` (CI, docs).
Mise a jour : `gaspezia/update.sh` (merge, jamais de rebase). Aucune PR ni message n'est poste sur Pumpkin-MC.

## Patches

| # | Sujet | Commit(s) | PR / issue upstream | Statut | Date |
|---|-------|-----------|---------------------|--------|------|
| 1 | SDK invite du plugin API v0.2 (`plugin-v0_2`) | merge `1a5cff98f` (tete de PR `0740d3f6`) | Pumpkin-MC/Pumpkin#3962 | ouverte, non fusionnee upstream | 2026-11-10 |

Base initiale : upstream `master` @ `eda69440e` (0.2.2+26.3-26.60).

## Journal des mises a jour

| Date | Commits upstream integres | Conflits | Duree | Build |
|------|---------------------------|----------|-------|-------|
| 2026-11-10 | base initiale (`eda69440e`) | - | - | OK en local (~28 min) |
| 2026-11-10 | essai à blanc de `update.sh` (clone jetable, non poussé) : 16 commits upstream jusqu'à `d393d3581` | aucun | 26 min 16 s | OK ; `Cargo.lock` modifié par le build (restaurer avant le prochain `update.sh`, qui exige un arbre propre) |
