//! Minecraft 1.8 projectile rules (`[pvp] combat = "legacy_1_8"`).
//!
//! Pure functions for the launch, the flight step, the arrow damage and Punch, and the fishing
//! hook, so they can be tested against the reference vectors of the spec (P1 to P14). The
//! projectile entities only branch to this module.
//!
//! In 1.8 a projectile moves first, then loses speed to the drag, then to gravity; nothing is
//! inherited from the shooter's motion. `sin` and `cos` use the exact functions rounded to `f32`
//! (1.8 uses a lookup table, within 1e-4 of these).

use pumpkin_util::math::vector3::Vector3;

/// Gravity of an arrow, a potion and a fiole.
pub const ARROW_GRAVITY: f64 = 0.05;
/// Gravity of a snowball, an egg and a pearl.
pub const THROWN_GRAVITY: f64 = 0.03;
/// Drag of an arrow in the air.
pub const AIR_DRAG: f64 = 0.990_000_009_536_743_2;
/// Drag of an arrow in water.
pub const ARROW_WATER_DRAG: f64 = 0.600_000_023_841_857_9;
/// Drag of a thrown item in water.
pub const THROWN_WATER_DRAG: f64 = 0.8;
/// Speed of a thrown item and of the hook.
pub const THROWN_SPEED: f32 = 1.5;
/// Gravity of the hook, applied before its drag (`0.04F` widened).
pub const ROD_GRAVITY: f64 = 0.039_999_999_105_930_33;
/// Drag of the hook in the air.
pub const ROD_DRAG: f64 = 0.920_000_016_689_300_5;
/// Drag of the hook on the ground or against a wall.
pub const ROD_GROUND_DRAG: f64 = 0.5;
/// Standard deviation of the launch imprecision, per unit of inaccuracy.
pub const GAUSS_SCALE: f64 = 0.007_5;
/// Margin added to every entity box a projectile can hit.
pub const HIT_MARGIN: f64 = 0.3;
/// Ticks in flight before the shooter can be hit.
pub const OWNER_GRACE_TICKS: i32 = 5;

/// Power of a bow drawn for `ticks`; nothing is shot below `0.1`. Full draw (`1.0`) is critical.
#[must_use]
pub fn bow_power(ticks: i32) -> Option<f32> {
    let f = ticks as f32 / 20.0;
    let f = ((f * f + f * 2.0) / 3.0).min(1.0);
    (f >= 0.1).then_some(f)
}

fn trig(degrees: f32) -> (f32, f32) {
    let angle = degrees / 180.0 * std::f32::consts::PI;
    (f64::from(angle).sin() as f32, f64::from(angle).cos() as f32)
}

/// Where the projectile starts, relative to the shooter's eye: 0.16 to the side, 0.1 down.
#[must_use]
pub fn launch_offset(yaw: f32) -> Vector3<f64> {
    let (sin, cos) = trig(yaw);
    Vector3::new(f64::from(-cos * 0.16), -0.1, f64::from(-sin * 0.16))
}

/// Unnormalised direction of a shot. `pitch_offset` (-20 for a potion) only tilts Y.
#[must_use]
pub fn launch_direction(yaw: f32, pitch: f32, pitch_offset: f32) -> Vector3<f64> {
    let (yaw_sin, yaw_cos) = trig(yaw);
    let (_, pitch_cos) = trig(pitch);
    let (tilt_sin, _) = trig(pitch + pitch_offset);
    Vector3::new(
        f64::from(-yaw_sin * pitch_cos),
        f64::from(-tilt_sin),
        f64::from(yaw_cos * pitch_cos),
    )
}

/// Velocity of a shot: the direction normalised, `gauss * 0.0075 * inaccuracy` added to each
/// axis, then scaled to `speed`.
#[must_use]
pub fn launch_velocity(
    direction: Vector3<f64>,
    speed: f64,
    inaccuracy: f64,
    mut gauss: impl FnMut() -> f64,
) -> Vector3<f64> {
    let n = direction.normalize();
    let spread = GAUSS_SCALE * inaccuracy;
    Vector3::new(
        (n.x + gauss() * spread) * speed,
        (n.y + gauss() * spread) * speed,
        (n.z + gauss() * spread) * speed,
    )
}

