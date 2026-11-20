#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_swing_arm(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        swing_arm: &SSwingArm,
    ) {
        player.update_last_action_time();
        let Ok(hand) = Hand::from_packet_id(swing_arm.hand.0) else {
            self.try_kick(&TextComponent::text("Invalid hand"));
            return;
        };

        let mut anim_event = crate::plugin::api::events::player::player_animation::PlayerAnimationEvent::new(
            player.clone(),
            match hand {
                Hand::Left => crate::plugin::api::events::player::player_animation::PlayerAnimationType::ArmSwingOff,
                Hand::Right => crate::plugin::api::events::player::player_animation::PlayerAnimationType::ArmSwingMain,
            },
        );
        server.plugin_manager.fire_blocking(server, &mut anim_event);
        if anim_event.cancelled {
            return;
        }

        let (yaw, pitch) = player.rotation();
        let hit = player.world().clip(&pumpkin_world::clip::ClipContext::new(
            player.eye_position(),
            player
                .eye_position()
                .add(&(Vector3::rotation_vector(f64::from(pitch), f64::from(yaw)) * 4.5)),
            pumpkin_world::clip::ClipBlock::Outline,
            pumpkin_world::clip::ClipFluid::None,
        ));

        if hit.is_miss {
            let event =
                PlayerInteractEvent::new(player, InteractAction::LeftClickAir, &Block::AIR, None);
            send_cancellable_blocking! {{
                &server;
                event;
                'after: {
                    player.swing_hand(hand, false);
                }
            }}
        } else {
            // Paper `handleAnimate`: an ADVENTURE client that cannot break the block never sends a
            // dig packet, only the arm swing, so the left click on a block is read from here.
            let gamemode = player.gamemode.load();
            if gamemode == GameMode::Adventure {
                let world = player.world();
                let block = world.get_block(&hit.block_pos);
                if left_click_block_from_swing(
                    gamemode,
                    player.cannot_mine_block(Some(server), block, gamemode),
                ) {
                    let event = PlayerInteractEvent::new(
                        player,
                        InteractAction::LeftClickBlock,
                        block,
                        Some(hit.block_pos),
                    );
                    send_cancellable_blocking! {{
                        &server;
                        event;
                        'after: {
                            player.swing_hand(hand, false);
                        }
                    }}
                    return;
                }
            }
            player.swing_hand(hand, false);
        }
    }
}

/// An arm swing stands for a left click on a block only in ADVENTURE and when the block cannot
/// be mined (otherwise the client also sends a dig packet, which fires the event itself).
const fn left_click_block_from_swing(gamemode: GameMode, cannot_mine: bool) -> bool {
    matches!(gamemode, GameMode::Adventure) && cannot_mine
}

#[cfg(test)]
mod tests {
    use super::{GameMode, left_click_block_from_swing};

    #[test]
    fn swing_is_a_left_click_only_in_adventure_on_unbreakable_blocks() {
        assert!(left_click_block_from_swing(GameMode::Adventure, true));
        assert!(!left_click_block_from_swing(GameMode::Adventure, false));
        for mode in [GameMode::Survival, GameMode::Creative, GameMode::Spectator] {
            assert!(!left_click_block_from_swing(mode, true));
        }
    }
}
