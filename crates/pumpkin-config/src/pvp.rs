use serde::{Deserialize, Serialize};

/// Combat rules used for melee, knockback and sprinting.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CombatMode {
    /// Current vanilla rules (attack cooldown, modern knockback).
    #[default]
    #[serde(rename = "modern")]
    Modern,
    /// Minecraft 1.8 combat: no attack cooldown, 1.8 damage table and knockback.
    #[serde(rename = "legacy_1_8")]
    Legacy1_8,
}

impl CombatMode {
    /// Whether the 1.8 combat rules are active.
    #[must_use]
    pub const fn is_legacy(self) -> bool {
        matches!(self, Self::Legacy1_8)
    }
}

/// Configuration for player-versus-player mechanics.
///
/// Controls whether PVP is enabled, combat effects, and player protections.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct PVPConfig {
    /// Whether PVP is enabled on the server.
    pub enabled: bool,
    /// Whether to show the red hurt animation and FOV bobbing when hit.
    pub hurt_animation: bool,
    /// Whether players in creative mode are protected from PVP.
    pub protect_creative: bool,
    /// Whether knockback from attacks is applied.
    pub knockback: bool,
    /// Whether players swing their hand when attacking.
    pub swing: bool,
    /// Combat ruleset: `"modern"` (default) or `"legacy_1_8"`.
    pub combat: CombatMode,
}

impl Default for PVPConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hurt_animation: true,
            protect_creative: true,
            knockback: true,
            swing: true,
            combat: CombatMode::Modern,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combat_defaults_to_modern() {
        assert_eq!(PVPConfig::default().combat, CombatMode::Modern);
        let parsed: PVPConfig = toml::from_str("enabled = false").unwrap();
        assert_eq!(parsed.combat, CombatMode::Modern);
    }

    #[test]
    fn combat_legacy_1_8_parses_and_round_trips() {
        let parsed: PVPConfig = toml::from_str("combat = \"legacy_1_8\"").unwrap();
        assert!(parsed.combat.is_legacy());
        assert!(toml::to_string(&parsed).unwrap().contains("combat = \"legacy_1_8\""));
    }
}
