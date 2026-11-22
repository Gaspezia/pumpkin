use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use pumpkin_core::{
    entity::{Entity, EntityBase, passive::sheep::SheepEntity},
    server::Server,
};
use pumpkin_data::{entity::EntityType, packet::CURRENT_MC_VERSION};
use pumpkin_protocol::{
    MultiVersionJavaPacket,
    codec::{lp_vector_3d::LpVector3d, var_int::VarInt},
    java::{client::play::CEntityVelocity, server::play::SAttack},
};
use pumpkin_util::{GameMode, math::vector3::Vector3};
use tokio::time::timeout;

use crate::bot::HeadlessBot;

const TICKS: usize = 12;
const SET_X: f64 = 0.5;
// Gap that ends a drain; generous, the VM is often loaded.
const QUIET: Duration = Duration::from_secs(1);
// Longest wait for the knockback packet already queued by the tick that handled the hit.
const HIT_WAIT: Duration = Duration::from_secs(10);

/// Reads every pending packet and returns the velocities sent for `id`.
async fn drain_velocities(bot: &mut HeadlessBot, id: i32) -> Result<Vec<Vector3<f64>>, String> {
    let velocity_id = CEntityVelocity::to_id(CURRENT_MC_VERSION);
    let mut out = Vec::new();
    loop {
        let pkt = match timeout(QUIET, bot.reader.get_raw_packet()).await {
            Ok(Ok(pkt)) => pkt,
            Ok(Err(err)) => return Err(format!("Bot read error: {err}")),
            Err(_) => return Ok(out),
        };
        if pkt.id != velocity_id {
            continue;
        }
        let mut slice = &pkt.payload[..];
        let entity_id = VarInt::decode(&mut slice).map_err(|err| err.to_string())?;
        if entity_id.0 == id {
            let LpVector3d(v) = LpVector3d::read(&mut slice).map_err(|err| err.to_string())?;
            out.push(v);
        }
    }
}

/// Reads packets until a horizontal velocity for `id` arrives; `None` after `HIT_WAIT`.
async fn wait_horizontal_velocity(
    bot: &mut HeadlessBot,
    id: i32,
) -> Result<Option<Vector3<f64>>, String> {
    let velocity_id = CEntityVelocity::to_id(CURRENT_MC_VERSION);
    let deadline = tokio::time::Instant::now() + HIT_WAIT;
    while let Ok(read) = tokio::time::timeout_at(deadline, bot.reader.get_raw_packet()).await {
        let pkt = read.map_err(|err| format!("Bot read error: {err}"))?;
        if pkt.id != velocity_id {
            continue;
        }
        let mut slice = &pkt.payload[..];
        let entity_id = VarInt::decode(&mut slice).map_err(|err| err.to_string())?;
        if entity_id.0 == id {
            let LpVector3d(v) = LpVector3d::read(&mut slice).map_err(|err| err.to_string())?;
            if v.x.hypot(v.z) > 0.01 {
                return Ok(Some(v));
            }
        }
    }
    Ok(None)
}

async fn tick(server: &Arc<Server>) {
    tokio::time::sleep(Duration::from_millis(5)).await;
    server.tick();
}

/// Vanilla/Paper order: velocity cause -> `ChunkMap.tick` (`sendChanges`) -> entity physics.
/// 1. A velocity set at the head of the tick (plugin task) reaches the client as set, never as
///    the friction-decayed value of the same tick.
/// 2. A knockback from an incoming packet is sent in the tick that handled the packet, not in
///    the next one (no extra tick of knockback latency).
pub async fn run_tracker_before_entity_tick_test(
    server: &Arc<Server>,
    bot: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let player = server.get_player_by_name(&bot.name).ok_or("missing bot")?;
    let world = player.world();
    let spawn = || {
        let sheep = SheepEntity::new(Entity::new(
            world.clone(),
            player.position() + Vector3::new(1.5, 0.0, 0.0),
            &EntityType::SHEEP,
        ));
        world.spawn_entity(sheep.clone());
        sheep
    };
    let sheep = spawn();
    let id = sheep.get_entity().entity_id;

    // Let the spawn packets through, then forget everything sent so far.
    for _ in 0..4 {
        tick(server).await;
    }
    drain_velocities(bot, id).await?;

    // Like a plugin task: velocity set at the head of every tick, then the server ticks.
    for _ in 0..TICKS {
        sheep
            .get_entity()
            .set_velocity(Vector3::new(SET_X, 0.0, 0.0));
        tick(server).await;
    }
    let sent = drain_velocities(bot, id).await?;
    if let Some(v) = sent.iter().find(|v| (v.x - SET_X).abs() > 0.01) {
        return Err(format!(
            "Tracker sent a velocity of x={} after the physics of the same tick (expected the set x={SET_X})",
            v.x
        ));
    }
    if sent.len() < TICKS {
        return Err(format!(
            "Only {} velocity packets seen for {TICKS} ticks",
            sent.len()
        ));
    }

    // A hit on a fresh sheep (the first one was pushed out of reach): the packet is handled in
    // some tick, the knockback must leave in that same tick.
    sheep.get_entity().remove();
    let sheep = spawn();
    let id = sheep.get_entity().entity_id;
    for _ in 0..8 {
        tick(server).await;
    }
    drain_velocities(bot, id).await?;
    let original_mode = player.gamemode.load();
    player.gamemode.store(GameMode::Survival);
    // `Player::attack` sets this only once the hit has landed (damage accepted), just before the
    // knockback. Health is not a marker: the sheep can lose it to the terrain (fall, suffocation)
    // in a tick that has not handled the packet yet.
    let attacker = &player.living_entity;
    attacker.last_attacking_id.store(0, Ordering::Relaxed);
    bot.send_packet(&SAttack {
        entity_id: VarInt(id),
    })
    .await?;
    let mut handled = false;
    for _ in 0..50 {
        tick(server).await;
        if attacker.last_attacking_id.load(Ordering::Relaxed) == id {
            handled = true;
            break;
        }
    }
    player.gamemode.store(original_mode);
    if !handled {
        return Err("The bot's attack never landed on the sheep".into());
    }
    // No further tick: a knockback sent late (next tick) must stay missing. The packet is already
    // queued, so wait for it as long as a loaded VM needs.
    let sent = wait_horizontal_velocity(bot, id).await?;
    if sent.is_none() {
        return Err(format!(
            "Knockback was not sent in the tick that handled the hit (vel now {:?}, hurt {})",
            sheep.get_entity().velocity.load(),
            sheep
                .mob_entity
                .living_entity
                .hurt_cooldown
                .load(Ordering::Relaxed)
        ));
    }
    sheep.get_entity().remove();
    summary_lines.push("[PASS] bot:tracker_before_entity_tick".into());
    Ok(())
}
