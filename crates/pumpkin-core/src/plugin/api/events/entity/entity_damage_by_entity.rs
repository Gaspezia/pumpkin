use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity takes damage from another entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDamageByEntityEvent {
    /// The ID of the entity taking damage.
    pub entity_id: i32,

    /// The ID of the damaging entity.
    pub damager_id: i32,

    /// The amount of damage dealt.
    pub damage: f32,

    /// The damage cause.
    pub cause: String,

    /// The damage that would be taken off the victim's health once armor, resistance,
    /// protection enchantments and absorption are applied (read-only).
    pub final_damage: f32,

    /// The victim's health when the hit lands.
    pub health: f32,
}
