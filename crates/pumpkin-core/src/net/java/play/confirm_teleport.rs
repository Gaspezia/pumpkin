#[allow(clippy::wildcard_imports)]
use super::*;

/// Outcome of a client teleport confirmation, vanilla `handleAcceptTeleportPacket`.
#[derive(Debug, PartialEq, Eq)]
enum TeleportConfirm {
    /// Latest id while waiting: snap to the requested position.
    Accept,
    /// Id is not the latest issued: ignored, no kick (stale or unknown id).
    Ignore,
    /// Latest id but nothing awaited (duplicate confirm): vanilla disconnects.
    Kick,
}

/// `latest_id` is the last id sent (vanilla `awaitingTeleport`), `awaiting` whether
/// a confirmation is still pending (vanilla `awaitingPositionFromClient != null`).
const fn classify_confirm(latest_id: i32, awaiting: bool, confirmed_id: i32) -> TeleportConfirm {
    if confirmed_id != latest_id {
        TeleportConfirm::Ignore
    } else if awaiting {
        TeleportConfirm::Accept
    } else {
        TeleportConfirm::Kick
    }
}

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        let mut awaiting_teleport = player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let latest_id = player.teleport_id_count.load(Ordering::Relaxed);
        let result = classify_confirm(
            latest_id,
            awaiting_teleport.is_some(),
            confirm_teleport.teleport_id.0,
        );
        if result == TeleportConfirm::Accept
            && let Some((_, position)) = awaiting_teleport.take()
        {
            // We should set the position now to what we requested in the teleport packet.
            // This may fix issues when the client sends the position while being teleported.
            player.get_entity().set_pos(position);
        }
        drop(awaiting_teleport);

        if result == TeleportConfirm::Kick {
            self.try_kick(&TextComponent::text(
                "Send Teleport confirm, but we did not teleport",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TeleportConfirm::*, classify_confirm};

    #[test]
    fn vanilla_cases() {
        assert_eq!(classify_confirm(3, true, 3), Accept);
        assert_eq!(classify_confirm(3, true, 2), Ignore); // stale (tp-burst)
        assert_eq!(classify_confirm(3, false, 2), Ignore); // stale after latest confirmed
        assert_eq!(classify_confirm(3, false, 3), Kick); // duplicate of latest
        assert_eq!(classify_confirm(3, true, 99), Ignore); // unknown id
    }
}
