//! Minecraft 1.8 combat rules (`[pvp] combat = "legacy_1_8"`).
//!
//! Pure functions carrying the 1.8 melee and knockback formulas, so they can be tested against
//! reference vectors. The upstream entry points (`Player::attack`, `deal_default_knockback`,
//! `Player::jump`) only branch to this module.
//!
//! Float widths follow the 1.8 server: damage and the sprint push use `f32`, motion uses `f64`,
//! and the vertical cap and the `0.4F` push keep their `f32` rounding. `sin` and `cos` use the
//! exact `f64` functions rounded to `f32` (1.8 uses a lookup table, within 1e-4 of these).

use pumpkin_data::item::Item;
use pumpkin_util::math::vector3::Vector3;

/// Base push of a landed hit, `0.4F`.
const BASE_PUSH: f64 = 0.400_000_005_960_464_5;
/// Cap of the vertical speed after the base push, `0.4000000059604645`.
const VERTICAL_CAP: f64 = 0.400_000_005_960_464_5;
/// Horizontal push per level of extra knockback (Knockback enchantment, +1 when sprinting).
const EXTRA_HORIZONTAL: f32 = 0.5;
/// Vertical push of the extra knockback, applied once whatever the level.
const EXTRA_VERTICAL: f64 = 0.1;
/// Largest component of a velocity packet in 1.8.
pub const PACKET_VELOCITY_LIMIT: f64 = 3.9;

/// Item attack bonus (on top of the player's base 1.0) in 1.8, for the tools that exist there.
#[must_use]
pub fn weapon_bonus(item: &Item) -> Option<f64> {
    // (sword, axe, pickaxe, shovel) share the tier: wood 0, stone 1, iron 2, diamond 3, gold 0.
    const TIERS: [([&Item; 4], f64); 5] = [
        (
            [
                &Item::WOODEN_SWORD,
                &Item::WOODEN_AXE,
                &Item::WOODEN_PICKAXE,
                &Item::WOODEN_SHOVEL,
            ],
            0.0,
        ),
        (
            [
                &Item::STONE_SWORD,
                &Item::STONE_AXE,
                &Item::STONE_PICKAXE,
                &Item::STONE_SHOVEL,
            ],
            1.0,
        ),
        (
            [
                &Item::IRON_SWORD,
                &Item::IRON_AXE,
                &Item::IRON_PICKAXE,
                &Item::IRON_SHOVEL,
            ],
            2.0,
        ),
        (
            [
                &Item::DIAMOND_SWORD,
                &Item::DIAMOND_AXE,
                &Item::DIAMOND_PICKAXE,
                &Item::DIAMOND_SHOVEL,
            ],
            3.0,
        ),
        (
            [
                &Item::GOLDEN_SWORD,
                &Item::GOLDEN_AXE,
                &Item::GOLDEN_PICKAXE,
                &Item::GOLDEN_SHOVEL,
            ],
            0.0,
        ),
    ];
    const KIND_BASE: [f64; 4] = [4.0, 3.0, 2.0, 1.0];
    for (items, tier) in TIERS {
        for (index, candidate) in items.iter().enumerate() {
            if candidate.id == item.id {
                return Some(KIND_BASE[index] + tier);
            }
        }
    }
    None
}

/// Attack damage attribute after the Strength / Weakness modifiers of 1.8.
///
/// Weakness adds `-0.5 * (amp + 1)` before the multipliers; Strength multiplies the total by
/// `1 + 1.3 * (amp + 1)`. `*_amp` are the amplifiers (0 = level I).
#[must_use]
pub fn damage_with_effects(
    attribute: f64,
    strength_amp: Option<i32>,
    weakness_amp: Option<i32>,
) -> f64 {
    let mut value = attribute;
    if let Some(amp) = weakness_amp {
        value += -0.5 * f64::from(amp + 1);
    }
    if let Some(amp) = strength_amp {
        value *= 1.3f64.mul_add(f64::from(amp + 1), 1.0);
    }
    value
}