/// Standard normal sample (Box-Muller) from two uniform samples in `[0, 1)`.
#[must_use]
pub fn gauss(u1: f64, u2: f64) -> f64 {
    (-2.0 * (1.0 - u1).ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// One tick of an arrow or a thrown item: move, then drag, then gravity.
#[must_use]
pub fn flight_step(
    position: Vector3<f64>,
    velocity: Vector3<f64>,
    drag: f64,
    gravity: f64,
) -> (Vector3<f64>, Vector3<f64>) {
    (
        position + velocity,
        Vector3::new(
            velocity.x * drag,
            velocity.y * drag - gravity,
            velocity.z * drag,
        ),
    )
}

/// Damage of an arrow: `ceil(speed * base)`, plus up to `l / 2 + 1` when critical.
#[must_use]
pub fn arrow_damage(
    speed: f64,
    base: f64,
    critical: bool,
    random_below: impl FnOnce(i32) -> i32,
) -> i32 {
    let damage = (speed * base).ceil().clamp(0.0, f64::from(i32::MAX)) as i32;
    if critical {
        damage.saturating_add(random_below(damage / 2 + 2))
    } else {
        damage
    }
}

/// Punch: pushes along the flight, `level * 0.6` horizontally and `0.1` up, ignoring knockback
/// resistance. `None` when the arrow has no horizontal speed.
#[must_use]
pub fn punch_push(velocity: Vector3<f64>, level: i32) -> Option<Vector3<f64>> {
    let horizontal = velocity.x.hypot(velocity.z);
    (level > 0 && horizontal > 0.0).then(|| {
        let k = f64::from(level) * 0.6 / horizontal;
        Vector3::new(velocity.x * k, 0.1, velocity.z * k)
    })
}

/// One tick of the fishing hook: move, then gravity, then drag.
///
/// The drag is halved on the ground or against a wall. `immersion` is the share of the hook under
/// water (0 to 1); the buoyancy `0.04 * (2 * immersion - 1)` replaces the gravity.
#[must_use]
pub fn rod_step(
    position: Vector3<f64>,
    velocity: Vector3<f64>,
    immersion: f64,
    grounded: bool,
) -> (Vector3<f64>, Vector3<f64>) {
    let drag = if grounded { ROD_GROUND_DRAG } else { ROD_DRAG };
    let vy = velocity.y + ROD_GRAVITY * (immersion * 2.0 - 1.0);
    (
        position + velocity,
        Vector3::new(velocity.x * drag, vy * drag, velocity.z * drag),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: f64, b: f64, tolerance: f64) {
        assert!((a - b).abs() < tolerance, "{a} != {b}");
    }

    /// Positions (x, y) of `ticks` steps.
    fn track(v0: Vector3<f64>, drag: f64, gravity: f64, ticks: usize) -> Vec<(f64, f64)> {
        let (mut p, mut v) = (Vector3::new(0.0, 0.0, 0.0), v0);
        (0..ticks)
            .map(|_| {
                (p, v) = flight_step(p, v, drag, gravity);
                (p.x, p.y)
            })
            .collect()
    }

    #[test]
    fn p1_bow_power() {
        for (ticks, power, speed, damage) in [
            (3, 0.1075, 0.3225, 1),
            (5, 0.1875, 0.5625, 2),
            (10, 0.4167, 1.25, 3),
            (15, 0.6875, 2.0625, 5),
            (20, 1.0, 3.0, 6),
        ] {
            let f = f64::from(bow_power(ticks).unwrap());
            near(f, power, 1.0e-4);
            near(3.0 * f, speed, 1.0e-4);
            assert_eq!(arrow_damage(3.0 * f, 2.0, false, |_| 0), damage);
        }
        assert!(bow_power(2).is_none());
        assert_eq!(bow_power(40), Some(1.0));
    }

    #[test]
    fn p2_arrow_trajectory() {
        let rows = track(Vector3::new(3.0, 0.0, 0.0), AIR_DRAG, ARROW_GRAVITY, 6);
        for (row, (x, y)) in rows.iter().zip([
            (3.0, 0.0),
            (5.97, -0.05),
            (8.9103, -0.1495),
            (11.8212, -0.298),
            (14.703, -0.495),
            (17.556, -0.7401),
        ]) {
            near(row.0, x, 1.0e-3);
            near(row.1, y, 1.0e-3);
        }
    }

    #[test]
    fn p6_thrown_trajectory() {
        let rows = track(Vector3::new(1.5, 0.0, 0.0), AIR_DRAG, THROWN_GRAVITY, 6);
        for (row, (x, y)) in rows.iter().zip([
            (1.5, 0.0),
            (2.985, -0.03),
            (4.4552, -0.0897),
            (5.9106, -0.1788),
            (7.3515, -0.297),
            (8.778, -0.444),
        ]) {
            near(row.0, x, 1.0e-3);
            near(row.1, y, 1.0e-3);
        }
    }

    #[test]
    fn p7_arrow_in_water_drags_after_the_move() {
        let rows = track(
            Vector3::new(3.0, 0.0, 0.0),
            ARROW_WATER_DRAG,
            ARROW_GRAVITY,
            4,
        );
        for (row, (x, y)) in
            rows.iter()
                .zip([(3.0, 0.0), (4.8, -0.05), (5.88, -0.13), (6.528, -0.228)])
        {
            near(row.0, x, 1.0e-3);
            near(row.1, y, 1.0e-3);
        }
    }

    #[test]
    fn p8_launch_offset() {
        let o = launch_offset(0.0);
        near(o.x, -0.16, 1.0e-6);
        near(o.y, -0.1, 1.0e-12);
        near(o.z, 0.0, 1.0e-6);
        near(launch_offset(90.0).z, -0.16, 1.0e-6);
        near(launch_offset(45.0).x, -0.113_137, 1.0e-5);
        near(launch_offset(45.0).z, -0.113_137, 1.0e-5);
    }

    #[test]
    fn p9_launch_direction() {
        let d = launch_direction(45.0, -30.0, 0.0);
        near(d.x, -0.612_372, 1.0e-5);
        near(d.y, 0.5, 1.0e-5);
        near(d.z, 0.612_372, 1.0e-5);
        // the potion tilt only moves Y: cos(pitch) is not corrected
        let d = launch_direction(0.0, 10.0, -20.0);
        near(d.y, 0.173_648, 1.0e-5);
        near(d.z, 0.984_808, 1.0e-5);
    }

    #[test]
    fn launch_velocity_is_the_direction_at_speed_without_spread() {
        let v = launch_velocity(Vector3::new(0.0, 0.0, 2.0), 3.0, 1.0, || 0.0);
        near(v.z, 3.0, 1.0e-12);
        // one standard deviation on every axis
        let v = launch_velocity(Vector3::new(0.0, 0.0, 2.0), 1.0, 1.0, || 1.0);
        near(v.x, GAUSS_SCALE, 1.0e-12);
        near(v.z, 1.0 + GAUSS_SCALE, 1.0e-12);
    }

    #[test]
    fn gauss_is_centred_and_unit() {
        let n = 20_000;
        let samples: Vec<f64> = (0..n)
            .map(|i| {
                gauss(
                    f64::from(i) / f64::from(n),
                    f64::from((i * 7919) % n) / f64::from(n),
                )
            })
            .collect();
        let mean = samples.iter().sum::<f64>() / f64::from(n);
        let var = samples.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / f64::from(n);
        near(mean, 0.0, 0.05);
        near(var, 1.0, 0.1);
    }

    #[test]
    fn p10_arrow_damage_power_and_critical() {
        // Power V: base 2 + 0.5 * 5 + 0.5, full charge
        assert_eq!(arrow_damage(3.0, 5.0, false, |_| 0), 15);
        assert_eq!(arrow_damage(1.25, 2.0, false, |_| 0), 3);
        // critical: the extra is drawn below l / 2 + 2
        assert_eq!(
            arrow_damage(3.0, 2.0, true, |bound| {
                assert_eq!(bound, 5);
                4
            }),
            10
        );
    }

    #[test]
    fn p4_p11_punch() {
        let p = punch_push(Vector3::new(2.97, 0.0, 0.0), 2).unwrap();
        near(p.x, 1.2, 1.0e-12);
        near(p.y, 0.1, 1.0e-12);
        near(
            punch_push(Vector3::new(2.97, 0.0, 0.0), 1).unwrap().x,
            0.6,
            1.0e-12,
        );
        assert!(punch_push(Vector3::new(0.0, -1.0, 0.0), 2).is_none());
        assert!(punch_push(Vector3::new(1.0, 0.0, 0.0), 0).is_none());
    }

    #[test]
    fn p3_rod_trajectory_and_range() {
        let (mut p, mut v) = (Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.5, 0.0, 0.0));
        let mut rows = Vec::new();
        let mut landed = None;
        for t in 1..=40 {
            (p, v) = rod_step(p, v, 0.0, false);
            rows.push((p.x, p.y));
            if landed.is_none() && p.y <= -1.52 {
                landed = Some((t, p.x));
            }
        }
        for (row, (x, y)) in rows.iter().zip([
            (1.5, 0.0),
            (2.88, -0.037),
            (4.15, -0.107),
            (5.318, -0.209),
            (6.392, -0.34),
        ]) {
            near(row.0, x, 1.0e-3);
            near(row.1, y, 1.0e-3);
        }
        let (tick, x) = landed.unwrap();
        assert_eq!(tick, 11);
        near(x, 11.26, 0.01);
    }

    #[test]
    fn p13_p14_rod_water_and_ground() {
        let (_, v) = rod_step(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            1.0,
            false,
        );
        near(v.y, 0.0368, 1.0e-4);
        let (_, v) = rod_step(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.5, 0.0, 0.0),
            0.0,
            true,
        );
        near(v.x, 0.75, 1.0e-12);
    }
}
