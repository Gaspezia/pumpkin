use rustc_hash::FxHashMap;

use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use pumpkin_protocol::{
    BClientPacket, ClientPacket, NumberFormat,
    bedrock::client::{
        remove_objective::CRemoveObjective as BRemoveObjective,
        set_display_objective::CSetDisplayObjective as BSetDisplayObjective,
        set_score::{CSetScore as BSetScore, ScoreEntry as BScoreEntry},
    },
    codec::var_int::VarInt,
    java::client::play::{
        CDisplayObjective, CResetScore, CSetPlayerTeam, CUpdateObjectives, CUpdateScore, Mode,
        RenderType, TeamMethod, TeamParameters,
    },
};
use pumpkin_util::text::{TextComponent, color::NamedColor};
use tracing::warn;

use super::World;
use crate::entity::player::Player;

pub trait ScoreboardTarget: Send + Sync {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    );
    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J);

    /// Sends an objective packet. Real clients never get a second `Add` for a
    /// name they already know (it would be a client-side protocol error).
    fn send_objective<B: BClientPacket + Sync>(
        &self,
        je_packet: &CUpdateObjectives,
        be_packet: &B,
    ) {
        self.send_editioned(je_packet, be_packet);
    }

    /// Same as [`Self::send_objective`] for team packets.
    fn send_team(&self, packet: &CSetPlayerTeam<'_>) {
        self.send_je(packet);
    }
}

/// Objective and team names a client has already been told about.
///
/// Vanilla never sends the creation of the same objective/team twice to a
/// client; a duplicate makes the client throw and the player gets kicked.
#[derive(Debug, Default)]
pub struct Announced {
    objectives: std::collections::HashSet<String>,
    teams: std::collections::HashSet<String>,
}

impl Announced {
    /// Returns the objective mode to really send: a repeated `Add` becomes an `Update`.
    pub fn objective_mode(&mut self, name: &str, mode: u8) -> u8 {
        if mode == Mode::Add as u8 {
            if !self.objectives.insert(name.to_string()) {
                return Mode::Update as u8;
            }
        } else if mode == Mode::Remove as u8 {
            self.objectives.remove(name);
        }
        mode
    }

    /// Returns the team method to really send: a repeated `Create` becomes an `Update`.
    pub fn team_method(&mut self, name: &str, method: TeamMethod) -> TeamMethod {
        match method {
            TeamMethod::Create if !self.teams.insert(name.to_string()) => TeamMethod::Update,
            TeamMethod::Remove => {
                self.teams.remove(name);
                method
            }
            _ => method,
        }
    }
}

fn send_objective_to_player<B: BClientPacket + Sync>(
    player: &Player,
    je: &CUpdateObjectives,
    be: &B,
) {
    let mode = player
        .scoreboard_announced
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .objective_mode(&je.objective_name, je.mode);
    if mode == je.mode {
        player.try_enqueue_packet_editioned(je, be);
    } else {
        let fixed = CUpdateObjectives {
            objective_name: je.objective_name.clone(),
            mode,
            display_name: je.display_name.clone(),
            render_type: je.render_type,
            number_format: je.number_format.clone(),
        };
        player.try_enqueue_packet_editioned(&fixed, be);
    }
}

fn send_team_to_player(player: &Player, packet: &CSetPlayerTeam<'_>) {
    let method = player
        .scoreboard_announced
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .team_method(&packet.team_name, packet.method);
    if method == packet.method {
        player.try_send_client_packet(packet);
        return;
    }
    // Duplicate `Create`: update the options, then re-add the members.
    player.try_send_client_packet(&CSetPlayerTeam {
        team_name: packet.team_name.clone(),
        method,
        parameters: packet.parameters.clone(),
        players: Box::new([]),
    });
    if !packet.players.is_empty() {
        player.try_send_client_packet(&CSetPlayerTeam {
            team_name: packet.team_name.clone(),
            method: TeamMethod::AddPlayers,
            parameters: None,
            players: packet.players.clone(),
        });
    }
}

impl ScoreboardTarget for World {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        self.broadcast_editioned(je_packet, be_packet);
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        self.broadcast_packet_all(je_packet);
    }

    fn send_objective<B: BClientPacket + Sync>(
        &self,
        je_packet: &CUpdateObjectives,
        be_packet: &B,
    ) {
        for player in self.players.load().iter() {
            send_objective_to_player(player, je_packet, be_packet);
        }
    }

    fn send_team(&self, packet: &CSetPlayerTeam<'_>) {
        for player in self.players.load().iter() {
            send_team_to_player(player, packet);
        }
    }
}

impl<T: ScoreboardTarget + ?Sized> ScoreboardTarget for &T {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        (*self).send_editioned(je_packet, be_packet);
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        (*self).send_je(je_packet);
    }

    fn send_objective<B: BClientPacket + Sync>(
        &self,
        je_packet: &CUpdateObjectives,
        be_packet: &B,
    ) {
        (**self).send_objective(je_packet, be_packet);
    }

    fn send_team(&self, packet: &CSetPlayerTeam<'_>) {
        (**self).send_team(packet);
    }
}

