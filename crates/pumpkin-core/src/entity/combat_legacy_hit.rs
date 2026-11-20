//! Base knockback of a landed hit under the 1.8 rules (`[pvp] combat = "legacy_1_8"`).
//!
//! The formula is [`combat_legacy::base_knockback`]; this file picks the direction and the
//! resistance roll and fires the plugin knockback events in the same order as the modern path.

use pumpkin_data::attributes::Attributes;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::combat_legacy;
use crate::entity::living::LivingEntity;
use crate::entity::{Entity, EntityBase};

impl Entity {
    /// Sets the 1.8 base knockback on this entity (`(dx, dz)` points from it to the attacker).
    pub(crate) fn apply_knockback_legacy(&self, dx: f64, dz: f64, attacker: Option<&Self>) {
        let velocity = self.velocity.load();
        let target = combat_legacy::base_knockback(velocity, dx, dz, rand::random::<f64>);
        let Some(knockback) = self.fire_knockback_events(0.4, target - velocity, attacker) else {
            return;
        };
        self.velocity_dirty
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.velocity.store(velocity + knockback);
    }
}

impl LivingEntity {
    /// 1.8 `knockBack` of a landed hit: pushed away from the attacking entity (the shooter for
    /// a projectile), tall enough to relaunch a victim already in the air, and all or nothing
    /// against knockback resistance. A source without an entity pushes nothing.
    pub(crate) fn deal_default_knockback_legacy(
        &self,
        caller: &dyn EntityBase,
        position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
        damage: f32,
    ) {
        let attacker = cause.or(source);
        let Some(origin) = attacker
            .map(|attacker| attacker.get_entity().pos.load())
            .or(position)
        else {
            return;
        };
        let target = self.entity.pos.load();
        let (dx, dz) = (origin.x - target.x, origin.z - target.z);
        if caller.get_mob().is_some_and(|mob| {
            mob.mob_knockback(
                0.4,
                dx,
                dz,
                attacker.map(EntityBase::get_entity),
                damage,
                false,
            )
        }) {
            return;
        }
        let resistance = self.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE);
        if combat_legacy::resists_knockback(resistance, rand::random::<f64>()) {
            return;
        }
        self.entity
            .apply_knockback_legacy(dx, dz, source.map(EntityBase::get_entity));
    }
}
