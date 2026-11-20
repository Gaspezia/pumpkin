# Ecarts du fork Gaspezia/pumpkin par rapport a Pumpkin-MC/Pumpkin

Branche `gaspezia` = `upstream/master` + les lignes ci-dessous + ce dossier `gaspezia/` (CI, docs).
Mise a jour : `gaspezia/update.sh` (merge, jamais de rebase). Aucune PR ni message n'est poste sur Pumpkin-MC.

## Patches

| # | Sujet | Commit(s) | PR / issue upstream | Statut | Date |
|---|-------|-----------|---------------------|--------|------|
| 1 | SDK invite du plugin API v0.2 (`plugin-v0_2`) | merge `1a5cff98f` (tete de PR `0740d3f6`), remis a jour le 2026-11-16 (tete `cc9102bd2`, qui retire `pumpkin-plugin-helpers` et retouche le SDK invite) | Pumpkin-MC/Pumpkin#3962 | ouverte, non fusionnee upstream | 2026-11-10 |
| 2 | Confirmation de teleport : id perime ou inconnu ignore (plus de kick), kick seulement pour un doublon du dernier id sans attente (vanilla `handleAcceptTeleportPacket`) ; branche `gaspezia-patch/ignore-stale-teleport-id`, fichier `crates/pumpkin-core/src/net/java/play/confirm_teleport.rs`. Proposable upstream : oui (correctif autonome, test unitaire inclus) | PR Gaspezia/pumpkin #1 (merge `4558864a4`) | aucune (proposable) | merge | 2026-11-11 |
| 3 | `player-spawn-location-event` tire aussi au join (reapplique a la main le 2026-11-16 : upstream a reorganise `spawn_java_player`, #3708) | branche `gaspezia-patch/spawn-location-on-join` | aucune (proposable) | PR Gaspezia/pumpkin #2 | 2026-11-11 |
| 4 | Metriques du processus pour les plugins v0.2 : `server.get-process-info` (temps CPU, CPU du cgroup, RSS, working set du cgroup, `memory.max`) et `world.get-loaded-chunk-count` (cette 2e fonction existe upstream depuis le 2026-11-16, meme signature : la version upstream est gardee) | branche `gaspezia-patch/process-metrics` | aucune (proposable) | PR Gaspezia/pumpkin #3 | 2026-11-12 |
| 5 | Feux d'artifice pour les plugins v0.2 : `world.spawn-firework` (fusee avec explosions colorees, explosion immediate optionnelle) | branche `gaspezia-patch/plugin-spawn-firework` | aucune (proposable) | PR Gaspezia/pumpkin #5 | 2026-11-16 |
| 6 | Dialogs utilisables depuis un plugin WASM v0.2 (serialiseur NBT, cles d'inputs, lecture du clic 26.2+) | branche `gaspezia-patch/plugin-dialog-nbt` | aucune (proposable) | PR Gaspezia/pumpkin #6 | 2026-11-16 |
| 7 | `Scoreboard::update_team` conserve les membres de l'equipe (la regle de collision serveur disparaissait apres le 1er `update_team`) | branche `gaspezia-patch/update-team-keeps-members` | aucune (proposable) | PR Gaspezia/pumpkin #7 | 2026-11-17 |
| 19 | Degats de la bordure du monde pour les joueurs hors zone de securite (formule vanilla `outside_border`) | branche `gaspezia-patch/world-border-damage` | aucune (proposable) | PR Gaspezia/pumpkin (lot 4 du 2026-11-20) | 2026-11-20 |
| 20 | `final-damage` et `health` dans `entity-damage-event` et `entity-damage-by-entity-event` | branche `gaspezia-patch/final-damage` | aucune (proposable) | PR Gaspezia/pumpkin (lot 4 du 2026-11-20) | 2026-11-20 |
| 21 | Laisse pour les plugins v0.2 (`set-leash-holder`, `get-leash-holder`, `get-leashed-entities`), forme de la PR upstream #3927 | branche `gaspezia-patch/entity-leash` | aucune (proposable) | PR Gaspezia/pumpkin (lot 4 du 2026-11-20) | 2026-11-20 |
| 22 | `entity-change-block-event` tire a l'atterrissage d'un bloc qui tombe | branche `gaspezia-patch/falling-block-change-event` | aucune (proposable) | PR Gaspezia/pumpkin (lot 4 du 2026-11-20) | 2026-11-20 |
| 23 | `entity-explode-event` porte les blocs a detruire (reductibles) et le proprietaire | branche `gaspezia-patch/explosion-blocks` | aucune (proposable) | PR Gaspezia/pumpkin (lot 4 du 2026-11-20) | 2026-11-20 |
| 24 | `entity.get-owner` / `set-owner` et `world.spawn-primed-tnt` | branche `gaspezia-patch/entity-owner` | aucune (proposable) | PR Gaspezia/pumpkin (lot 4 du 2026-11-20) | 2026-11-20 |
| 25 | `player.launch-fireball` (puissance, incendiaire, vitesse) | branche `gaspezia-patch/launch-fireball` | aucune (proposable) | PR Gaspezia/pumpkin (lot 4 du 2026-11-20) | 2026-11-20 |
| 26 | `projectile-launch-event` pour tous les projectiles lances (perle, boule de neige, boule de feu...) | branche `gaspezia-patch/projectile-launch-all` | aucune (proposable) | PR Gaspezia/pumpkin (lot 4 du 2026-11-20) | 2026-11-20 |
| 27 | Combat PvP 1.8, lot 1 (activation) : `[pvp] combat = "legacy_1_8"` et `World::combat_mode()` ; aucun changement de comportement | branche `gaspezia-patch/pvp18-activation` | aucune (propre au fork) | PR Gaspezia/pumpkin (PvP 1.8, lot 1) | 2026-11-20 |
| 28 | Combat PvP 1.8, lot 1 (melee) : `Player::attack_legacy` (sans recharge ni balayage ni son d'attaque, table d'armes 1.8, Force/Faiblesse, Tranchant/Chatiment/Fleau, critique en sprint, animations 4 et 5, epuisement 0,3, jauge d'attaque masquee) et `entity/combat_legacy.rs` (fonctions pures testees contre les vecteurs) | branche `gaspezia-patch/pvp18-melee` | aucune (propre au fork) | PR Gaspezia/pumpkin (PvP 1.8, lot 1) | 2026-11-20 |
| 29 | Combat PvP 1.8, lot 1 (recul et sprint) : recul de base 1.8 (sans condition de sol, plafond 0,4, jet de resistance tout ou rien, direction depuis l'entite source), recul supplementaire (enchantement Recul + sprint) additif, arret du sprint serveur de l'attaquant, impulsion de saut du fantome serveur, borne +-3,9 du paquet | branche `gaspezia-patch/pvp18-knockback` | aucune (propre au fork) | PR Gaspezia/pumpkin (PvP 1.8, lot 1) | 2026-11-20 |
| 18 | `hide-player` masque reellement le joueur cote client (destruction de l'entite, plus de spawn tant qu'il est cache, retrait de la tab list) ; `show-player` restaure | branche `gaspezia-patch/hide-player` | aucune (proposable) | PR Gaspezia/pumpkin (lot 3 du 2026-11-20) | 2026-11-20 |
| 17 | L'hote ne supprime plus une seconde fois les ressources `own` passees a l'invite (`handle-task`, `on-unload`, `handle-generate-phase`) : fin des traps `resource not present` | branche `gaspezia-patch/scheduler-no-double-delete` | aucune (proposable) | PR Gaspezia/pumpkin (lot 2 du 2026-11-20) | 2026-11-20 |
| 16 | `left-click-block` en ADVENTURE tire depuis l'animation du bras (parite Paper `handleAnimate`) | branche `gaspezia-patch/left-click-from-swing` | aucune (proposable) | PR Gaspezia/pumpkin (lot 2 du 2026-11-20) | 2026-11-20 |
| 15 | Pas de recul d'explosion quand les degats sont refuses (evenement annule par un plugin), parite Paper | branche `gaspezia-patch/explosion-knockback-cancelled` | aucune (proposable) | PR Gaspezia/pumpkin (lot 2 du 2026-11-20) | 2026-11-20 |
| 14 | Un spectateur ne peut pas attaquer (garde `may_attack` dans `Player::attack`, commune a Java, Bedrock et aux plugins) | branche `gaspezia-patch/spectator-no-attack` | aucune (proposable) | PR Gaspezia/pumpkin (lot 2 du 2026-11-20) | 2026-11-20 |
| 13 | `world.create-explosion` : le drapeau `create-fire` pose du feu dans le cratere (regle vanilla) | branche `gaspezia-patch/explosion-fire` | aucune (proposable) | PR Gaspezia/pumpkin (lot 2 du 2026-11-20) | 2026-11-20 |
| 12 | `entity.set-velocity` d'une entite non joueuse diffuse la vitesse aux clients (comme le `hurtMarked` de Paper) | branche `gaspezia-patch/entity-velocity-broadcast` | aucune (proposable) | PR Gaspezia/pumpkin (lot 2 du 2026-11-20) | 2026-11-20 |
| 8 | `play-sound`, `play-sound-at` et `stop-sound` de l'hote v0.2 : correspondance fiable entre le variant WIT et `pumpkin_data::sound::Sound` (plus de trap sur tout son) | branche `gaspezia-patch/plugin-play-sound` | aucune (proposable) | PR Gaspezia/pumpkin (lot du 2026-11-19) | 2026-11-19 |
| 9 | `custom_name` d'un objet garde ses styles (gras, couleurs) a l'encodage reseau : `CustomNameImpl` encode comme `LoreImpl` | branche `gaspezia-patch/custom-name-styles` | aucune (proposable) | PR Gaspezia/pumpkin (lot du 2026-11-19) | 2026-11-19 |
| 10 | `display.set-interpolation-start` toujours renvoye au client (champ force comme vanilla) : `SynchedEntityData::set_forced` | branche `gaspezia-patch/forced-interpolation` | aucune (proposable) | PR Gaspezia/pumpkin (lot du 2026-11-19) | 2026-11-19 |
| 11 | Un trap de plugin journalise sa cause complete (chaine `Caused by`) : `DriverError` et log du scheduler en `Debug` | branche `gaspezia-patch/plugin-trap-cause` | aucune (proposable) | PR Gaspezia/pumpkin (lot du 2026-11-19) | 2026-11-19 |

### Patch 3 : `player-spawn-location-event` au join

- Quoi : `World::spawn_java_player` tire `PlayerSpawnLocationEvent` pour tout joueur Java (neuf ou connu) apres avoir calcule la position de depart (sauvegardee ou spawn du monde) et AVANT de l'appliquer, de charger le chunk central et d'envoyer `request_teleport`. La position renvoyee par le plugin est utilisee. Le respawn est inchange.
- Pourquoi : un joueur connu etait restaure a sa derniere position sans que le plugin puisse la changer (le plugin le teleportait ensuite : 2 paquets de position). Paper tire `AsyncPlayerSpawnLocationEvent` pour tous les joueurs avant l'apparition.
- Fichiers : `crates/pumpkin-core/src/world/mod.rs` (11 lignes dans `spawn_java_player`). WIT inchangee : le handler plugin doit etre `blocking = true` pour que l'hote relise `spawn-pos`.
- Limites : pas de monde, pas de rotation, pas de `is_new_player` dans l'evenement ; Bedrock non touche. Analyse : gaspezia-rs-plugins `docs/PARITE-PAPER-PUMPKIN.md` (G43, D12).
- Proposable upstream : oui (petit, sans dependance Gaspezia), a condition de l'ouvrir via l'utilisateur ; chevauche #4519 (`PlayerSpawnLocationEvent`, #4331) : relire cette PR d'abord.

### Patch 4 : metriques du processus pour les plugins

- Quoi : WIT v0.2, ajout seulement (les plugins compiles avant restent compatibles). `server.get-process-info() -> process-info` : `cpu-time-ms` (temps CPU utilisateur + systeme du processus, sysinfo), `available-cpus` (quota `cpu.max` cgroup v2 arrondi au-dessus, sinon `cpu.cfs_quota_us / cpu.cfs_period_us` v1, sinon CPU logiques), `memory-rss` (sysinfo), `memory-working-set` (working set du cgroup, comme kubelet et `kubectl top` : `memory.current` - `inactive_file` de `memory.stat` en v2, `memory.usage_in_bytes` - `total_inactive_file` en v1, `none` hors cgroup ; ajoute le 2026-11-19, le RSS comptait ~60 Mio de cache de fichier de plus), `memory-limit` (`memory.max` v2 ou `memory.limit_in_bytes` v1, `none` si illimite). Champs CPU soumis a `sys.info` ou `sys.info.cpu`, memoire a `sys.info` ou `sys.info.ram`, comme `get-sys-info`. `world.get-loaded-chunk-count() -> u32` (`Level::loaded_chunk_count`).
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

### Patch 7 : `update_team` conserve les membres

- Quoi : `Scoreboard::update_team` remplacait l'equipe par celle construite par l'appelant (l'hote WASM la construit avec `players: []`) : la liste des membres cote serveur etait perdue. Elle est maintenant reprise de l'equipe existante avant le remplacement. Le paquet `CSetPlayerTeam` en mode `Update` n'est pas modifie : il ne porte que les options, sans membres, comme vanilla (`ClientboundSetPlayerTeamPacket.createPlayerPacket(team, false)`) ; le client garde donc les siens.
- Pourquoi : apres un `update_team`, `get_entity_team(nom)` renvoyait `None` et la regle de collision d'equipe du coeur (`is_allowed_by_team_rules`, `push_entities`) ne s'appliquait plus aux joueurs de l'equipe. Un mode qui met a jour l'equipe a chaque manche perdait la regle des la 1re.
- Fichiers : `crates/pumpkin-core/src/world/scoreboard.rs` (`update_team` + module `tests`, test `update_team_keeps_members_and_applies_options`, ecrit d'abord et rouge avant le correctif).
- Limites : un `update_team` ne peut plus vider les membres (il faut `clear_team_players` ou `remove_player_from_team`, comme vanilla). La copie WASM v0_1 passe par la meme methode du coeur : corrigee du meme coup.
- Proposable upstream : oui (sans dependance Gaspezia), a condition de l'ouvrir via l'utilisateur.

### Patch 8 : `play-sound` de l'hote v0.2

- Quoi : le nom interne etait construit par `format!("{sound:?}")` en minuscules (`Debug` d'un enum wasmtime donne `Sound::EntityGenericExplode`) et ne correspondait a aucun son : `Unknown sound`, erreur `wasmtime` et instance du plugin tuee. `from_wit_sound` (`wasm-host-v0_2/src/world.rs`) compare maintenant les deux noms sur leurs seuls caracteres alphanumeriques (`entity.generic.explode` et `EntityGenericExplode`), via une table construite une fois depuis `Sound::slice()`. Utilise par `World::play_sound`, `Player::play_sound`, `play_sound_at` et `stop_sound`. Un son sans correspondance est ignore avec un avertissement, sans trapper.
- Pourquoi : tout plugin qui appelait `play-sound` perdait son instance au premier appel (constate au banc Moutrun-rs, `entity.generic.explode`).
- Fichiers : `crates/pumpkin-wasm-host-v0_2/src/world.rs` (helper + tests : un son de chaque categorie, unicite des noms normalises), `player.rs`.
- Limites : la signature WIT est inchangee (`play-sound` ne renvoie pas de resultat) : un son inconnu est silencieux cote plugin.
- Proposable upstream : oui (a verifier d'abord si upstream l'a deja corrige), a condition de l'ouvrir via l'utilisateur.

### Patch 9 : `custom_name` style sur le reseau

- Quoi : `CustomNameImpl::serialize` aplatissait le composant en texte (`get_text()`) : le client recevait une chaine brute, sans gras ni couleur, et en italique par defaut. Il l'encode maintenant avec `encode_for_version(V_26_2)`, comme `LoreImpl`. Le hash (`basic.rs`) reste celui du texte, comme pour le lore.
- Pourquoi : les noms d'objets des modes (ex. le lit du lobby : `<bold><red>Retour au Spawn</red></bold> ...`) perdaient leurs styles.
- Fichiers : `crates/pumpkin-protocol/src/codec/data_component.rs` (+ test `custom_name_keeps_its_styles_on_the_wire` : un nom gras et rouge relu par `deserialize` garde ses styles).
- Limites : le defaut italique du client reste pour un nom sans `italic:false` explicite (a poser cote mode).
- Proposable upstream : oui (sans dependance Gaspezia), a condition de l'ouvrir via l'utilisateur.

### Patch 10 : interpolation d'un display toujours envoyee

- Quoi : `SynchedEntityData::set` ne marque un champ sale que si sa valeur change. `SynchedEntityData::set_forced` (et `Entity::set_synced_data_forced`) le marque toujours, comme le `force=true` de vanilla (`Display.setTransformationInterpolationDelay`). `DisplayEntity::set_interpolation_start_delta_ticks` l'utilise.
- Pourquoi : le client ne relance l'interpolation d'un display qu'a la reception de ce champ ; sans cela, une animation envoyee avec `delay=0` et `duration=n` saute d'une pose a l'autre au lieu d'etre lissee.
- Fichiers : `crates/pumpkin-core/src/entity/synched_entity_data.rs` (+ test `set_forced_marks_dirty_even_when_unchanged`), `entity/mod.rs`, `entity/decoration/display.rs`.
- Limites : le renvoi groupe de l'etat du display (`display.rs`, autre appel a `START_INTERPOLATION`) garde l'envoi sur changement.
- Proposable upstream : oui (fidelite vanilla).

### Patch 11 : cause complete d'un trap de plugin

- Quoi : `DriverError::new(error.to_string())` (`pumpkin-plugin-runtime/src/executor.rs`) devient `format!("{error:?}")`, et le log du scheduler (`pumpkin-wasm-host/src/scheduler.rs`) passe de `%error` a `error = ?error` : le `Debug` d'une erreur `wasmtime` garde la chaine `Caused by`.
- Pourquoi : un trap ne journalisait que `error while executing at wasm backtrace`, sans la cause, ce qui bloquait le diagnostic.
- Fichiers : les deux ci-dessus. Journalisation seule, aucun changement de comportement.
- Proposable upstream : oui.


### Patch 17 : plus de double suppression des ressources `own`

- Quoi : le scheduler (`pumpkin-wasm-host/src/scheduler.rs`, branches v0_1 et v0_2) refaisait `resource_table.delete::<Arc<Server>>(new_own(rep))` apres `handle_task`. Le handle est passe en `own` : l'invite le libere (l'hote le retire de la table via `drop`). Si l'invite creait une ressource entre-temps (entite de `spawn-entity`, flux wasi...), elle reprenait la case et ce second delete la DETRUISAIT ; l'appel suivant trappait (`resource not present`). Meme motif retire pour `on-unload` (contexte, `runtime.rs`) et `handle-generate-phase` (`ChunkBuffer`, hotes v0_1 et v0_2). Non touches : le dispatch d'evenements (il ne supprime que les ressources du RETOUR de l'invite, ou en cas d'echec avant l'appel) et les chemins d'erreur avant l'appel de `mob.rs`.
- Preuve (banc du moteur de modeles) : 4 traps sur 4 avec l'ancien code, 0 sur 3 sans la liberation.
- Fichiers : `crates/pumpkin-wasm-host/src/scheduler.rs`, `runtime.rs`, `crates/pumpkin-wasm-host-v0_1/src/world.rs`, `crates/pumpkin-wasm-host-v0_2/src/world.rs`.
- Limites : pas de test unitaire (il faudrait un composant invite compile pour reproduire la reutilisation de case) ; si l'invite trappe avant de liberer le handle, celui-ci reste dans la table (fuite negligeable, le store est mort).

### Patch 12 : `set-velocity` diffuse la vitesse

- Quoi : l'hote v0.2 (`entity.rs`, `set_velocity`) faisait `velocity.store(..)` seul : le client ne recevait la vitesse qu'au prochain envoi periodique, d'ou des arcs saccades (moutons lances, missiles, recul). Il appelle maintenant `Entity::set_velocity`, qui stocke puis envoie `CEntityVelocity` aux observateurs (ou `send_hurt_velocity` pour un joueur), comme l'hote joueur.
- Pourquoi : un mode qui pose une vitesse a chaque tick ou lance une entite en arc obtenait 0,7 a 4 paquets de vitesse par seconde au lieu d'un par tick.
- Fichiers : `crates/pumpkin-wasm-host-v0_2/src/entity.rs` ; test `set_velocity_is_sent_to_watchers` dans `crates/pumpkin-core/src/entity/mod.rs`.
- Limites : un envoi par appel (pas de regroupement par tick).
- Proposable upstream : oui.


### Patch 13 : feu d'explosion

- Quoi : l'hote v0.2 ignorait `create-fire` (`_create_fire`) et le cœur avait un `// TODO: fire`. `Explosion::with_fire` (+ `World::explode_with_fire`) pose, apres la destruction des blocs, du feu sur chaque position detruite avec 1 chance sur 3, si elle est de l'air au-dessus d'une face pleine (regle vanilla `ServerExplosion.interactWithBlocks`) ; feu d'ame sur sable/terre des ames, `FireBlock::get_state_for_position` sinon. Pas de feu sans destruction de blocs (`interaction::none`), comme vanilla.
- Pourquoi : les moutons incendiaires et les boules de feu d'un mode devaient poser le feu a la main (photo des blocs avant/apres).
- Fichiers : `crates/pumpkin-core/src/world/explosion.rs` (+ test `fire_is_scattered_on_one_in_three_air_positions_above_solid`), `world/mod.rs`, `crates/pumpkin-wasm-host-v0_2/src/world.rs`.
- Limites : les autres appelants (`explode`, `explode_from`) ne posent pas de feu (comportement inchange).
- Proposable upstream : oui.


### Patch 14 : un spectateur n'attaque pas

- Quoi : `Player::attack` (`pumpkin-core/src/entity/player.rs`) sort immediatement si le joueur est en `GameMode::Spectator`, comme le `handleAttack` vanilla. Placee dans `attack` plutot que dans `net/java/play/attack.rs` : tous les chemins (paquet Attack, Interact, transaction Bedrock) y passent.
- Pourquoi : un joueur elimine reste spectateur et pouvait quand meme frapper et repousser joueurs et mobs.
- Fichiers : `crates/pumpkin-core/src/entity/player.rs` (+ test `only_spectators_cannot_attack` sur la fonction pure `may_attack`).
- Limites : le test porte sur la regle, pas sur un paquet reel.
- Proposable upstream : oui.


### Patch 15 : pas de recul si les degats d'explosion sont annules

- Quoi : dans `Explosion::damage_entities`, si l'explosion a tente d'infliger des degats et que `damage_with_context` renvoie `false` (evenement `entity-damage` / `entity-damage-by-entity` annule, ou degats refuses), l'entite n'est plus repoussee (et ne memorise pas d'impulsion). Une entite qui ne doit pas subir de degats (`should_damage_entity` faux, ex. vent) est repoussee comme avant.
- Pourquoi : un mode qui annule les degats natifs pour appliquer son propre recul obtenait un recul double ; Paper ne repousse pas si `EntityDamageEvent` est annule.
- Fichiers : `crates/pumpkin-core/src/world/explosion.rs` (+ test `refused_damage_cancels_the_knockback` sur la regle pure `knockback_allowed`).
- Limites : tout refus de degats (pas seulement l'annulation par plugin, ex. entite invulnerable) supprime aussi le recul.
- Proposable upstream : oui, a discuter (changement de comportement vanilla pour les entites invulnerables).


### Patch 16 : clic gauche sur un bloc depuis le balancement du bras

- Quoi : `handle_swing_arm` tirait `LeftClickAir` sur un rayon sans bloc, mais rien pour un bloc touche. En ADVENTURE, quand le bloc ne peut pas etre mine (`cannot_mine_block`), il tire maintenant `PlayerInteractEvent` `LeftClickBlock` avec le bloc et sa position, comme `handleAnimate` de Paper ; `cancelled` est lu comme pour l'air.
- Pourquoi : un client vanilla en ADVENTURE ne mine pas, il agite seulement le bras : aucun `left-click-block` n'arrivait au plugin (le missile de SheepWars en depend).
- Fichiers : `crates/pumpkin-core/src/net/java/play/swing_arm.rs` (+ test `swing_is_a_left_click_only_in_adventure_on_unbreakable_blocks` sur la regle pure `left_click_block_from_swing`).
- Limites : pas de face touchee dans l'evenement (le constructeur n'en porte pas) ; Java seulement. Si le bloc est minable, le paquet de minage tire l'evenement : pas de doublon.
- Proposable upstream : oui.


### Patch 18 : `hide-player` effectif

- Quoi : `Player::hide_player` ne faisait qu'enregistrer l'UUID, jamais lu. Il retire maintenant l'entite du suivi de l'observateur (`TrackedEntity::remove_player` : paquet de destruction) et sa ligne de la tab list (`CRemovePlayerInfo`) ; `TrackedEntity::broadcast_to_player` refuse tout joueur cache (donc pas de re-spawn au deplacement ou au changement de chunk). `show_player` renvoie d'abord la ligne de tab list (`World::send_tab_list_entry`, Java supprime les spawns sans elle) puis relance le suivi (`update_player`).
- Pourquoi : un mode (Loup-Garou) masque les dormeurs aux autres joueurs ; le joueur cache restait visible.
- Fichiers : `crates/pumpkin-core/src/entity/player.rs`, `world/entity_tracker.rs` (+ tests `a_hidden_player_is_never_visible` et `spectators_are_seen_only_by_spectators` sur la regle pure `player_visible_to`), `world/mod.rs`.
- Limites : le retrait/renvoi de la tab list n'est fait que pour les clients Java ; un joueur cache qui rejoint apres coup, ou change de monde, reste soumis au filtre du suivi mais sa ligne de tab list peut reapparaitre. Pas de test de bout en bout (aucun joueur factice dans le cœur).
- Proposable upstream : oui.


### Patch 19 : degats de la bordure du monde

- Quoi : un joueur dont la position est au-dela de la zone de securite de la bordure (`buffer`, 5 blocs par defaut) subit chaque tick `max(1, floor(-(distance a la bordure + zone de securite) x degats par bloc))` de type `outside_border`, comme `LivingEntity.baseTick` vanilla (joueurs seulement, apres le test de suffocation). `Worldborder::new` prend les valeurs par defaut vanilla (0,2 degat par bloc, zone de securite de 5) au lieu de 0 : une bordure posee par un plugin (`set-size`) ou par `/worldborder` blesse desormais.
- Pourquoi : un mode (Arena) reduit la bordure en fin de manche et attend que les joueurs hors zone prennent des degats comme avec Paper ; Pumpkin n'en infligeait aucun (le champ `damage_per_block` n'etait lu nulle part).
- Fichiers : `crates/pumpkin-core/src/world/border.rs` (`outside_damage` + test `border_damage_follows_the_vanilla_formula`), `entity/living.rs`.
- Limites : le test vanilla `isWithinBounds(boite)` n'est pas refait (la distance du point suffit des que la zone de securite est >= 0) ; les degats suivent `new_diameter` (la bordure ne s'interpole pas cote serveur, comportement existant) ; les valeurs de `level.dat` (`BorderDamagePerBlock`, `BorderSafeZone`) ne sont toujours pas relues.
- Proposable upstream : oui (fidelite vanilla).
### Patch 20 : degats finaux dans `entity-damage-event`

- Quoi : `entity-damage-event-data` et `entity-damage-by-entity-event-data` (WIT v0.2) portent `final-damage: f32` (ce qui sera retire a la vie : casque x0,75, cooldown de coup, armure et robustesse, resistance, protections enchantees, absorption) et `health: f32` (vie de la victime au moment du coup). `LivingEntity::preview_final_damage` rejoue le pipeline de `damage_with_context` sans effet de bord (pas d'usure d'armure, pas de statistique) ; `get_damage_after_armor_absorb` et `get_damage_after_magic_absorb` sont decoupes en une partie pure reutilisee. Les deux champs sont informatifs : les relire dans le plugin n'a aucun effet, `damage` reste le champ a modifier.
- Pourquoi : GArena et BedWars annulent un coup mortel (`vie <= degat final`) pour que le joueur ne passe jamais par l'ecran de mort. Avec le montant brut le mode devait estimer l'armure. Cela couvre aussi le besoin « mort annulable » : le plugin annule lui-meme l'evenement de degats quand `final-damage >= health`.
- Fichiers : `crates/pumpkin-core/src/entity/living.rs` (+ test `preview_final_damage_matches_what_reaches_the_health`), `plugin/api/events/entity/entity_damage*.rs`, `crates/pumpkin-plugin-wit/v0.2/event.wit`, `crates/pumpkin-wasm-host-v0_2/src/events/entity.rs`.
- Limites : le blocage au bouclier n'est pas previsualise (effets de bord) ; un plugin qui change `damage` dans le premier evenement voit le second recalculer `final-damage` sur le nouveau montant ; `entity-damage-by-block-event` n'a pas les champs ; les entites sans vie (item, orbe) recoivent `final-damage` = `damage` et `health` = 0.
- Proposable upstream : oui (champ ajoute, changement cassant du WIT v0.2 pour les plugins qui construisent l'enregistrement).
### Patch 21 : laisse (`set-leash-holder`, `get-leash-holder`, `get-leashed-entities`)

- Quoi : WIT v0.2, ajout seulement, avec les noms et signatures de la PR upstream Pumpkin-MC/Pumpkin#3927 (ouverte, non fusionnee) : `entity.is-leashed() -> bool`, `entity.get-leash-holder() -> option<entity>`, `entity.get-leashed-entities() -> list<entity>` et `mob.set-leash-holder(option<entity>)` (laisse ou, avec `none`, detache). Le code hote est reecrit chez nous : le cœur garde le detenteur en `Weak`, on passe donc par `Entity::get_leash_holder`, et `get-leashed-entities` parcourt les entites et joueurs du monde. `leash_to` / `unleash` du cœur tirent leurs evenements (`PlayerLeashEntityEvent`, `EntityUnleashEvent`) comme d'habitude ; un plugin relit `is-leashed` pour savoir si un gestionnaire a annule.
- Pourquoi : le largage de coffre de GArena suspend le coffre sous 5 poulets relies par des laisses a un armor stand (`setLeashHolder`).
- Fichiers : `crates/pumpkin-plugin-wit/v0.2/world.wit`, `crates/pumpkin-wasm-host-v0_2/src/entity.rs` et `mob.rs`. Aucune modification du cœur.
- Limites : pas de test automatique (aucun composant invite) ; la distance de rupture de la laisse et la chute du detenteur restent celles du cœur ; `unleash` ne fait pas tomber l'objet laisse ; `set-leash-holder` n'est que sur `mob` (comme la PR upstream) : un joueur ou un armor stand ne peut pas etre le laisse.
- Proposable upstream : sans objet (la PR #3927 existe) ; a la mise a jour, garder la version upstream si elle est fusionnee.
### Patch 22 : `entity-change-block-event` a l'atterrissage d'un bloc qui tombe

- Quoi : `FallingBlockEntity::tick` tire `EntityChangeBlockEvent` (position d'arrivee, bloc pose) juste avant `set_block_state`. Annule : le bloc ne se pose pas et tombe en objet (comportement Paper), l'entite est retiree.
- Pourquoi : GArena memorise la position d'arrivee de la poudre de beton (`onEntityChangeBlock`) pour restaurer la carte a la fin de la manche ; sans l'evenement, les blocs tombes restaient d'une manche a l'autre.
- Fichiers : `crates/pumpkin-core/src/entity/falling.rs`. WIT inchangee (l'enregistrement existait deja).
- Source : l'idee (evenement a l'atterrissage) existe dans le fork ArkFlame/Kabocha (GPL v3, `falling.rs`) ; le code est reecrit ici, sans copie, sur le modele de `pick_up_block.rs`.
- Limites : pas de test unitaire (la boucle de tick demande un monde complet) ; le nom du bloc est celui apres solidification (poudre -> beton).
### Patch 23 : blocs et proprietaire dans `entity-explode-event`

- Quoi : `entity-explode-event-data` porte `blocks: list<block-pos>` (blocs que l'explosion va detruire, calcules avant l'evenement par `Explosion::get_blocks_to_destroy`) et `owner-id: option<s32>` (entite creditee : celui qui a allume la TNT, tire la boule de feu). Le plugin retire des entrees pour epargner des blocs ; `Explosion::explode` recoit l'ensemble reduit et ne recalcule rien. Vide pour `interaction::none`.
- Pourquoi : BedWars ne laisse partir que les blocs poses par un joueur (laine, toile, feuilles) et jamais un lit ; le mode annulait l'explosion et recassait une sphere a la main, sans la geometrie ni les drops vanilla.
- Fichiers : `crates/pumpkin-core/src/world/explosion.rs`, `world/mod.rs` (`run_explosion`), `plugin/api/events/entity/entity_explode.rs`, `event.wit`, `crates/pumpkin-wasm-host-v0_2/src/events/entity.rs`.
- Limites : seules les entrees d'origine comptent (un bloc ajoute est ignore) ; les blocs sont calcules avant les degats aux entites (ordre vanilla) ; `block-explode-event` n'est pas change ; pas de test unitaire dedie (la logique est un `retain`).
### Patch 24 : proprietaire d'une entite et TNT amorcee

- Quoi : `entity.get-owner() -> option<entity>` et `entity.set-owner(option<entity>) -> bool` (projectiles et TNT amorcee ; faux pour les autres entites), `world.spawn-primed-tnt(pos, fuse-ticks, owner) -> option<entity>`. Cote cœur : `EntityBase::set_owner_entity`, `ProjectileOwner::set_owner`, `get_owner_entity` implemente pour `TNTEntity`.
- Pourquoi : BedWars credite les morts par TNT, boule de feu ou fleche a leur proprietaire (`TNTPrimed.getSource`, `Projectile.getShooter`) ; le mode tenait une table id -> uuid, aveugle aux fleches.
- Fichiers : `crates/pumpkin-core/src/entity/mod.rs`, `tnt.rs`, `projectile/owner.rs`, `crates/pumpkin-plugin-wit/v0.2/world.wit`, `crates/pumpkin-wasm-host-v0_2/src/entity.rs` et `world.rs`.
- Limites : pas de test automatique ; `spawn-primed-tnt` utilise la puissance par defaut (4) et ne lui donne pas de vitesse initiale ; un `EntitySpawnEvent` annule renvoie `none`.
- Proposable upstream : oui.
### Patch 25 : `player.launch-fireball`

- Quoi : `player.launch-fireball(power, incendiary, speed) -> option<entity>` (asynchrone) lance une boule de feu depuis les yeux du joueur dans sa ligne de visee, creditee au joueur. `FireballEntity` gagne `incendiary` (l'explosion pose du feu, via `Explosion::with_fire` du patch 13) et `set_speed` (acceleration en blocs par tick, 0,1 par defaut ; zero ou moins = defaut).
- Pourquoi : la boutique de BedWars vend une boule de feu de puissance 2, non incendiaire ; le mode lancait une boule vanilla et recreait l'explosion.
- Fichiers : `crates/pumpkin-core/src/entity/projectile/fireball.rs`, `crates/pumpkin-plugin-wit/v0.2/player.wit`, `crates/pumpkin-wasm-host-v0_2/src/player.rs`.
- Limites : `launch-projectile` reste un stub renvoyant `none` ; pas de test automatique ; l'explosion de la boule garde l'interaction `Mob` (regle `mobGriefing`).
- Proposable upstream : oui (apres le patch 13, dont il depend pour le feu).
### Patch 26 : `projectile-launch-event` pour tous les projectiles lances

- Quoi : `ThrownItemEntity::owned` (point commun de la perle, de la boule de neige, de l'oeuf, des potions, de la boule de feu, du crâne de wither...) tire `ProjectileLaunchEvent` avec le tireur. La fleche et le trident tiraient deja le leur.
- Pourquoi : BedWars compte les perles lancees (`pearlsThrown`) ; l'evenement ne tirait que pour fleche et trident.
- Fichiers : `crates/pumpkin-core/src/entity/projectile/mod.rs`.
- Limites : l'entite n'est pas encore apparue : `cancelled` n'est pas lu (comme pour la fleche) ; tire aussi pour les tirs de mobs (blaze, ghast) comme Paper ; pas de test unitaire.
- Proposable upstream : oui.

### Patch 27 : activation du combat PvP 1.8

- Quoi : `crates/pumpkin-config/src/pvp.rs` gagne `combat: CombatMode` (`"modern"` par defaut, `"legacy_1_8"`), range sous `[pvp]` de `pumpkin.toml`. `World::combat_mode()` lit ce reglage : point d'acces unique, pour qu'une surcharge par monde puisse arriver sans toucher aux appelants. Aucun changement de comportement tant que le reglage reste `modern`.
- Pourquoi : un serveur Agones heberge un seul mode ; Arena (et plus tard BedWars) jouent le combat 1.8, le lobby et les autres modes gardent le moderne. Les formules arrivent dans les patchs 28 (melee) et 29 (recul).
- Fichiers : `crates/pumpkin-config/src/pvp.rs`, `lib.rs`, `crates/pumpkin-core/src/world/mod.rs`. WIT inchangee.
- Limites : portee serveur entier, pas par monde. Propre au fork (jamais propose upstream).

### Patch 28 : melee 1.8

- Quoi : quand `[pvp] combat = "legacy_1_8"` (patch 27), `Player::attack` passe a `attack_legacy` (`entity/player/attack_legacy.rs`) : aucune recharge (ni garde `minimum_attack_charge`), aucun balayage, aucun son d'attaque ; valeur d'attaque de l'outil tenu prise dans la table 1.8 (epee 4 + materiau, hache 3 +, pioche 2 +, pelle 1 +, or comme le bois ; ajoutee a la base 1,0) tant que le plugin n'a pas redefini les `attribute_modifiers` de la pile ; Force = `x (1 + 1,3 (niv))` et Faiblesse = `- 0,5 (niv)` avant les multiplicateurs ; critique 1.8 (chute, ni au sol, ni echelle, ni eau, ni Cecite, ni monture, cible vivante ; le sprint n'empeche pas) a `x 1,5` AVANT l'ajout des enchantements ; Tranchant 1,25, Chatiment et Fleau 2,5 par niveau ; animations de critique 4 (physique) et 5 (magique, si enchantement) envoyees apres la vitesse de la victime ; epuisement 0,3 ; la base `attack_speed` du joueur est poussee a 1024 (tick du joueur) pour que le client n'affiche plus la jauge. Les formules sont des fonctions pures dans `entity/combat_legacy.rs`, testees contre les vecteurs D1 a D4 de la spec.
- Pourquoi : Arena rejoue le PvP 1.8 ; le moderne divise les degats par la recharge et n'a ni les memes valeurs d'armes ni les memes enchantements.
- Fichiers : `crates/pumpkin-core/src/entity/combat_legacy.rs` (nouveau), `entity/player/attack_legacy.rs` (nouveau), aiguillage de 5 lignes dans `Player::attack`, une ligne dans le tick du joueur, `entity/mod.rs`. Les formules viennent des faits du jeu 1.8 (aucun code Mojang copie).
- Limites : invulnerabilite, armure 1.8, blocage et projectiles restent ceux du moderne (lots L4 a L7) ; Aura de feu via `on_post_attack` moderne (sans le `setFire(1)` prealable) ; masse, lance, trident et netherite gardent leurs valeurs modernes ; pas de test d'integration (monde complet requis), le banc Mineflayer verifiera.

### Patch 29 : recul et sprint 1.8

- Quoi : en mode `legacy_1_8`, `LivingEntity::deal_default_knockback` passe a `deal_default_knockback_legacy` (`entity/combat_legacy_hit.rs`) : direction = position de l'entite source moins celle de la victime (le tireur pour un projectile, aleatoire si les deux sont presque confondus), jet de resistance tout ou rien (`rand < resistance` annule le recul), puis `m = m / 2` sur les trois axes, `-dir x 0,4F`, `+0,4F` vertical et plafond 0,4 : une victime en l'air est relancee, une victime immobile au sol recoit `(0 ; 0,3608 ; 0,4)`. Dans `attack_legacy`, recul supplementaire `i = niveau de Recul + 1 si l'attaquant sprinte` : `+(-sin(lacet) i 0,5 ; 0,1 ; cos(lacet) i 0,5)` ajoute (+0,1 vertical une seule fois), sans resistance, sur tout coup ACCEPTE (un coup partiel pendant l'invulnerabilite le laisse dans le fantome serveur sans paquet) ; vitesse serveur de l'attaquant `x 0,6` en X/Z et `set_sprinting(false)` apres le coup (le W-tap redevient necessaire) ; la vitesse envoyee a la victime est bornee a +-3,9 ; `Player::jump` donne au fantome serveur l'impulsion 1.8 (0,42 + 0,1 par niveau de Saut, +0,2 le long du lacet en sprint). Ordre des paquets : evenements de degats, vitesse de la victime, animations de critique, metadonnee de sprint de l'attaquant.
- Pourquoi : c'est le ressenti du combat 1.8 (combos en l'air, bonus de sprint, W-tap). Le moderne ne releve pas en l'air, divise deux fois la vitesse en sprint et ne coupe jamais le sprint de l'attaquant.
- Fichiers : `crates/pumpkin-core/src/entity/combat_legacy.rs` (formules : `base_knockback`, `extra_knockback`, `jump_motion`...), `entity/combat_legacy_hit.rs` (nouveau), `entity/player/attack_legacy.rs`, aiguillages dans `LivingEntity::deal_default_knockback` et `Player::jump`. Les evenements de plugin `entity-knockback*` et `player-velocity` sont tires comme sur le chemin moderne (le recul supplementaire est un evenement a part, comme avant).
- Limites : `sin` et `cos` exacts au lieu de la table de 1.8 (ecart < 1e-4, sous le critere de 2,5e-4 du banc) ; le fantome n'est pas verifie tick par tick (le banc mesurera) ; le recul d'explosion, de canne et des fleches (Frappe) reste a faire (lots L6, L7, L9) ; le client 26.x decide seul de son ralentissement et de la relance de son sprint.

### Patch 30 : `hide-player` sans fuite de spawn

- Quoi : `TrackedEntity::update_player_in_view` relit la liste des caches (`broadcast_to_player`) sous le verrou d'ecriture de `seen_by` au moment d'inserer l'observateur, puis une derniere fois apres l'envoi du spawn (`add_pairing`) : un `hide_player` qui passait entre la premiere verification et l'insertion trouvait `seen_by` vide, son `remove_player` ne faisait rien et le spawn partait quand meme. Tout chemin qui (re)cree une entite chez un observateur passe par ce point (ajout d'entite, changement de chunk, respawn, repairage apres changement de monde, passage en spectateur, teleportation).
- Pourquoi : Loup-Garou voyait 7 `spawn_entity` de joueurs caches chez un mort passe spectateur et teleporte.
- Fichiers : `crates/pumpkin-core/src/world/entity_tracker.rs`.
- Limites : la lecture du code montre que tous les chemins de spawn de joueur passaient deja par le filtre ; la fuite retrouvee est cette course (le passage en spectateur concentre les evenements dans le meme tick). Non reproduite en banc, pas de test unitaire (aucun joueur factice dans le coeur). La ligne de tab list envoyee a un joueur qui rejoint reste inchangee.
- Proposable upstream : oui.

### Patch 31 : particules avec donnees (`spawn-particle-data`, `spawn-item-particle`)

- Quoi : `particles.wit` gagne `dust-data`, `dust-transition-data`, `block-state-data`, `color-data` et la variante `particle-data` (`dust`, `dust-color-transition`, `block-state`, `color`). `world` gagne `spawn-particle-data(data, pos, offset, max-speed, count) -> result<_, string>` et `spawn-item-particle(item-id, ...) -> result<_, string>`. L'hote encode les options (couleur RGB ou ARGB en entier 32 bits, taille en `f32`, etat de bloc en VarInt) ; `block-state` n'accepte que `block`, `block-marker`, `falling-dust`, `dust-pillar`, `block-crumble` et `color` que `entity-effect`, `tinted-leaves`, `flash` (erreur sinon). `World::spawn_particle_with_data` envoie le paquet.
- Pourquoi : `spawn-particle` envoyait toujours des donnees vides : `dust` (la particule la plus utilisee des montures), `block`, `item` et `entity-effect` etaient invalides. Additif : `spawn-particle` et l'enumeration `particle` ne changent pas, les plugins compiles restent valides.
- Fichiers : `crates/pumpkin-plugin-wit/v0.2/particles.wit`, `world.wit`, `crates/pumpkin-wasm-host-v0_2/src/world.rs` (encodeur pur + test), `crates/pumpkin-core/src/world/mod.rs`.
- Limites : version par monde seulement (pas de variante par joueur) ; `item` sans composants (une pile de 1) ; `effect`, `instant-effect`, `trail`, `vibration`, `sculk-charge`, `shriek` restent non envoyables avec donnees.
- Proposable upstream : oui.

### Patch 32 : `count-item` accepte `minecraft:`

- Quoi : `Inventory::count_item` reutilise `item_id_matches` (comme `remove-by-id`) : l'identifiant est accepte avec ou sans espace de noms.
- Pourquoi : `count-item("minecraft:iron_ingot")` rendait 0 (BedWars).
- Fichiers : `crates/pumpkin-wasm-host-v0_2/src/inventory.rs` (+ test).
- Limites : aucune.
- Proposable upstream : oui.

### Patch 33 : chaine de degats 1.8 (L4)

- Quoi : en mode `legacy_1_8`, l'armure reduit de 4 % par point (`d x (25 - points)/25`, sans robustesse), la Protection se calcule comme en 1.8 (points `floor((6 + niv^2)/3 x k)` par piece : Protection 0,75, Feu 1,25 contre le feu, Chute 2,5 contre les chutes, Explosion 1,5, Projectile 1,5 ; somme plafonnee a 25 puis tirage `ceil(s/2) + rand[0, floor(s/2)]` plafonne a 20 ; l'apercu envoye aux plugins prend la moyenne, sans tirer) et l'epuisement par coup recu vaut 0,3 (0 pour les sources qui ignorent l'armure). Fonctions pures dans `combat_legacy.rs` (`armor_reduced`, `resistance_reduced`, `protection_points`, `protection_roll`, `protection_mean`, `protection_reduced`, `block_reduced`, `armor_wear`, `invulnerability`), testees contre D5, D6, D7, D8, D9 et P5.
- Pourquoi : les kits d'Arena portent des armures en diamant ; avec la robustesse moderne le diamant reduit bien plus qu'en 1.8.
- Fichiers : `crates/pumpkin-core/src/entity/combat_legacy.rs`, `entity/living.rs` (branches dans `get_damage_after_armor_absorb`, `magic_reduced_damage`, epuisement).
- Limites : la fenetre d'invulnerabilite du moderne (compteur > 10, seul le surplus passe, brut memorise apres le casque) est deja la regle 1.8 : inchangee ; l'usure `max(1, floor(d/4))` est deja celle du moderne ; `block_reduced` et `invulnerability` sont prets pour le blocage a l'epee (L5, non fait : composants `consumable`/`attack_range` et utilisation de 72000 ticks restent a faire) ; les sources « sans armure » suivent la balise `bypasses_armor` moderne ; la Resistance garde le code moderne (meme formule).
- Propre au fork (jamais propose upstream).

## Journal des mises a jour

| Date | Commits upstream integres | Conflits | Duree | Build |
|------|---------------------------|----------|-------|-------|
| 2026-11-10 | base initiale (`eda69440e`) | - | - | OK en local (~28 min) |
| 2026-11-10 | essai à blanc de `update.sh` (clone jetable, non poussé) : 16 commits upstream jusqu'à `d393d3581` | aucun | 26 min 16 s | OK ; `Cargo.lock` modifié par le build (restaurer avant le prochain `update.sh`, qui exige un arbre propre) |
| 2026-11-16 | `upstream/master` @ `2285efe83` (89 commits depuis `eda69440e`) + tete de la PR #3962 `cc9102bd2` ; avance du 25/11 (decision de Bastien) | 4 : `v0.2/world.wit` + `wasm-host-v0_2/src/world.rs` (upstream a ajoute le meme `get-loaded-chunk-count` : version upstream gardee) ; `pumpkin-core/src/world/mod.rs` (patch 3 reinsere apres la reorganisation de `spawn_java_player`, #3708) ; `Cargo.lock` (`pumpkin-plugin-helpers` retire par la PR #3962) | verification 14 min 17 s (debug, sans LTO) | clippy + tests `pumpkin-wasm-host-v0_2` (19) OK ; `cargo check -p pumpkin --features plugin-v0_2` OK ; gapi-rs : clippy wasm OK contre le nouveau SDK |
| 2026-11-19 | aucun (patch 8 seul, sans fusion upstream) | aucun | - | tests `pumpkin-wasm-host-v0_2` (son) OK |
| 2026-11-19 | aucun (patch 9 seul, sans fusion upstream) | aucun | - | tests `pumpkin-protocol` OK |
| 2026-11-19 | aucun (patch 10 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-19 | aucun (patch 4 : ajout de `memory-working-set`, sans fusion upstream) | aucun | - | tests `pumpkin-wasm-host-v0_2` OK |
| 2026-11-19 | aucun (patch 11 seul, sans fusion upstream) | aucun | - | `cargo check` des crates touchees OK |
| 2026-11-20 | aucun (patch 17 seul, sans fusion upstream) | aucun | - | `cargo clippy -p pumpkin-wasm-host` OK |
| 2026-11-20 | aucun (patch 12 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 13 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 14 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 15 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 16 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 18 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 19 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 20 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 21 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 22 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 23 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 24 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 25 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 26 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` OK |
| 2026-11-20 | aucun (patch 27 seul, sans fusion upstream) | aucun | - | tests `pumpkin-config` OK |
| 2026-11-20 | aucun (patch 28 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` `combat_legacy` OK |
| 2026-11-20 | aucun (patch 29 seul, sans fusion upstream) | aucun | - | clippy + tests `pumpkin-core` `combat_legacy` OK |
| 2026-11-20 | aucun (patch 30 seul, sans fusion upstream) | aucun | - | tests `pumpkin-core` `entity_tracker` OK |
| 2026-11-20 | aucun (patch 31 seul, sans fusion upstream) | aucun | - | tests `pumpkin-wasm-host-v0_2` OK |
| 2026-11-20 | aucun (patch 32 seul, sans fusion upstream) | aucun | - | tests `pumpkin-wasm-host-v0_2` OK |
| 2026-11-20 | aucun (patch 33 seul, sans fusion upstream) | aucun | - | clippy + tests `pumpkin-core` `combat_legacy` OK |