impl ScoreboardTarget for std::sync::Arc<World> {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        self.broadcast_editioned(je_packet, be_packet);
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        self.broadcast_packet_all(je_packet);
    }

    fn send_objective<B: BClientPacket + Sync>(
        &self,
        je_packet: &CUpdateObjectives,
        be_packet: &B,
    ) {
        (**self).send_objective(je_packet, be_packet);
    }

    fn send_team(&self, packet: &CSetPlayerTeam<'_>) {
        (**self).send_team(packet);
    }
}

impl ScoreboardTarget for Player {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        self.try_enqueue_packet_editioned(je_packet, be_packet);
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        self.try_send_client_packet(je_packet);
    }

    fn send_objective<B: BClientPacket + Sync>(
        &self,
        je_packet: &CUpdateObjectives,
        be_packet: &B,
    ) {
        send_objective_to_player(self, je_packet, be_packet);
    }

    fn send_team(&self, packet: &CSetPlayerTeam<'_>) {
        send_team_to_player(self, packet);
    }
}

impl ScoreboardTarget for std::sync::Arc<Player> {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        je_packet: &J,
        be_packet: &B,
    ) {
        self.try_enqueue_packet_editioned(je_packet, be_packet);
    }

    fn send_je<J: ClientPacket + Sync>(&self, je_packet: &J) {
        self.try_send_client_packet(je_packet);
    }

    fn send_objective<B: BClientPacket + Sync>(
        &self,
        je_packet: &CUpdateObjectives,
        be_packet: &B,
    ) {
        send_objective_to_player(self, je_packet, be_packet);
    }

    fn send_team(&self, packet: &CSetPlayerTeam<'_>) {
        send_team_to_player(self, packet);
    }
}

pub struct NoTarget;

impl ScoreboardTarget for NoTarget {
    fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(
        &self,
        _je_packet: &J,
        _be_packet: &B,
    ) {
    }

    fn send_je<J: ClientPacket + Sync>(&self, _je_packet: &J) {}
}

#[derive(Clone, Debug, Default)]
pub struct Scoreboard {
    objectives: FxHashMap<String, ScoreboardObjective>,
    display_slots: FxHashMap<ScoreboardDisplaySlot, String>,
    scores: FxHashMap<String, FxHashMap<String, ScoreboardScore>>,
    teams: FxHashMap<String, Team>,
}

impl Scoreboard {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn get_objectives(&self) -> &FxHashMap<String, ScoreboardObjective> {
        &self.objectives
    }

    #[must_use]
    pub fn get_objective(&self, name: &str) -> Option<&ScoreboardObjective> {
        self.objectives.get(name)
    }

    pub fn get_objective_mut(&mut self, name: &str) -> Option<&mut ScoreboardObjective> {
        self.objectives.get_mut(name)
    }

    #[must_use]
    pub const fn get_display_slots(&self) -> &FxHashMap<ScoreboardDisplaySlot, String> {
        &self.display_slots
    }

    #[must_use]
    pub fn get_display_objective(&self, slot: ScoreboardDisplaySlot) -> Option<&str> {
        self.display_slots.get(&slot).map(String::as_str)
    }

    #[must_use]
    pub const fn get_scores(&self) -> &FxHashMap<String, FxHashMap<String, ScoreboardScore>> {
        &self.scores
    }

    #[must_use]
    pub fn get_score(&self, entity_name: &str, objective_name: &str) -> Option<&ScoreboardScore> {
        self.scores.get(objective_name)?.get(entity_name)
    }

    #[must_use]
    pub fn get_score_value(&self, entity_name: &str, objective_name: &str) -> Option<i32> {
        self.get_score(entity_name, objective_name)
            .map(|s| s.value.0)
    }

    #[must_use]
    pub fn get_scores_for_objective(
        &self,
        objective_name: &str,
    ) -> Option<&FxHashMap<String, ScoreboardScore>> {
        self.scores.get(objective_name)
    }

    #[must_use]
    pub fn get_scores_for_entity(&self, entity_name: &str) -> FxHashMap<String, &ScoreboardScore> {
        let mut entity_scores = FxHashMap::default();
        for (obj_name, obj_scores) in &self.scores {
            if let Some(score) = obj_scores.get(entity_name) {
                entity_scores.insert(obj_name.clone(), score);
            }
        }
        entity_scores
    }

    #[must_use]
    pub const fn get_teams(&self) -> &FxHashMap<String, Team> {
        &self.teams
    }

    #[must_use]
    pub fn get_team(&self, name: &str) -> Option<&Team> {
        self.teams.get(name)
    }

    pub fn get_team_mut(&mut self, name: &str) -> Option<&mut Team> {
        self.teams.get_mut(name)
    }

    #[must_use]
    pub fn get_entity_team(&self, entity_name: &str) -> Option<&Team> {
        self.teams
            .values()
            .find(|team| team.players.iter().any(|p| p == entity_name))
    }

    pub fn add_objective(
        &mut self,
        target: &impl ScoreboardTarget,
        objective: ScoreboardObjective,
    ) {
        if self.objectives.contains_key(&objective.name) {
            warn!(
                "Tried to create an objective which already exists: {}",
                &objective.name
            );
            return;
        }

        let je_update = CUpdateObjectives::new(
            objective.name.clone(),
            Mode::Add,
            objective.display_name.clone(),
            objective.render_type,
            objective.number_format.clone(),
        );

        let be_update = BSetDisplayObjective {
            display_slot_name: "sidebar".to_string(), // Default to sidebar
            objective_name: objective.name.clone(),
            objective_display_name: objective.display_name.clone().get_text(),
            criteria_name: "dummy".to_string(),
            sort_order: VarInt(0),
        };

        target.send_objective(&je_update, &be_update);

        self.objectives.insert(objective.name.clone(), objective);
    }