/// Enchantment damage of 1.8: Sharpness 1.25 per level, Smite and Bane of Arthropods 2.5 per
/// level against their targets.
#[must_use]
pub fn enchantment_damage(sharpness: i32, smite_vs_undead: i32, bane_vs_arthropod: i32) -> f32 {
    sharpness as f32 * 1.25 + smite_vs_undead as f32 * 2.5 + bane_vs_arthropod as f32 * 2.5
}

/// Conditions of a 1.8 critical hit. The sprint does not matter.
#[must_use]
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "one flag per 1.8 condition, read at the call site"
)]
pub const fn is_critical(
    fall_distance: f32,
    on_ground: bool,
    on_climbable: bool,
    in_water: bool,
    blind: bool,
    riding: bool,
    victim_living: bool,
) -> bool {
    fall_distance > 0.0
        && !on_ground
        && !on_climbable
        && !in_water
        && !blind
        && !riding
        && victim_living
}

/// Damage of one melee hit: the attribute value (x1.5 when critical), then the enchantment
/// damage, which a critical hit does not multiply.
#[must_use]
pub fn melee_damage(attribute: f64, critical: bool, enchantment: f32) -> f32 {
    let mut damage = attribute as f32;
    if critical {
        damage *= 1.5;
    }
    damage + enchantment
}

/// Base knockback of a landed hit (1.8 `knockBack`), without any ground condition.
///
/// `motion` is the server-side motion of the victim, `(dx, dz)` the vector from the victim to
/// the attacker. When the two are nearly on the same spot the direction is drawn with `random`
/// (a value in `[0, 1)`). The caller rolls the knockback resistance beforehand.
pub fn base_knockback(
    motion: Vector3<f64>,
    mut dx: f64,
    mut dz: f64,
    mut random: impl FnMut() -> f64,
) -> Vector3<f64> {
    while dx * dx + dz * dz < 1.0e-4 {
        dx = (random() - random()) * 0.01;
        dz = (random() - random()) * 0.01;
    }
    let norm = f64::from(dx.hypot(dz) as f32);
    Vector3::new(
        motion.x / 2.0 - dx / norm * BASE_PUSH,
        (motion.y / 2.0 + BASE_PUSH).min(VERTICAL_CAP),
        motion.z / 2.0 - dz / norm * BASE_PUSH,
    )
}

/// 1.8 knockback resistance: all or nothing. True when the base knockback is cancelled.
#[must_use]
pub fn resists_knockback(resistance: f64, random: f64) -> bool {
    random < resistance
}

/// Extra knockback of an accepted hit with `level` (Knockback enchantment + 1 when the attacker
/// sprints): pushed along the attacker's yaw (degrees), `+0.1` vertical once.
#[must_use]
pub fn extra_knockback(motion: Vector3<f64>, attacker_yaw: f32, level: i32) -> Vector3<f64> {
    let (sin, cos) = yaw_sin_cos(attacker_yaw);
    let push = level as f32 * EXTRA_HORIZONTAL;
    Vector3::new(
        motion.x + f64::from(-sin * push),
        motion.y + EXTRA_VERTICAL,
        motion.z + f64::from(cos * push),
    )
}

fn yaw_sin_cos(yaw: f32) -> (f32, f32) {
    let angle = yaw * std::f32::consts::PI / 180.0;
    (f64::from(angle).sin() as f32, f64::from(angle).cos() as f32)
}

