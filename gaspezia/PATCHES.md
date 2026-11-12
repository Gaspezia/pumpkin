# Ecarts du fork Gaspezia/pumpkin par rapport a Pumpkin-MC/Pumpkin

Branche `gaspezia` = `upstream/master` + les lignes ci-dessous + ce dossier `gaspezia/` (CI, docs).
Mise a jour : `gaspezia/update.sh` (merge, jamais de rebase). Aucune PR ni message n'est poste sur Pumpkin-MC.

## Patches

| # | Sujet | Commit(s) | PR / issue upstream | Statut | Date |
|---|-------|-----------|---------------------|--------|------|
| 1 | SDK invite du plugin API v0.2 (`plugin-v0_2`) | merge `1a5cff98f` (tete de PR `0740d3f6`) | Pumpkin-MC/Pumpkin#3962 | ouverte, non fusionnee upstream | 2026-11-10 |
| 2 | Confirmation de teleport : id perime ou inconnu ignore (plus de kick), kick seulement pour un doublon du dernier id sans attente (vanilla `handleAcceptTeleportPacket`) ; branche `gaspezia-patch/ignore-stale-teleport-id`, fichier `crates/pumpkin-core/src/net/java/play/confirm_teleport.rs`. Proposable upstream : oui (correctif autonome, test unitaire inclus) | PR Gaspezia/pumpkin #1 (merge `4558864a4`) | aucune (proposable) | merge | 2026-11-11 |
| 3 | `player-spawn-location-event` tire aussi au join | branche `gaspezia-patch/spawn-location-on-join` | aucune (proposable) | PR Gaspezia/pumpkin #2 | 2026-11-11 |
| 4 | Metriques du processus pour les plugins v0.2 : `server.get-process-info` (temps CPU, CPU du cgroup, RSS, `memory.max`) et `world.get-loaded-chunk-count` | branche `gaspezia-patch/process-metrics` | aucune (proposable) | PR Gaspezia/pumpkin (a ouvrir) | 2026-11-12 |

### Patch 3 : `player-spawn-location-event` au join

- Quoi : `World::spawn_java_player` tire `PlayerSpawnLocationEvent` pour tout joueur Java (neuf ou connu) apres avoir calcule la position de depart (sauvegardee ou spawn du monde) et AVANT de l'appliquer, de charger le chunk central et d'envoyer `request_teleport`. La position renvoyee par le plugin est utilisee. Le respawn est inchange.
- Pourquoi : un joueur connu etait restaure a sa derniere position sans que le plugin puisse la changer (le plugin le teleportait ensuite : 2 paquets de position). Paper tire `AsyncPlayerSpawnLocationEvent` pour tous les joueurs avant l'apparition.
- Fichiers : `crates/pumpkin-core/src/world/mod.rs` (11 lignes dans `spawn_java_player`). WIT inchangee : le handler plugin doit etre `blocking = true` pour que l'hote relise `spawn-pos`.
- Limites : pas de monde, pas de rotation, pas de `is_new_player` dans l'evenement ; Bedrock non touche. Analyse : gaspezia-rs-plugins `docs/PARITE-PAPER-PUMPKIN.md` (G43, D12).
- Proposable upstream : oui (petit, sans dependance Gaspezia), a condition de l'ouvrir via l'utilisateur ; chevauche #4519 (`PlayerSpawnLocationEvent`, #4331) : relire cette PR d'abord.

### Patch 4 : metriques du processus pour les plugins

- Quoi : WIT v0.2, ajout seulement (les plugins compiles avant restent compatibles). `server.get-process-info() -> process-info` : `cpu-time-ms` (temps CPU utilisateur + systeme du processus, sysinfo), `available-cpus` (quota `cpu.max` cgroup v2 arrondi au-dessus, sinon `cpu.cfs_quota_us / cpu.cfs_period_us` v1, sinon CPU logiques), `memory-rss` (sysinfo), `memory-limit` (`memory.max` v2 ou `memory.limit_in_bytes` v1, `none` si illimite). Champs CPU soumis a `sys.info` ou `sys.info.cpu`, memoire a `sys.info` ou `sys.info.ram`, comme `get-sys-info`. `world.get-loaded-chunk-count() -> u32` (`Level::loaded_chunk_count`).
- Pourquoi : le `/gc` de GAPI (Java) affiche charge CPU du processus, memoire et chunks charges par monde ; un plugin WASM n'a ni `/proc` ni cgroup (aucun repertoire ouvert dans le contexte WASI) et `get-sys-info` ne donne que la machine entiere.
- Fichiers : `crates/pumpkin-plugin-wit/v0.2/server.wit`, `world.wit` ; `crates/pumpkin-wasm-host-v0_2/src/server.rs` (lecture cgroup + tests), `world.rs`.
- Limites : la charge CPU se calcule cote plugin entre deux releves de `cpu-time-ms` ; lecture cgroup Linux seulement (ailleurs : CPU logiques, pas de limite).
- Proposable upstream : oui (sans dependance Gaspezia), a condition de l'ouvrir via l'utilisateur.

Base initiale : upstream `master` @ `eda69440e` (0.2.2+26.3-26.60).

## Journal des mises a jour

| Date | Commits upstream integres | Conflits | Duree | Build |
|------|---------------------------|----------|-------|-------|
| 2026-11-10 | base initiale (`eda69440e`) | - | - | OK en local (~28 min) |
| 2026-11-10 | essai à blanc de `update.sh` (clone jetable, non poussé) : 16 commits upstream jusqu'à `d393d3581` | aucun | 26 min 16 s | OK ; `Cargo.lock` modifié par le build (restaurer avant le prochain `update.sh`, qui exige un arbre propre) |
