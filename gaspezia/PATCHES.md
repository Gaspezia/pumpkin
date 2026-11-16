# Ecarts du fork Gaspezia/pumpkin par rapport a Pumpkin-MC/Pumpkin

Branche `gaspezia` = `upstream/master` + les lignes ci-dessous + ce dossier `gaspezia/` (CI, docs).
Mise a jour : `gaspezia/update.sh` (merge, jamais de rebase). Aucune PR ni message n'est poste sur Pumpkin-MC.

## Patches

| # | Sujet | Commit(s) | PR / issue upstream | Statut | Date |
|---|-------|-----------|---------------------|--------|------|
| 1 | SDK invite du plugin API v0.2 (`plugin-v0_2`) | merge `1a5cff98f` (tete de PR `0740d3f6`), remis a jour le 2026-11-16 (tete `cc9102bd2`, qui retire `pumpkin-plugin-helpers` et retouche le SDK invite) | Pumpkin-MC/Pumpkin#3962 | ouverte, non fusionnee upstream | 2026-11-10 |
| 2 | Confirmation de teleport : id perime ou inconnu ignore (plus de kick), kick seulement pour un doublon du dernier id sans attente (vanilla `handleAcceptTeleportPacket`) ; branche `gaspezia-patch/ignore-stale-teleport-id`, fichier `crates/pumpkin-core/src/net/java/play/confirm_teleport.rs`. Proposable upstream : oui (correctif autonome, test unitaire inclus) | PR Gaspezia/pumpkin #1 (merge `4558864a4`) | aucune (proposable) | merge | 2026-11-11 |
| 3 | `player-spawn-location-event` tire aussi au join (reapplique a la main le 2026-11-16 : upstream a reorganise `spawn_java_player`, #3708) | branche `gaspezia-patch/spawn-location-on-join` | aucune (proposable) | PR Gaspezia/pumpkin #2 | 2026-11-11 |
| 4 | Metriques du processus pour les plugins v0.2 : `server.get-process-info` (temps CPU, CPU du cgroup, RSS, `memory.max`) et `world.get-loaded-chunk-count` (cette 2e fonction existe upstream depuis le 2026-11-16, meme signature : la version upstream est gardee) | branche `gaspezia-patch/process-metrics` | aucune (proposable) | PR Gaspezia/pumpkin #3 | 2026-11-12 |
| 5 | Feux d'artifice pour les plugins v0.2 : `world.spawn-firework` (fusee avec explosions colorees, explosion immediate optionnelle) | branche `gaspezia-patch/plugin-spawn-firework` | aucune (proposable) | PR Gaspezia/pumpkin #5 | 2026-11-16 |
| 6 | Dialogs utilisables depuis un plugin WASM v0.2 (serialiseur NBT, cles d'inputs, lecture du clic 26.2+) | branche `gaspezia-patch/plugin-dialog-nbt` | aucune (proposable) | PR Gaspezia/pumpkin #6 | 2026-11-16 |

### Patch 3 : `player-spawn-location-event` au join

- Quoi : `World::spawn_java_player` tire `PlayerSpawnLocationEvent` pour tout joueur Java (neuf ou connu) apres avoir calcule la position de depart (sauvegardee ou spawn du monde) et AVANT de l'appliquer, de charger le chunk central et d'envoyer `request_teleport`. La position renvoyee par le plugin est utilisee. Le respawn est inchange.
- Pourquoi : un joueur connu etait restaure a sa derniere position sans que le plugin puisse la changer (le plugin le teleportait ensuite : 2 paquets de position). Paper tire `AsyncPlayerSpawnLocationEvent` pour tous les joueurs avant l'apparition.
- Fichiers : `crates/pumpkin-core/src/world/mod.rs` (11 lignes dans `spawn_java_player`). WIT inchangee : le handler plugin doit etre `blocking = true` pour que l'hote relise `spawn-pos`.
- Limites : pas de monde, pas de rotation, pas de `is_new_player` dans l'evenement ; Bedrock non touche. Analyse : gaspezia-rs-plugins `docs/PARITE-PAPER-PUMPKIN.md` (G43, D12).
- Proposable upstream : oui (petit, sans dependance Gaspezia), a condition de l'ouvrir via l'utilisateur ; chevauche #4519 (`PlayerSpawnLocationEvent`, #4331) : relire cette PR d'abord.

### Patch 4 : metriques du processus pour les plugins

- Quoi : WIT v0.2, ajout seulement (les plugins compiles avant restent compatibles). `server.get-process-info() -> process-info` : `cpu-time-ms` (temps CPU utilisateur + systeme du processus, sysinfo), `available-cpus` (quota `cpu.max` cgroup v2 arrondi au-dessus, sinon `cpu.cfs_quota_us / cpu.cfs_period_us` v1, sinon CPU logiques), `memory-rss` (sysinfo), `memory-limit` (`memory.max` v2 ou `memory.limit_in_bytes` v1, `none` si illimite). Champs CPU soumis a `sys.info` ou `sys.info.cpu`, memoire a `sys.info` ou `sys.info.ram`, comme `get-sys-info`. `world.get-loaded-chunk-count() -> u32` (`Level::loaded_chunk_count`).
- Pourquoi : le `/gc` de GAPI (Java) affiche charge CPU du processus, memoire et chunks charges par monde ; un plugin WASM n'a ni `/proc` ni cgroup (aucun repertoire ouvert dans le contexte WASI) et `get-sys-info` ne donne que la machine entiere.
- Fichiers : `crates/pumpkin-plugin-wit/v0.2/server.wit` ; `crates/pumpkin-wasm-host-v0_2/src/server.rs` (lecture cgroup + tests). `world.get-loaded-chunk-count` est upstream depuis la mise a jour du 2026-11-16.
- Limites : la charge CPU se calcule cote plugin entre deux releves de `cpu-time-ms` ; lecture cgroup Linux seulement (ailleurs : CPU logiques, pas de limite).
- Proposable upstream : oui (sans dependance Gaspezia), a condition de l'ouvrir via l'utilisateur.

### Patch 5 : feux d'artifice pour les plugins

- Quoi : WIT v0.2, ajout seulement. `enum firework-shape` (small-ball, large-ball, star, creeper, burst = ids vanilla 0-4), `record firework-explosion` (`shape`, `colors`, `fade-colors` en RGB `u32`, `trail`, `twinkle`) et `world.spawn-firework(pos, flight-duration: u8, explosions, detonate) -> option<entity>`. L'hote construit la pile `firework_rocket` avec le composant `fireworks` (`build_fireworks` : explosions, couleurs et couleurs de fondu plafonnees a 256 comme le codec reseau), cree un `FireworkRocketEntity` et le fait apparaitre par `World::spawn_entity`. Si `detonate`, `explode_and_remove` est appele dans l'appel : les clients recoivent apparition, metadonnees, evenement d'entite 17 puis suppression.
- Pourquoi : le mode Java dessine des feux d'artifice colores (victoire, fins de manche) avec `FireworkMeta` ; un plugin WASM n'avait aucun moyen de creer une fusee ni d'en fixer les explosions.
- Fichiers : `crates/pumpkin-plugin-wit/v0.2/world.wit` ; `crates/pumpkin-wasm-host-v0_2/src/world.rs` (`spawn_firework`, `build_fireworks`, tests) et `bindings.rs` (fonction asynchrone) ; `crates/pumpkin-plugin-api-v0_2/src/lib.rs` (reexport des 2 types). Aucune modification du coeur.
- Limites : un `EntitySpawnEvent` annule fait renvoyer `none` (aucune fusee). Les couleurs sont stockees telles quelles (`u32 as i32`, pas de masque). Ecart Paper : `Firework.detonate()` explose au tick suivant (`setLife(lifetime + 1)`), ici l'explosion a lieu pendant l'appel ; le rendu client est identique. Le `FireworkExplodeEvent` peut annuler l'explosion (la fusee reste alors en vol).
- Degats : l'explosion inflige les degats vanilla (5 + 2 x nombre d'explosions, rayon 5 blocs ; Paper identique) : faire exploser loin des joueurs.
- Proposable upstream : oui (sans dependance Gaspezia), a condition de l'ouvrir via l'utilisateur.

### Patch 6 : Dialogs depuis un plugin WASM v0.2

- Quoi : `Dialog::to_nbt_compound(version)` ecrit a la main le format vanilla 26.3 (`Dialog.DIRECT_CODEC`) ; `DialogNBT::from_dialog` ne renvoie plus d'erreur pour un dialogue valide. Conventions imposees par l'hote (WIT inchangee) :
  - cles des inputs : `key = "i{index}"` (`i0`, `i1`... dans l'ordre de `inputs`) ;
  - ids des options de `single-option` : l'index decimal (`"0"`, `"1"`...) ; `initial-index` devient `initial: true` sur cette entree ;
  - `after-action` : le serialiseur n'ecrit que `close` ou `wait_for_response` ; toute autre valeur (`peek`, `pop`, `none`, inconnue) devient `close` ; absent : champ omis (defaut vanilla). Cela corrige aussi l'hote v0.1 (qui envoie `peek`/`pop`) sans le modifier ;
  - boutons repartis selon le type : `multi-action` -> `actions` (au moins un), `confirmation` -> `yes`/`no` (les deux premiers), `notice` -> `action` (le premier, optionnel), `server-links` -> champs communs seulement ;
  - `custom-click` -> `minecraft:dynamic/custom` avec `id` et `additions` (le payload s'il se decode en compound NBT sans nom, sinon omis) ; `open-url` -> `minecraft:open_url` ;
  - inputs : `number-range` -> `start`/`end`/`initial`/`step` ; `text` -> `initial` et `max_length` = max(32, longueur de `initial`) ; `boolean` -> `initial`.
  - Valeurs recues au clic (`dialog-click-action-event.payload` = arbre NBT sans nom, compound) : cle `i{index}` -> `Byte` (boolean, 0/1), `Float` (number-range), `String` (text, et id de l'option choisie). Les cles des inputs ecrasent celles d'`additions`.
  - Lecture du clic (config et play) : 26.2+ = VarInt de longueur (<= 65536) puis NBT, `00` = pas de payload ; avant 26.2 = bool + NBT (comportement precedent). Avant ce patch, un payload de 128 octets ou plus etait corrompu et l'absence de payload donnait `Some([0])`.
  - validation (le client leve une `DecoderException` et deconnecte le joueur sur tout champ invalide, donc un dialogue refusable est une erreur cote serveur, rien n'est envoye) : `width` d'un bouton dans 1..=1024 (`Dialog.WIDTH_CODEC`) ; `id` d'un `custom-click` = Identifier valide (parseur `pumpkin_util::identifier`) ; `single-option` sans option refuse ; `number-range` decroissant accepte, `initial` compare aux bornes reelles min/max ;
  - `send_packet` / `enqueue_client_packet` / `try_send_packet` journalisent l'erreur de serialisation au lieu de l'avaler : `warn!` une seule fois par type de paquet pour la duree du processus, puis `debug!` (la meme erreur reviendrait a chaque synchro, par joueur).
- Pourquoi : `from_dialog` renvoyait « Serializing Dialog struct to NBT is not supported » et l'erreur etait avalee : le joueur ne voyait rien. Format verifie contre le vrai codec Java 26.3 (test `#[ignore]` `write_nbt_for_java_verifier` + harnais Java hors depot).
- Fichiers : `crates/pumpkin-protocol/src/java/client/dialog.rs` ; `crates/pumpkin-protocol/src/java/server/{play,config}/custom_click_action.rs` ; `crates/pumpkin-wasm-host-v0_2/src/events/dialog.rs` ; `crates/pumpkin-core/src/net/java/mod.rs`.
- Limites : `dialog-list`, les corps `item` et les valeurs refusables ci-dessus sont refuses (erreur journalisee, rien affiche) ; `columns`, `exit_action`, `button_width`, `placeholder`, `label_visible`, `multiline` ne sont pas exprimables (WIT inchangee) ; les liens de `server-links` sont ignores (vanilla les prend du paquet server_links) ; `initial` hors des bornes et `step` <= 0 d'un `number-range` sont omis (le client les refuserait) ; le plugin doit valider lui-meme les valeurs recues (client modifiable).
- Proposable upstream : oui pour le serialiseur et la lecture du clic (correctifs autonomes) ; la convention des cles `i{index}` est propre au fork (upstream voudra sans doute `key` dans le WIT, changement cassant).

Base initiale : upstream `master` @ `eda69440e` (0.2.2+26.3-26.60).

## Journal des mises a jour

| Date | Commits upstream integres | Conflits | Duree | Build |
|------|---------------------------|----------|-------|-------|
| 2026-11-10 | base initiale (`eda69440e`) | - | - | OK en local (~28 min) |
| 2026-11-10 | essai à blanc de `update.sh` (clone jetable, non poussé) : 16 commits upstream jusqu'à `d393d3581` | aucun | 26 min 16 s | OK ; `Cargo.lock` modifié par le build (restaurer avant le prochain `update.sh`, qui exige un arbre propre) |
| 2026-11-16 | `upstream/master` @ `2285efe83` (89 commits depuis `eda69440e`) + tete de la PR #3962 `cc9102bd2` ; avance du 25/11 (decision de Bastien) | 4 : `v0.2/world.wit` + `wasm-host-v0_2/src/world.rs` (upstream a ajoute le meme `get-loaded-chunk-count` : version upstream gardee) ; `pumpkin-core/src/world/mod.rs` (patch 3 reinsere apres la reorganisation de `spawn_java_player`, #3708) ; `Cargo.lock` (`pumpkin-plugin-helpers` retire par la PR #3962) | verification 14 min 17 s (debug, sans LTO) | clippy + tests `pumpkin-wasm-host-v0_2` (19) OK ; `cargo check -p pumpkin --features plugin-v0_2` OK ; gapi-rs : clippy wasm OK contre le nouveau SDK |
