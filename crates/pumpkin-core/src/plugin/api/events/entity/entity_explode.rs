use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

/// An event that occurs when an entity explodes.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityExplodeEvent {
    /// The ID of the exploding entity.
    pub entity_id: i32,

    /// The position where the explosion occurred.
    pub position: Vector3<f64>,

    /// The yield rate of blocks destroyed.
    pub yield_rate: f32,

    /// The blocks the explosion is about to destroy. A plugin removes entries to spare
    /// blocks; entries it adds are ignored. Empty when the explosion does not touch blocks.
    pub blocks: Vec<BlockPos>,

    /// The entity credited with the explosion (who lit the TNT, who shot the fireball), if any.
    pub owner_id: Option<i32>,
}

impl EntityExplodeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, position: Vector3<f64>, yield_rate: f32) -> Self {
        Self {
            entity_id,
            position,
            yield_rate,
            blocks: Vec::new(),
            owner_id: None,
            cancelled: false,
        }
    }

    #[must_use]
    pub fn with_blocks(mut self, blocks: Vec<BlockPos>) -> Self {
        self.blocks = blocks;
        self
    }
}
