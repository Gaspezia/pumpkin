use std::{sync::Arc, time::Duration};

use pumpkin_core::{entity::EntityBase, server::Server};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::vector3::Vector3;

use crate::bot::{HeadlessBot, MAX_POLL_TICKS};

const BOT_NAME: &str = "TeleportBot";

/// A far teleport moves the watched section and the chunk tickets to the destination at once,
/// without waiting for a move packet (vanilla `ChunkMap.move`). The bot stays still afterwards.
pub async fn run_teleport_recenter_test(
    server: &Arc<Server>,
    addr: std::net::SocketAddr,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let _bot = HeadlessBot::connect_and_join(addr, BOT_NAME).await?;

    let mut player = None;
    for _ in 0..MAX_POLL_TICKS {
        server.tick();
        tokio::time::sleep(Duration::from_millis(5)).await;
        player = world.get_player_by_name(BOT_NAME);
        if player.is_some() {
            break;
        }
    }
    let Some(player) = player else {
        return Err(format!("Could not find '{BOT_NAME}' in Overworld"));
    };

    let from = player.get_entity().pos.load();
    let to = Vector3::new(from.x + 320.0, from.y, from.z + 320.0);
    player.request_teleport(to, 0.0, 0.0);

    for _ in 0..20 {
        server.tick();
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    let expected = player.get_entity().chunk_pos.load();
    let watched = player.watched_section.load().center;
    if watched != expected {
        return Err(format!(
            "teleport_recenter: watched section stayed at {watched:?}, expected {expected:?}"
        ));
    }
    let held = player
        .held_chunk_tickets
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .map(|(center, _, _)| center);
    if held != Some(expected) {
        return Err(format!(
            "teleport_recenter: tickets held at {held:?}, expected {expected:?}"
        ));
    }

    summary_lines.push("[PASS] bot:teleport_recenters_chunks_while_still".to_string());
    Ok(())
}
