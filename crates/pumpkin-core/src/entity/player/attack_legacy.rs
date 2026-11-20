//! Melee attack under the 1.8 combat rules (`[pvp] combat = "legacy_1_8"`).
//!
//! Formulas live in [`crate::entity::combat_legacy`]; this file only gathers the inputs from the
//! player and plays the hit in the packet order of a 1.8 server. Source of the rules: the
//! 1.8 `EntityPlayer.attackTargetEntityWithCurrentItem` (see `gaspezia/PATCHES.md`, patches 28 and 29).

use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::Enchantment;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{AttributeModifiersImpl, EnchantmentsImpl, Operation};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{Animation, CEntityAnimation};

use super::Player;
use crate::entity::EntityBase;
use crate::entity::combat;
use crate::entity::combat_legacy;

/// Id of the modifier carrying an item's attack damage.
const BASE_ATTACK_DAMAGE_ID: &str = "minecraft:base_attack_damage";

/// Attack damage attribute of the player, with the 1.8 value of the held tool when the item
/// still has its vanilla attribute modifiers (a plugin that redefines them keeps control).
fn attack_attribute(player: &Player, stack: &ItemStack) -> f64 {
    let mut attribute = player
        .living_entity
        .get_attribute_value(&Attributes::ATTACK_DAMAGE);
    let redefined = stack
        .patch
        .iter()
        .any(|(id, _)| *id == DataComponent::AttributeModifiers);
    if !redefined && let Some(legacy) = combat_legacy::weapon_bonus(stack.item) {
        let modern: f64 = stack
            .get_data_component::<AttributeModifiersImpl>()
            .into_iter()
            .flat_map(|component| component.attribute_modifiers.iter())
            .filter(|modifier| {
                modifier.r#type == &Attributes::ATTACK_DAMAGE
                    && modifier.operation == Operation::AddValue
                    && modifier.id == BASE_ATTACK_DAMAGE_ID
            })
            .map(|modifier| modifier.amount)
            .sum();
        attribute += legacy - modern;
    }
    let strength = player
        .living_entity
        .get_effect(&StatusEffect::STRENGTH)
        .map(|effect| i32::from(effect.amplifier));
    let weakness = player
        .living_entity
        .get_effect(&StatusEffect::WEAKNESS)
        .map(|effect| i32::from(effect.amplifier));
    combat_legacy::damage_with_effects(attribute, strength, weakness)
}

impl Player {
    /// The 1.8 melee hit: no cooldown, no sweep, no attack sound, table of 1.8 weapons.
    pub(super) fn attack_legacy(&self, victim: &Arc<dyn EntityBase>) {
        let world = self.world();
        let victim_entity = victim.get_entity();

        let item_stack = self.inventory().held_item();
        if !item_stack.is_empty() {
            self.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Used,
                item_stack.item.id as i32,
                1,
            );
        }
        self.last_attacked_ticks.store(0, Ordering::Relaxed);

        if self.deflect_projectile(victim.as_ref()) {
            return;
        }