/// Vertical speed and horizontal boost of a jump in the server-side motion (1.8 `jump`):
/// `0.42` (+0.1 per Jump Boost level), plus 0.2 along the yaw when sprinting.
#[must_use]
pub fn jump_motion(
    motion: Vector3<f64>,
    jump_boost_amp: Option<i32>,
    sprinting: bool,
    yaw: f32,
) -> Vector3<f64> {
    let mut y = f64::from(0.42f32);
    if let Some(amp) = jump_boost_amp {
        y += f64::from((amp + 1) as f32 * 0.1);
    }
    let mut result = Vector3::new(motion.x, y, motion.z);
    if sprinting {
        let angle = yaw * 0.017_453_292;
        let (sin, cos) = (f64::from(angle).sin() as f32, f64::from(angle).cos() as f32);
        result.x -= f64::from(sin * 0.2);
        result.z += f64::from(cos * 0.2);
    }
    result
}

/// Velocity as a 1.8 client reads it: each component clamped to +-3.9, then `(int)(v * 8000)`
/// divided by 8000.
#[must_use]
pub fn packet_velocity(velocity: Vector3<f64>) -> Vector3<f64> {
    let quantize = |v: f64| {
        f64::from((v.clamp(-PACKET_VELOCITY_LIMIT, PACKET_VELOCITY_LIMIT) * 8000.0) as i16) / 8000.0
    };
    Vector3::new(
        quantize(velocity.x),
        quantize(velocity.y),
        quantize(velocity.z),
    )
}