    pub fn update_objective(
        &mut self,
        target: &impl ScoreboardTarget,
        objective: ScoreboardObjective,
    ) {
        if !self.objectives.contains_key(&objective.name) {
            warn!(
                "Tried to update an objective which does not exist: {}",
                &objective.name
            );
            return;
        }

        let je_update = CUpdateObjectives::new(
            objective.name.clone(),
            Mode::Update,
            objective.display_name.clone(),
            objective.render_type,
            objective.number_format.clone(),
        );

        let be_update = BSetDisplayObjective {
            display_slot_name: "sidebar".to_string(),
            objective_name: objective.name.clone(),
            objective_display_name: objective.display_name.clone().get_text(),
            criteria_name: "dummy".to_string(),
            sort_order: VarInt(0),
        };

        target.send_objective(&je_update, &be_update);

        self.objectives.insert(objective.name.clone(), objective);
    }

    pub fn set_display_objective(
        &mut self,
        target: &impl ScoreboardTarget,
        slot: ScoreboardDisplaySlot,
        objective_name: Option<&str>,
    ) {
        let slot_str = match slot {
            ScoreboardDisplaySlot::List => "list",
            ScoreboardDisplaySlot::BelowName => "belowname",
            _ => "sidebar",
        };

        let obj_name_str = objective_name.unwrap_or("");

        let display_name = objective_name
            .and_then(|name| self.objectives.get(name))
            .map_or_else(
                || obj_name_str.to_string(),
                |o| o.display_name.clone().get_text(),
            );

        let je_display = CDisplayObjective::new(slot, obj_name_str.to_string());
        let be_display = BSetDisplayObjective {
            display_slot_name: slot_str.to_string(),
            objective_name: obj_name_str.to_string(),
            objective_display_name: display_name,
            criteria_name: "dummy".to_string(),
            sort_order: VarInt(0),
        };

        target.send_editioned(&je_display, &be_display);

        if let Some(name) = objective_name {
            self.display_slots.insert(slot, name.to_string());
        } else {
            self.display_slots.remove(&slot);
        }
    }

    pub fn clear_display_objective(
        &mut self,
        target: &impl ScoreboardTarget,
        slot: ScoreboardDisplaySlot,
    ) {
        self.set_display_objective(target, slot, None);
    }

    pub fn remove_objective(&mut self, target: &impl ScoreboardTarget, name: &str) {
        if !self.objectives.contains_key(name) {
            warn!(
                "Tried to remove an objective which does not exist: {}",
                name
            );
            return;
        }

        let je_packet = CUpdateObjectives::new(
            name.to_string(),
            Mode::Remove,
            TextComponent::empty(),
            RenderType::Integer,
            None,
        );

        let be_packet = BRemoveObjective {
            objective_name: name.to_string(),
        };

        target.send_objective(&je_packet, &be_packet);

        self.objectives.remove(name);
        self.scores.remove(name);
        self.display_slots.retain(|_, obj| obj != name);
    }

    pub fn update_score(&mut self, target: &impl ScoreboardTarget, score: ScoreboardScore) {
        if !self.objectives.contains_key(&score.objective_name) {
            warn!(
                "Tried to place a score into an objective which does not exist: {}",
                &score.objective_name
            );
            return;
        }

        let je_packet = CUpdateScore::new(
            score.entity_name.clone(),
            score.objective_name.clone(),
            score.value,
            score.display_name.clone(),
            score.number_format.clone(),
        );

        let be_packet = BSetScore {
            action: VarInt(0), // Change
            entries: vec![BScoreEntry {
                scoreboard_id: score.entity_name.as_ptr() as i64, // Internal ID
                objective_name: score.objective_name.clone(),
                score: score.value,
                entry_type: VarInt(3), // Fake player / Literal
                entity_unique_id: 0,
                custom_name: score.entity_name.clone(),
            }],
        };

        target.send_editioned(&je_packet, &be_packet);

        self.scores
            .entry(score.objective_name.clone())
            .or_default()
            .insert(score.entity_name.clone(), score);
    }

    pub fn set_score_value(
        &mut self,
        target: &impl ScoreboardTarget,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        value: i32,
    ) {
        let entity_s = entity_name.into();
        let obj_s = objective_name.into();
        let existing = self.get_score(&entity_s, &obj_s).cloned();
        let score = ScoreboardScore {
            entity_name: entity_s,
            objective_name: obj_s,
            value: VarInt(value),
            display_name: existing.as_ref().and_then(|s| s.display_name.clone()),
            number_format: existing.as_ref().and_then(|s| s.number_format.clone()),
            locked: existing.as_ref().is_none_or(|s| s.locked),
        };
        self.update_score(target, score);
    }

    pub fn add_score(
        &mut self,
        target: &impl ScoreboardTarget,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        delta: i32,
    ) -> i32 {
        let entity_s = entity_name.into();
        let obj_s = objective_name.into();
        let current_val = self.get_score_value(&entity_s, &obj_s).unwrap_or(0);
        let new_val = current_val.wrapping_add(delta);
        self.set_score_value(target, entity_s, obj_s, new_val);
        new_val
    }