        let victim_type = victim_entity.entity_type;
        let (mut sharpness, mut smite, mut bane, mut knockback_level) = (0, 0, 0, 0);
        if let Some(enchantments) = item_stack.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                if **enchantment == Enchantment::SHARPNESS {
                    sharpness = *level;
                } else if **enchantment == Enchantment::KNOCKBACK {
                    knockback_level = *level;
                } else if **enchantment == Enchantment::SMITE
                    && victim_type.has_tag(&tag::EntityType::MINECRAFT_SENSITIVE_TO_SMITE)
                {
                    smite = *level;
                } else if **enchantment == Enchantment::BANE_OF_ARTHROPODS
                    && victim_type
                        .has_tag(&tag::EntityType::MINECRAFT_SENSITIVE_TO_BANE_OF_ARTHROPODS)
                {
                    bane = *level;
                }
            }
        }
        let enchantment_damage = combat_legacy::enchantment_damage(sharpness, smite, bane);

        let attacker_entity = &self.living_entity.entity;
        let critical = combat_legacy::is_critical(
            self.living_entity.fall_distance.load(),
            attacker_entity.on_ground.load(Ordering::Relaxed),
            self.living_entity.on_climbable(),
            attacker_entity.is_in_water(),
            self.living_entity.has_effect(&StatusEffect::BLINDNESS),
            attacker_entity.has_vehicle(),
            victim.get_living_entity().is_some(),
        );
        let attribute = attack_attribute(self, &item_stack);
        let damage = combat_legacy::melee_damage(attribute, critical, enchantment_damage);
        // The hit only leaves when it carries damage (a 0 attribute with no enchantment does not).
        if damage <= 0.0 {
            return;
        }

        let old_movement = victim_entity.velocity.load();
        if !victim.damage_with_context(
            victim.as_ref(),
            damage,
            item_stack
                .get_data_component::<pumpkin_data::data_component_impl::DamageTypeImpl>()
                .map_or(DamageType::PLAYER_ATTACK, |dt| dt.damage_type),
            None,
            Some(self),
            Some(self),
        ) {
            return;
        }

        crate::enchantment::EnchantmentHelper::on_post_attack(self, victim_entity, &item_stack);

        // The extra push needs an accepted hit, a partial one during the invulnerability
        // included (then nothing is sent and the server-side motion keeps the push).
        let level = knockback_level + i32::from(attacker_entity.is_sprinting());
        let knockback_enabled = world
            .server
            .upgrade()
            .is_none_or(|server| server.advanced_config.pvp.knockback);
        let extra_knockback = level > 0 && knockback_enabled;
        if extra_knockback {
            self.push_victim(victim.as_ref(), level, damage);
        }

        // 1.8 order: the victim's velocity first, then the critical animations.
        if victim.get_player().is_some() && victim_entity.sync_velocity.load(Ordering::SeqCst) {
            let clamped = combat_legacy::clamp_packet_velocity(victim_entity.velocity.load());
            victim_entity.velocity.store(clamped);
        }
        combat::sync_hit_player_velocity(victim.as_ref(), old_movement);
        if extra_knockback && attacker_entity.is_sprinting() {
            // The server stops the attacker's sprint after the hit: a new sprint needs a new
            // START_SPRINTING from the client (the W-tap of 1.8).
            self.set_sprinting(false);
        }

        if critical {
            Self::broadcast_hit_animation(&world, victim_entity.entity_id, true);
        }
        if enchantment_damage > 0.0 {
            Self::broadcast_hit_animation(&world, victim_entity.entity_id, false);
        }

        self.living_entity
            .last_attacking_id
            .store(victim_entity.entity_id, Ordering::Relaxed);
        self.living_entity.last_attack_time.store(
            attacker_entity.tick_count.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );

        let durability_cost = Self::combat_weapon_durability_cost(&item_stack);
        self.damage_held_item(durability_cost);
        // 1.8 exhaustion of a landed hit.
        self.add_exhaustion(0.3);
    }

    /// Extra knockback of an accepted hit (Knockback enchantment, +1 when sprinting): along the
    /// attacker's yaw, `+0.1` vertical, no resistance. The attacker's server-side motion drops
    /// to 60 % horizontally, as in 1.8.
    fn push_victim(&self, victim: &dyn EntityBase, level: i32, damage: f32) {
        let attacker = &self.living_entity.entity;
        let victim_entity = victim.get_entity();
        let yaw = attacker.yaw.load();
        let strength = f64::from(level) * 0.5;
        let handled = victim.get_mob().is_some_and(|mob| {
            mob.mob_knockback(
                strength,
                f64::from(yaw.to_radians().sin()),
                f64::from(-yaw.to_radians().cos()),
                Some(attacker),
                damage,
                false,
            )
        });
        if !handled {
            let current = victim_entity.velocity.load();
            let target = combat_legacy::extra_knockback(current, yaw, level);
            if let Some(knockback) =
                victim_entity.fire_knockback_events(strength, target - current, Some(attacker))
            {
                victim_entity.velocity.store(current + knockback);
            }
        }
        let velocity = attacker.velocity.load();
        attacker.velocity.store(velocity.multiply(0.6, 1.0, 0.6));
    }

    /// 1.8 jump of the server-side motion (the "ghost" that the victim's knockback halves): the
    /// vertical impulse, plus 0.2 along the yaw when sprinting.
    pub(super) fn jump_legacy(&self) {
        let entity = &self.living_entity.entity;
        let boost = self
            .living_entity
            .get_effect(&StatusEffect::JUMP_BOOST)
            .map(|effect| i32::from(effect.amplifier));
        entity.velocity.store(combat_legacy::jump_motion(
            entity.velocity.load(),
            boost,
            entity.is_sprinting(),
            entity.yaw.load(),
        ));
    }

    /// Pushes the base attack speed to its maximum so the client cooldown stays full and its
    /// attack indicator disappears (the server ignores the cooldown in this mode anyway).
    pub(super) fn hide_attack_gauge(&self) {
        let max = Attributes::ATTACK_SPEED.max_value;
        let living = &self.living_entity;
        if (living.get_attribute_base(&Attributes::ATTACK_SPEED) - max).abs() < f64::EPSILON {
            return;
        }
        living.set_attribute_base(&Attributes::ATTACK_SPEED, max);
        crate::entity::attributes::send_attribute_updates_for_living(
            living,
            vec![Attributes::ATTACK_SPEED],
        );
    }

    /// Critical (`physical`) or enchanted (magic) hit animation, for Java and Bedrock.
    fn broadcast_hit_animation(world: &crate::world::World, entity_id: i32, physical: bool) {
        let (java, bedrock_action) = if physical {
            (
                Animation::CriticalEffect,
                pumpkin_protocol::bedrock::server::animate::AnimateAction::CriticalHit,
            )
        } else {
            (
                Animation::MagicCriticaleffect,
                pumpkin_protocol::bedrock::server::animate::AnimateAction::MagicCriticalHit,
            )
        };
        let java_packet = CEntityAnimation::new(entity_id.into(), java);
        let bedrock_packet = pumpkin_protocol::bedrock::server::animate::SAnimate {
            action: bedrock_action,
            target_actor_runtime_id: VarULong(entity_id as u64),
            data: 0.0,
            swing_source: None,
            hand: 0,
        };
        world.broadcast_editioned(&java_packet, &bedrock_packet);
    }
}
