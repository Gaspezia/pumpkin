use std::{sync::Arc, time::Duration};

use pumpkin_core::{
    entity::{Entity, EntityBase, passive::sheep::SheepEntity},
    server::Server,
};
use pumpkin_data::{entity::EntityType, packet::CURRENT_MC_VERSION};
use pumpkin_protocol::{
    MultiVersionJavaPacket,
    codec::{lp_vector_3d::LpVector3d, var_int::VarInt},
    java::client::play::CEntityVelocity,
};
use pumpkin_util::math::vector3::Vector3;
use tokio::time::timeout;

use crate::bot::HeadlessBot;

const TICKS: usize = 12;
const SET_X: f64 = 0.5;
// Gap that ends a drain; generous, the VM is often loaded.
const QUIET: Duration = Duration::from_secs(1);

/// Vanilla/Paper send entity changes (`ChunkMap.tick` -> `sendChanges`) before the entity pass.
/// A velocity set at the head of the tick (plugin task) must reach the client as set, never as
/// the friction-decayed value of the same tick.
pub async fn run_tracker_before_entity_tick_test(
    server: &Arc<Server>,
    bot: &mut HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let player = server.get_player_by_name(&bot.name).ok_or("missing bot")?;
    let world = player.world();
    let sheep = SheepEntity::new(Entity::new(
        world.clone(),
        player.position() + Vector3::new(2.0, 0.0, 0.0),
        &EntityType::SHEEP,
    ));
    world.spawn_entity(sheep.clone());
    let id = sheep.get_entity().entity_id;
    let velocity_id = CEntityVelocity::to_id(CURRENT_MC_VERSION);

    // Let the spawn packets through, then forget everything sent so far.
    for _ in 0..4 {
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
    }
    while timeout(QUIET, bot.reader.get_raw_packet()).await.is_ok() {}

    // Like a plugin task: velocity set at the head of every tick, then the server ticks.
    for _ in 0..TICKS {
        sheep
            .get_entity()
            .set_velocity(Vector3::new(SET_X, 0.0, 0.0));
        tokio::time::sleep(Duration::from_millis(5)).await;
        server.tick();
    }
    sheep.get_entity().remove();

    let mut seen = 0;
    let mut total = 0;
    loop {
        let pkt = match timeout(QUIET, bot.reader.get_raw_packet()).await {
            Ok(Ok(pkt)) => pkt,
            Ok(Err(err)) => return Err(format!("Bot read error after {total} packets: {err}")),
            Err(_) => break,
        };
        total += 1;
        if pkt.id != velocity_id {
            continue;
        }
        let mut slice = &pkt.payload[..];
        let entity_id = VarInt::decode(&mut slice).map_err(|err| err.to_string())?;
        if entity_id.0 != id {
            continue;
        }
        let LpVector3d(v) = LpVector3d::read(&mut slice).map_err(|err| err.to_string())?;
        seen += 1;
        if (v.x - SET_X).abs() > 0.01 {
            return Err(format!(
                "Tracker sent a velocity of x={} after the physics of the same tick (expected the set x={SET_X})",
                v.x
            ));
        }
    }
    if seen < TICKS {
        return Err(format!(
            "Only {seen} velocity packets seen for {TICKS} ticks ({total} packets)"
        ));
    }
    summary_lines.push("[PASS] bot:tracker_before_entity_tick".into());
    Ok(())
}
