use pumpkin_data::damage::DamageType;
use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity takes damage.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDamageEvent {
    /// The ID of the entity taking damage.
    pub entity_id: i32,

    /// The amount of damage taken.
    pub damage: f32,

    /// The damage type.
    pub damage_type: DamageType,

    /// The damage that would be taken off the victim's health once armor, resistance,
    /// protection enchantments and absorption are applied (read-only: writing it back
    /// changes nothing, change `damage` instead).
    pub final_damage: f32,

    /// The victim's health when the hit lands (0 for entities that are not living).
    pub health: f32,
}

impl EntityDamageEvent {
    #[must_use]
    pub const fn new(entity_id: i32, damage_type: DamageType, damage_amount: f32) -> Self {
        Self {
            entity_id,
            damage: damage_amount,
            damage_type,
            final_damage: damage_amount,
            health: 0.0,
            cancelled: false,
        }
    }

    /// Sets the final damage and the victim's health (see the field docs).
    #[must_use]
    pub const fn with_final_damage(mut self, final_damage: f32, health: f32) -> Self {
        self.final_damage = final_damage;
        self.health = health;
        self
    }
}