/// Clamps each component to the 1.8 packet limit, keeping the full precision.
#[must_use]
pub fn clamp_packet_velocity(velocity: Vector3<f64>) -> Vector3<f64> {
    Vector3::new(
        velocity
            .x
            .clamp(-PACKET_VELOCITY_LIMIT, PACKET_VELOCITY_LIMIT),
        velocity
            .y
            .clamp(-PACKET_VELOCITY_LIMIT, PACKET_VELOCITY_LIMIT),
        velocity
            .z
            .clamp(-PACKET_VELOCITY_LIMIT, PACKET_VELOCITY_LIMIT),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Server-side motion of a player standing still: `(0 - 0.08) * 0.98F`.
    const GHOST_ON_GROUND: f64 = -0.078_400_001_5;

    fn close(a: Vector3<f64>, x: f64, y: f64, z: f64, tolerance: f64) {
        assert!(
            (a.x - x).abs() < tolerance
                && (a.y - y).abs() < tolerance
                && (a.z - z).abs() < tolerance,
            "{a:?} != ({x}, {y}, {z})"
        );
    }

    /// Attacker 2 blocks behind the victim along +Z, yaw 0: `(dx, dz) = (0, -2)`.
    fn hit(ghost: Vector3<f64>) -> Vector3<f64> {
        base_knockback(ghost, 0.0, -2.0, || 0.5)
    }

    /// E1: the ghost falls, so a victim in the air is lifted from its real vertical motion
    /// (Paper 1.8.8: `-0.2308` gives `0.28463`), and a rising one is capped at `0.4`.
    #[test]
    fn e1_airborne_victim_uses_real_vertical_motion() {
        assert!((hit(Vector3::new(0.0, -0.2308, 0.0)).y - 0.2846).abs() < 1.0e-4);
        assert!((hit(Vector3::new(0.0, 0.42, 0.0)).y - 0.4).abs() < 1.0e-6);
        // Gravity of the ghost at rest: `(0 - 0.08) * 0.98`, the value of `GHOST_ON_GROUND`.
        assert!(((0.0 - 0.08f64) * 0.98 - GHOST_ON_GROUND).abs() < 1.0e-6);
    }

    #[test]
    fn v1_plain_hit_on_still_victim() {
        let v = hit(Vector3::new(0.0, GHOST_ON_GROUND, 0.0));
        close(
            v,
            0.0,
            0.360_800_005_197_525_03,
            0.400_000_005_960_464_5,
            1e-9,
        );
        close(packet_velocity(v), 0.0, 0.36075, 0.4, 1e-12);
    }

    #[test]
    fn v2_sprint_hit() {
        let v = extra_knockback(hit(Vector3::new(0.0, GHOST_ON_GROUND, 0.0)), 0.0, 1);
        close(
            v,
            0.0,
            0.460_800_005_197_525_06,
            0.900_000_005_960_464_5,
            1e-9,
        );
        close(packet_velocity(v), 0.0, 0.46075, 0.9, 1e-12);
    }

    #[test]
    fn v4_knockback_two_and_sprint() {
        let v = extra_knockback(hit(Vector3::new(0.0, GHOST_ON_GROUND, 0.0)), 0.0, 3);
        close(
            v,
            0.0,
            0.460_800_005_197_525_06,
            1.900_000_005_960_464_5,
            1e-9,
        );
        close(packet_velocity(v), 0.0, 0.46075, 1.9, 1e-12);
    }

    #[test]
    fn v5_victim_one_tick_after_a_jump_is_launched_again() {
        let ghost_y = (0.42f64 - 0.08) * f64::from(0.98f32);
        let v = hit(Vector3::new(0.0, ghost_y, 0.0));
        close(
            v,
            0.0,
            0.400_000_005_960_464_5,
            0.400_000_005_960_464_5,
            1e-9,
        );
        close(
            extra_knockback(v, 0.0, 1),
            0.0,
            0.500_000_005_960_464_5,
            0.900_000_005_960_464_5,
            1e-9,
        );
    }

    #[test]
    fn v6_falling_victim_is_pushed_up_in_the_air() {
        let v = hit(Vector3::new(0.0, -0.3, 0.0));
        close(v, 0.0, 0.250_000_005_96, 0.400_000_006_0, 1e-9);
        close(
            extra_knockback(v, 0.0, 1),
            0.0,
            0.350_000_005_96,
            0.900_000_006_0,
            1e-9,
        );
    }

    #[test]
    fn v7_yaw_45() {
        let yaw = 45.0f32;
        let look = (
            -f64::from(yaw).to_radians().sin(),
            f64::from(yaw).to_radians().cos(),
        );
        let v = base_knockback(
            Vector3::new(0.0, GHOST_ON_GROUND, 0.0),
            -look.0 * 2.0,
            -look.1 * 2.0,
            || 0.5,
        );
        close(v, -0.282_842_72, 0.360_800_01, 0.282_842_72, 1e-6);
        close(
            extra_knockback(v, yaw, 1),
            -0.636_396_10,
            0.460_800_01,
            0.636_396_10,
            1e-6,
        );
    }

    #[test]
    fn v8_ghost_already_moving_away() {
        let ghost_y = (0.42f64 - 0.08) * f64::from(0.98f32);
        let v = extra_knockback(hit(Vector3::new(0.0, ghost_y, 0.2)), 0.0, 1);
        close(v, 0.0, 0.500_000_01, 1.000_000_01, 1e-8);
    }

    #[test]
    fn v9_partial_hit_leaves_the_extra_push_in_the_ghost() {
        // The caller adds the extra push to the server-side motion and sends nothing.
        let v = extra_knockback(Vector3::new(0.0, 0.0, 0.0), 0.0, 1);
        close(v, 0.0, 0.1, 0.5, 1e-9);
    }

    #[test]
    fn near_zero_distance_draws_a_direction() {
        // The first draw (0.1, 0.1, 0.5, 0.4) stays under the 1e-4 threshold and is redone.
        let mut values = [0.5, 0.4, 0.5, 0.4, 0.9, 0.1, 0.1, 0.9].into_iter();
        let v = base_knockback(Vector3::new(0.0, 0.0, 0.0), 0.0, 0.0, || {
            values.next().unwrap()
        });
        // (0.9 - 0.1, 0.1 - 0.9) * 0.01 = (0.008, -0.008): pushed away from the attacker.
        assert!(v.x < 0.0 && v.z > 0.0);
        assert!((v.x.hypot(v.z) - BASE_PUSH).abs() < 1e-6);
    }

    #[test]
    fn knockback_resistance_is_all_or_nothing() {
        assert!(resists_knockback(1.0, 0.999));
        assert!(!resists_knockback(0.0, 0.0));
        assert!(resists_knockback(0.6, 0.59));
        assert!(!resists_knockback(0.6, 0.6));
    }

    #[test]
    fn jump_impulse() {
        let still = jump_motion(Vector3::new(0.0, 0.0, 0.0), None, false, 0.0);
        close(still, 0.0, 0.419_999_986_886_978_15, 0.0, 1e-9);
        let boosted = jump_motion(Vector3::new(0.0, 0.0, 0.0), Some(1), true, 0.0);
        close(boosted, 0.0, 0.62, 0.2, 1e-6);
    }

    #[test]
    fn packet_is_clamped() {
        let v = packet_velocity(Vector3::new(10.0, -10.0, 0.0));
        close(v, 3.9, -3.9, 0.0, 1e-3);
    }

    const DIAMOND_SWORD_ATTRIBUTE: f64 = 8.0;

    #[test]
    fn weapon_table() {
        assert_eq!(weapon_bonus(&Item::DIAMOND_SWORD), Some(7.0));
        assert_eq!(weapon_bonus(&Item::WOODEN_SWORD), Some(4.0));
        assert_eq!(weapon_bonus(&Item::GOLDEN_SWORD), Some(4.0));
        assert_eq!(weapon_bonus(&Item::IRON_AXE), Some(5.0));
        assert_eq!(weapon_bonus(&Item::STONE_PICKAXE), Some(3.0));
        assert_eq!(weapon_bonus(&Item::DIAMOND_SHOVEL), Some(4.0));
        assert_eq!(weapon_bonus(&Item::STICK), None);
        assert_eq!(weapon_bonus(&Item::NETHERITE_SWORD), None);
    }

    #[test]
    fn d1_to_d4_melee_damage() {
        let sword = DIAMOND_SWORD_ATTRIBUTE;
        assert!((melee_damage(sword, false, 0.0) - 8.0).abs() < 1e-6);
        assert!((melee_damage(sword, true, 0.0) - 12.0).abs() < 1e-6);
        let sharp5 = enchantment_damage(5, 0, 0);
        assert!((melee_damage(sword, false, sharp5) - 14.25).abs() < 1e-6);
        assert!((melee_damage(sword, true, sharp5) - 18.25).abs() < 1e-6);
        let strength1 = damage_with_effects(sword, Some(0), None);
        assert!((melee_damage(strength1, false, 0.0) - 18.4).abs() < 1e-5);
        let strength2 = damage_with_effects(sword, Some(1), None);
        assert!((melee_damage(strength2, false, 0.0) - 28.8).abs() < 1e-5);
    }

    #[test]
    fn weakness_is_added_before_strength_multiplies() {
        assert!((damage_with_effects(8.0, None, Some(0)) - 7.5).abs() < 1e-9);
        assert!((damage_with_effects(8.0, Some(0), Some(0)) - 7.5 * 2.3).abs() < 1e-9);
    }

    #[test]
    fn smite_and_bane_count_only_against_their_targets() {
        assert!((enchantment_damage(0, 5, 0) - 12.5).abs() < 1e-6);
        assert!((enchantment_damage(2, 0, 3) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn critical_needs_a_fall_but_not_a_stop_of_the_sprint() {
        assert!(is_critical(0.1, false, false, false, false, false, true));
        assert!(!is_critical(0.0, false, false, false, false, false, true));
        assert!(!is_critical(0.1, true, false, false, false, false, true));
        assert!(!is_critical(0.1, false, true, false, false, false, true));
        assert!(!is_critical(0.1, false, false, true, false, false, true));
        assert!(!is_critical(0.1, false, false, false, true, false, true));
        assert!(!is_critical(0.1, false, false, false, false, true, true));
        assert!(!is_critical(0.1, false, false, false, false, false, false));
    }
}