    pub fn remove_score(
        &mut self,
        target: &impl ScoreboardTarget,
        entity_name: &str,
        objective_name: &str,
    ) {
        let je_packet = CResetScore::new(entity_name.to_string(), Some(objective_name.to_string()));

        let be_packet = BSetScore {
            action: VarInt(1), // Remove
            entries: vec![BScoreEntry {
                scoreboard_id: entity_name.as_ptr() as i64,
                objective_name: objective_name.to_string(),
                score: VarInt(0),
                entry_type: VarInt(3),
                entity_unique_id: 0,
                custom_name: entity_name.to_string(),
            }],
        };

        target.send_editioned(&je_packet, &be_packet);

        if let Some(objective_scores) = self.scores.get_mut(objective_name) {
            objective_scores.remove(entity_name);
        }
    }

    pub fn reset_scores_for_entity(&mut self, target: &impl ScoreboardTarget, entity_name: &str) {
        let je_packet = CResetScore::new(entity_name.to_string(), None);

        let mut be_entries = Vec::new();
        for (obj_name, obj_scores) in &self.scores {
            if obj_scores.contains_key(entity_name) {
                be_entries.push(BScoreEntry {
                    scoreboard_id: entity_name.as_ptr() as i64,
                    objective_name: obj_name.clone(),
                    score: VarInt(0),
                    entry_type: VarInt(3),
                    entity_unique_id: 0,
                    custom_name: entity_name.to_string(),
                });
            }
        }

        let be_packet = BSetScore {
            action: VarInt(1), // Remove
            entries: be_entries,
        };

        target.send_editioned(&je_packet, &be_packet);

        for obj_scores in self.scores.values_mut() {
            obj_scores.remove(entity_name);
        }
    }

    pub fn add_team(&mut self, target: &impl ScoreboardTarget, team: Team) {
        if self.teams.contains_key(&team.name) {
            warn!("Tried to create Team which already exists, {}", team.name);
            return;
        }

        let parameters = TeamParameters {
            display_name: &team.display_name,
            options: team.options,
            nametag_visibility: team.nametag_visibility.to_str(),
            collision_rule: team.collision_rule.to_str(),
            color: team.color as i32,
            player_prefix: &team.player_prefix,
            player_suffix: &team.player_suffix,
        };

        target.send_team(&CSetPlayerTeam {
            team_name: team.name.clone(),
            method: TeamMethod::Create,
            parameters: Some(parameters),
            players: team.players.clone().into(),
        });

        self.teams.insert(team.name.clone(), team);
    }

    pub fn create_team(&mut self, target: &impl ScoreboardTarget, team: Team) {
        self.add_team(target, team);
    }

    pub fn update_team(&mut self, target: &impl ScoreboardTarget, mut team: Team) {
        let Some(existing) = self.teams.get_mut(&team.name) else {
            warn!("Tried to update Team which does not exist, {}", team.name);
            return;
        };
        // Like vanilla `PlayerTeam`, an update only changes options: members stay.
        team.players = std::mem::take(&mut existing.players);

        let parameters = TeamParameters {
            display_name: &team.display_name,
            options: team.options,
            nametag_visibility: team.nametag_visibility.to_str(),
            collision_rule: team.collision_rule.to_str(),
            color: team.color as i32,
            player_prefix: &team.player_prefix,
            player_suffix: &team.player_suffix,
        };

        target.send_team(&CSetPlayerTeam {
            team_name: team.name.clone(),
            method: TeamMethod::Update,
            parameters: Some(parameters),
            players: Box::new([]),
        });

        self.teams.insert(team.name.clone(), team);
    }

    pub fn remove_team(&mut self, target: &impl ScoreboardTarget, name: &str) {
        if !self.teams.contains_key(name) {
            warn!("Tried to remove Team which does not exist, {}", name);
            return;
        }

        target.send_team(&CSetPlayerTeam {
            team_name: name.to_string(),
            method: TeamMethod::Remove,
            parameters: None,
            players: Box::new([]),
        });

        self.teams.remove(name);
    }

    pub fn add_player_to_team(
        &mut self,
        target: &impl ScoreboardTarget,
        team_name: &str,
        player: String,
    ) {
        let Some(team) = self.teams.get_mut(team_name) else {
            warn!(
                "Tried to add player to Team which does not exist, {}",
                team_name
            );
            return;
        };

        if team.players.contains(&player) {
            return;
        }

        target.send_je(&CSetPlayerTeam {
            team_name: team_name.to_string(),
            method: TeamMethod::AddPlayers,
            parameters: None,
            players: vec![player.clone()].into(),
        });

        team.players.push(player);
    }

    pub fn remove_player_from_team(
        &mut self,
        target: &impl ScoreboardTarget,
        team_name: &str,
        player: &str,
    ) {
        let Some(team) = self.teams.get_mut(team_name) else {
            warn!(
                "Tried to remove player from Team which does not exist, {}",
                team_name
            );
            return;
        };

        if !team.players.contains(&player.to_string()) {
            return;
        }

        target.send_je(&CSetPlayerTeam {
            team_name: team_name.to_string(),
            method: TeamMethod::RemovePlayers,
            parameters: None,
            players: vec![player.to_string()].into(),
        });

        team.players.retain(|p| p != player);
    }

    pub fn clear_team_players(&mut self, target: &impl ScoreboardTarget, team_name: &str) {
        let Some(team) = self.teams.get_mut(team_name) else {
            warn!(
                "Tried to clear players from Team which does not exist, {}",
                team_name
            );
            return;
        };

        if team.players.is_empty() {
            return;
        }

        let players_to_remove = team.players.clone();
        target.send_je(&CSetPlayerTeam {
            team_name: team_name.to_string(),
            method: TeamMethod::RemovePlayers,
            parameters: None,
            players: players_to_remove.into(),
        });

        team.players.clear();
    }

    pub fn send_to_player(&self, player: &impl ScoreboardTarget) {
        for objective in self.objectives.values() {
            let je_update = CUpdateObjectives::new(
                objective.name.clone(),
                Mode::Add,
                objective.display_name.clone(),
                objective.render_type,
                objective.number_format.clone(),
            );
            let be_update = BSetDisplayObjective {
                display_slot_name: "sidebar".to_string(),
                objective_name: objective.name.clone(),
                objective_display_name: objective.display_name.clone().get_text(),
                criteria_name: "dummy".to_string(),
                sort_order: VarInt(0),
            };
            player.send_objective(&je_update, &be_update);
        }

        for (slot, objective_name) in &self.display_slots {
            let slot_str = match slot {
                ScoreboardDisplaySlot::List => "list",
                ScoreboardDisplaySlot::BelowName => "belowname",
                _ => "sidebar",
            };
            let display_name = self.objectives.get(objective_name).map_or_else(
                || objective_name.clone(),
                |o| o.display_name.clone().get_text(),
            );
            let je_display = CDisplayObjective::new(*slot, objective_name.clone());
            let be_display = BSetDisplayObjective {
                display_slot_name: slot_str.to_string(),
                objective_name: objective_name.clone(),
                objective_display_name: display_name,
                criteria_name: "dummy".to_string(),
                sort_order: VarInt(0),
            };
            player.send_editioned(&je_display, &be_display);
        }

        for objective_scores in self.scores.values() {
            for score in objective_scores.values() {
                let je_packet = CUpdateScore::new(
                    score.entity_name.clone(),
                    score.objective_name.clone(),
                    score.value,
                    score.display_name.clone(),
                    score.number_format.clone(),
                );
                let be_packet = BSetScore {
                    action: VarInt(0),
                    entries: vec![BScoreEntry {
                        scoreboard_id: score.entity_name.as_ptr() as i64,
                        objective_name: score.objective_name.clone(),
                        score: score.value,
                        entry_type: VarInt(3),
                        entity_unique_id: 0,
                        custom_name: score.entity_name.clone(),
                    }],
                };
                player.send_editioned(&je_packet, &be_packet);
            }
        }

        for team in self.teams.values() {
            let parameters = TeamParameters {
                display_name: &team.display_name,
                options: team.options,
                nametag_visibility: team.nametag_visibility.to_str(),
                collision_rule: team.collision_rule.to_str(),
                color: team.color as i32,
                player_prefix: &team.player_prefix,
                player_suffix: &team.player_suffix,
            };
            let je_packet = CSetPlayerTeam {
                team_name: team.name.clone(),
                method: TeamMethod::Create,
                parameters: Some(parameters),
                players: team.players.clone().into(),
            };
            player.send_team(&je_packet);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreboardObjective {
    pub name: String,
    pub display_name: TextComponent,
    pub render_type: RenderType,
    pub number_format: Option<NumberFormat>,
    pub criterion: String,
}

impl ScoreboardObjective {
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        display_name: TextComponent,
        render_type: RenderType,
        number_format: Option<NumberFormat>,
        criterion: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            display_name,
            render_type,
            number_format,
            criterion: criterion.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScoreboardScore {
    pub entity_name: String,
    pub objective_name: String,
    pub value: VarInt,
    pub display_name: Option<TextComponent>,
    pub number_format: Option<NumberFormat>,
    pub locked: bool,
}

impl ScoreboardScore {
    #[must_use]
    pub fn new(
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        value: VarInt,
        display_name: Option<TextComponent>,
        number_format: Option<NumberFormat>,
    ) -> Self {
        Self {
            entity_name: entity_name.into(),
            objective_name: objective_name.into(),
            value,
            display_name,
            number_format,
            locked: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameTagVisibility {
    Always,
    Never,
    HideForOtherTeams,
    HideForOwnTeam,
}

impl NameTagVisibility {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::HideForOtherTeams => "hideForOtherTeams",
            Self::HideForOwnTeam => "hideForOwnTeam",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionRule {
    Always,
    Never,
    PushOtherTeams,
    PushOwnTeam,
}

impl CollisionRule {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::PushOtherTeams => "pushOtherTeams",
            Self::PushOwnTeam => "pushOwnTeam",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Team {
    pub name: String,
    pub display_name: TextComponent,
    pub options: i8,
    pub nametag_visibility: NameTagVisibility,
    pub collision_rule: CollisionRule,
    pub color: NamedColor,
    pub player_prefix: TextComponent,
    pub player_suffix: TextComponent,
    pub players: Vec<String>,
}

impl Team {
    pub(crate) const ALLOW_FRIENDLY_FIRE: i8 = 0x01;

    #[must_use]
    pub const fn is_allow_friendly_fire(&self) -> bool {
        self.options & Self::ALLOW_FRIENDLY_FIRE != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BedrockSortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BedrockDisplaySlot {
    PlayerList,
    Sidebar,
    BelowName,
}

impl BedrockDisplaySlot {
    #[must_use]
    pub const fn to_str(&self) -> &'static str {
        match self {
            Self::PlayerList => "list",
            Self::Sidebar => "sidebar",
            Self::BelowName => "belowname",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BedrockObjective {
    pub name: String,
    pub display_name: String,
    pub sort_order: BedrockSortOrder,
}

#[derive(Clone, Debug, Default)]
pub struct BedrockScoreboard {
    pub objectives: FxHashMap<String, BedrockObjective>,
    pub display_slots: FxHashMap<BedrockDisplaySlot, String>,
    pub scores: FxHashMap<(String, String), i32>,
}

impl BedrockScoreboard {
    pub fn add_objective(&mut self, player: &Player, objective: BedrockObjective) {
        let be_update = BSetDisplayObjective {
            display_slot_name: "sidebar".to_string(),
            objective_name: objective.name.clone(),
            objective_display_name: objective.display_name.clone(),
            criteria_name: "dummy".to_string(),
            sort_order: VarInt(match objective.sort_order {
                BedrockSortOrder::Ascending => 0,
                BedrockSortOrder::Descending => 1,
            }),
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CUpdateObjectives::new(
                objective.name.clone(),
                pumpkin_protocol::java::client::play::Mode::Add,
                TextComponent::text(objective.display_name.clone()),
                pumpkin_protocol::java::client::play::RenderType::Integer,
                None,
            ),
            &be_update,
        );

        self.objectives.insert(objective.name.clone(), objective);
    }

    pub fn update_objective(&mut self, player: &Player, objective: BedrockObjective) {
        let be_update = BSetDisplayObjective {
            display_slot_name: "sidebar".to_string(),
            objective_name: objective.name.clone(),
            objective_display_name: objective.display_name.clone(),
            criteria_name: "dummy".to_string(),
            sort_order: VarInt(match objective.sort_order {
                BedrockSortOrder::Ascending => 0,
                BedrockSortOrder::Descending => 1,
            }),
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CUpdateObjectives::new(
                objective.name.clone(),
                pumpkin_protocol::java::client::play::Mode::Update,
                TextComponent::text(objective.display_name.clone()),
                pumpkin_protocol::java::client::play::RenderType::Integer,
                None,
            ),
            &be_update,
        );

        self.objectives.insert(objective.name.clone(), objective);
    }

    pub fn remove_objective(&mut self, player: &Player, name: &str) {
        let be_remove = BRemoveObjective {
            objective_name: name.to_string(),
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CUpdateObjectives::new(
                name.to_string(),
                pumpkin_protocol::java::client::play::Mode::Remove,
                TextComponent::text(""),
                pumpkin_protocol::java::client::play::RenderType::Integer,
                None,
            ),
            &be_remove,
        );
        self.objectives.remove(name);
        self.display_slots.retain(|_, v| v != name);
        self.scores.retain(|(_, obj), _| obj != name);
    }

    pub fn set_display_objective(
        &mut self,
        player: &Player,
        slot: BedrockDisplaySlot,
        objective_name: Option<&str>,
    ) {
        let obj_name_str = objective_name.unwrap_or("");
        let display_name = objective_name
            .and_then(|name| self.objectives.get(name))
            .map_or_else(|| obj_name_str.to_string(), |o| o.display_name.clone());

        let be_display = BSetDisplayObjective {
            display_slot_name: slot.to_str().to_string(),
            objective_name: obj_name_str.to_string(),
            objective_display_name: display_name,
            criteria_name: "dummy".to_string(),
            sort_order: VarInt(0),
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CDisplayObjective::new(
                match slot {
                    BedrockDisplaySlot::PlayerList => ScoreboardDisplaySlot::List,
                    BedrockDisplaySlot::Sidebar => ScoreboardDisplaySlot::Sidebar,
                    BedrockDisplaySlot::BelowName => ScoreboardDisplaySlot::BelowName,
                },
                obj_name_str.to_string(),
            ),
            &be_display,
        );

        if let Some(name) = objective_name {
            self.display_slots.insert(slot, name.to_string());
        } else {
            self.display_slots.remove(&slot);
        }
    }

    pub fn clear_display_objective(&mut self, player: &Player, slot: BedrockDisplaySlot) {
        self.set_display_objective(player, slot, None);
    }

    pub fn update_score(
        &mut self,
        player: &Player,
        entity_name: &str,
        objective_name: &str,
        value: i32,
    ) {
        let score = ScoreboardScore::new(
            entity_name.to_string(),
            objective_name.to_string(),
            VarInt(value),
            None,
            None,
        );
        let be_score = BSetScore {
            action: VarInt(0),
            entries: vec![],
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CUpdateScore::new(
                score.entity_name.clone(),
                score.objective_name.clone(),
                score.value,
                score.display_name.clone(),
                score.number_format,
            ),
            &be_score,
        );

        self.scores
            .insert((entity_name.to_string(), objective_name.to_string()), value);
    }

    pub fn add_score(
        &mut self,
        player: &Player,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        delta: i32,
    ) -> i32 {
        let entity_s = entity_name.into();
        let obj_s = objective_name.into();
        let current = self
            .scores
            .get(&(entity_s.clone(), obj_s.clone()))
            .copied()
            .unwrap_or(0);
        let new_val = current + delta;
        self.update_score(player, &entity_s, &obj_s, new_val);
        new_val
    }

    pub fn remove_score(&mut self, player: &Player, entity_name: &str, objective_name: &str) {
        let be_score = BSetScore {
            action: VarInt(1),
            entries: vec![],
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CResetScore::new(
                entity_name.to_string(),
                Some(objective_name.to_string()),
            ),
            &be_score,
        );
        self.scores
            .remove(&(entity_name.to_string(), objective_name.to_string()));
    }

    pub fn reset_scores_for_entity(&mut self, player: &Player, entity_name: &str) {
        let be_score = BSetScore {
            action: VarInt(1),
            entries: vec![],
        };
        player.try_enqueue_packet_editioned(
            &pumpkin_protocol::java::client::play::CResetScore::new(entity_name.to_string(), None),
            &be_score,
        );
        self.scores.retain(|(entity, _), _| entity != entity_name);
    }

    pub fn send_to_player(&self, player: &Player) {
        for objective in self.objectives.values() {
            let be_update = BSetDisplayObjective {
                display_slot_name: "sidebar".to_string(),
                objective_name: objective.name.clone(),
                objective_display_name: objective.display_name.clone(),
                criteria_name: "dummy".to_string(),
                sort_order: VarInt(match objective.sort_order {
                    BedrockSortOrder::Ascending => 0,
                    BedrockSortOrder::Descending => 1,
                }),
            };
            player.try_enqueue_packet_editioned(
                &pumpkin_protocol::java::client::play::CUpdateObjectives::new(
                    objective.name.clone(),
                    pumpkin_protocol::java::client::play::Mode::Add,
                    TextComponent::text(objective.display_name.clone()),
                    pumpkin_protocol::java::client::play::RenderType::Integer,
                    None,
                ),
                &be_update,
            );
        }

        for (slot, objective_name) in &self.display_slots {
            let display_name = self
                .objectives
                .get(objective_name)
                .map_or_else(|| objective_name.clone(), |o| o.display_name.clone());
            let be_display = BSetDisplayObjective {
                display_slot_name: slot.to_str().to_string(),
                objective_name: objective_name.clone(),
                objective_display_name: display_name,
                criteria_name: "dummy".to_string(),
                sort_order: VarInt(0),
            };
            player.try_enqueue_packet_editioned(
                &pumpkin_protocol::java::client::play::CDisplayObjective::new(
                    match slot {
                        BedrockDisplaySlot::PlayerList => ScoreboardDisplaySlot::List,
                        BedrockDisplaySlot::Sidebar => ScoreboardDisplaySlot::Sidebar,
                        BedrockDisplaySlot::BelowName => ScoreboardDisplaySlot::BelowName,
                    },
                    objective_name.clone(),
                ),
                &be_display,
            );
        }

        for ((entity_name, objective_name), value) in &self.scores {
            let score = ScoreboardScore::new(
                entity_name.clone(),
                objective_name.clone(),
                VarInt(*value),
                None,
                None,
            );
            let be_score = BSetScore {
                action: VarInt(0),
                entries: vec![],
            };
            player.try_enqueue_packet_editioned(
                &pumpkin_protocol::java::client::play::CUpdateScore::new(
                    score.entity_name,
                    score.objective_name,
                    score.value,
                    score.display_name,
                    score.number_format,
                ),
                &be_score,
            );
        }
    }
}

#[derive(Default, Debug)]
pub struct ScoreboardBuilder {
    scoreboard: Scoreboard,
}

impl ScoreboardBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn objective(mut self, name: impl Into<String>, display_name: TextComponent) -> Self {
        let obj = ScoreboardObjective::new(
            name.into(),
            display_name,
            RenderType::Integer,
            None,
            "dummy",
        );
        self.scoreboard.objectives.insert(obj.name.clone(), obj);
        self
    }

    #[must_use]
    pub fn objective_with_render(
        mut self,
        name: impl Into<String>,
        display_name: TextComponent,
        render_type: RenderType,
    ) -> Self {
        let obj = ScoreboardObjective::new(name.into(), display_name, render_type, None, "dummy");
        self.scoreboard.objectives.insert(obj.name.clone(), obj);
        self
    }

    #[must_use]
    pub fn display_slot(
        mut self,
        slot: ScoreboardDisplaySlot,
        objective_name: impl Into<String>,
    ) -> Self {
        self.scoreboard
            .display_slots
            .insert(slot, objective_name.into());
        self
    }

    #[must_use]
    pub fn score(
        mut self,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        value: i32,
    ) -> Self {
        let entity_s = entity_name.into();
        let obj_s = objective_name.into();
        let score =
            ScoreboardScore::new(entity_s.clone(), obj_s.clone(), VarInt(value), None, None);
        self.scoreboard
            .scores
            .entry(entity_s)
            .or_default()
            .insert(obj_s, score);
        self
    }

    #[must_use]
    pub fn team(mut self, team: Team) -> Self {
        self.scoreboard.teams.insert(team.name.clone(), team);
        self
    }

    #[must_use]
    pub fn build(self) -> Scoreboard {
        self.scoreboard
    }
}

#[derive(Default, Debug)]
pub struct BedrockScoreboardBuilder {
    scoreboard: BedrockScoreboard,
}

impl BedrockScoreboardBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn objective(
        mut self,
        name: impl Into<String>,
        display_name: impl Into<String>,
        sort_order: BedrockSortOrder,
    ) -> Self {
        let name = name.into();
        let obj = BedrockObjective {
            name: name.clone(),
            display_name: display_name.into(),
            sort_order,
        };
        self.scoreboard.objectives.insert(name, obj);
        self
    }

    #[must_use]
    pub fn display_slot(
        mut self,
        slot: BedrockDisplaySlot,
        objective_name: impl Into<String>,
    ) -> Self {
        self.scoreboard
            .display_slots
            .insert(slot, objective_name.into());
        self
    }

    #[must_use]
    pub fn score(
        mut self,
        entity_name: impl Into<String>,
        objective_name: impl Into<String>,
        value: i32,
    ) -> Self {
        self.scoreboard
            .scores
            .insert((entity_name.into(), objective_name.into()), value);
        self
    }

    #[must_use]
    pub fn build(self) -> BedrockScoreboard {
        self.scoreboard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mimics a client connection: applies the same dedup as `Player`, records what is sent.
    #[derive(Default)]
    struct Client {
        announced: std::sync::Mutex<Announced>,
        sent: std::sync::Mutex<Vec<String>>,
    }

    impl ScoreboardTarget for Client {
        fn send_editioned<J: ClientPacket + Sync, B: BClientPacket + Sync>(&self, _: &J, _: &B) {}
        fn send_je<J: ClientPacket + Sync>(&self, _: &J) {}
        fn send_objective<B: BClientPacket + Sync>(&self, je: &CUpdateObjectives, _: &B) {
            let mode = self
                .announced
                .lock()
                .unwrap()
                .objective_mode(&je.objective_name, je.mode);
            self.sent
                .lock()
                .unwrap()
                .push(format!("obj {} {mode}", je.objective_name));
        }
        fn send_team(&self, p: &CSetPlayerTeam<'_>) {
            let m = self
                .announced
                .lock()
                .unwrap()
                .team_method(&p.team_name, p.method);
            self.sent
                .lock()
                .unwrap()
                .push(format!("team {} {}", p.team_name, m as u8));
        }
    }

    #[test]
    fn plugin_add_then_send_to_player_sends_one_objective_add() {
        let client = Client::default();
        let mut sb = Scoreboard::new();
        sb.add_objective(
            &client,
            ScoreboardObjective::new(
                "sb",
                TextComponent::text("sb"),
                RenderType::Integer,
                None,
                "dummy",
            ),
        );
        sb.send_to_player(&client);
        let sent = client.sent.lock().unwrap().clone();
        assert_eq!(sent, ["obj sb 0", "obj sb 2"]); // Add then Update, never Add twice
    }

    #[test]
    fn plugin_team_then_send_to_player_sends_one_team_create() {
        let client = Client::default();
        let mut sb = Scoreboard::new();
        sb.add_team(&client, team(CollisionRule::Always, 0));
        sb.send_to_player(&client);
        let sent = client.sent.lock().unwrap().clone();
        assert_eq!(sent, ["team t 0", "team t 2"]); // Create then Update
    }

    #[test]
    fn removed_names_can_be_announced_again() {
        let mut a = Announced::default();
        assert_eq!(a.objective_mode("o", 0), 0);
        assert_eq!(a.objective_mode("o", 0), 2);
        assert_eq!(a.objective_mode("o", 1), 1);
        assert_eq!(a.objective_mode("o", 0), 0);
        assert_eq!(a.team_method("t", TeamMethod::Create), TeamMethod::Create);
        assert_eq!(a.team_method("t", TeamMethod::Remove), TeamMethod::Remove);
        assert_eq!(a.team_method("t", TeamMethod::Create), TeamMethod::Create);
    }

    fn team(collision_rule: CollisionRule, options: i8) -> Team {
        Team {
            name: "t".to_string(),
            display_name: TextComponent::text("t"),
            options,
            nametag_visibility: NameTagVisibility::Always,
            collision_rule,
            color: NamedColor::White,
            player_prefix: TextComponent::text(""),
            player_suffix: TextComponent::text(""),
            players: Vec::new(),
        }
    }

    #[test]
    fn update_team_keeps_members_and_applies_options() {
        let mut sb = Scoreboard::new();
        sb.add_team(
            &NoTarget,
            team(CollisionRule::Never, Team::ALLOW_FRIENDLY_FIRE),
        );
        sb.add_player_to_team(&NoTarget, "t", "a".to_string());
        sb.add_player_to_team(&NoTarget, "t", "b".to_string());

        // Incoming update carries no members, like the WASM host builds it.
        sb.update_team(&NoTarget, team(CollisionRule::Always, 0));

        let updated = sb.get_team("t").unwrap();
        assert_eq!(updated.players, vec!["a".to_string(), "b".to_string()]);
        assert!(sb.get_entity_team("a").is_some());
        assert!(matches!(updated.collision_rule, CollisionRule::Always));
        assert!(!updated.is_allow_friendly_fire());
    }
}
