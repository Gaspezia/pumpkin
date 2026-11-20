use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId,
    attributes::Attributes,
    damage::DamageType,
    entity::EntityType,
    fluid::Fluid,
    particle::Particle,
    sound::Sound,
    tag::{Tag, Taggable},
};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use pumpkin_world::chunk::ChunkData;
use rustc_hash::FxHashMap;

use crate::{
    block::{
        ExplodeArgs,
        blocks::fire::{fire::FireBlock, soul_fire::SoulFireBlock},
        drop_loot,
    },
    entity::{Entity, EntityBase, projectile::is_redirectable, tnt::TNTEntity},
    world::loot::LootContextParameters,
};

use super::{BlockFlags, World};

/// Defines the type of explosion interaction with the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplosionInteraction {
    None,
    Block,
    Mob,
    Tnt,
    Trigger,
}

/// Defines how an explosion interacts with blocks in the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockInteraction {
    /// Keeps blocks intact (no block damage, no drops).
    Keep,
    /// Destroys blocks and drops 100% of items without decay.
    Destroy,
    /// Destroys blocks and applies loot decay based on explosion radius.
    DestroyWithDecay,
    /// Triggers block effects without destroying them.
    TriggerBlock,
}

/// Defines how damage and block destruction are calculated for an explosion.
pub trait ExplosionDamageCalculator: Send + Sync {
    /// Returns the block's explosion resistance. If None, the block is treated as air/empty.
    fn get_block_explosion_resistance(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        block: &Block,
        fluid: &pumpkin_data::fluid::FluidState,
    ) -> Option<f32> {
        if block.default_state.is_air() && fluid.is_empty {
            None
        } else {
            Some(fluid.blast_resistance.max(block.blast_resistance))
        }
    }

    /// Returns whether this block should be destroyed / affected by the explosion.
    fn should_block_explode(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        _block: &Block,
        _power: f32,
    ) -> bool {
        true
    }

    /// Returns whether the entity should take damage from the explosion.
    fn should_damage_entity(&self, _explosion: &Explosion, _entity: &dyn EntityBase) -> bool {
        true
    }

    /// Returns knockback multiplier for the given entity (default 1.0).
    fn get_knockback_multiplier(&self, _entity: &dyn EntityBase) -> f32 {
        1.0
    }

    /// Calculates the damage amount to deal to the entity given the exposure.
    fn get_entity_damage_amount(
        &self,
        explosion: &Explosion,
        entity: &dyn EntityBase,
        exposure: f32,
    ) -> f32 {
        let radius = explosion.power as f64 * 2.0;
        let distance = (entity
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&explosion.pos))
        .sqrt()
            / radius;
        let damage_multiplier = (1.0 - distance) * exposure as f64;
        (f64::midpoint(damage_multiplier * damage_multiplier, damage_multiplier)
            * 7.0
            * explosion.power as f64
            + 1.0) as f32
    }
}

/// Default explosion damage calculator implementing vanilla standard explosion rules.
pub struct DefaultExplosionDamageCalculator;

impl ExplosionDamageCalculator for DefaultExplosionDamageCalculator {}

/// A configurable explosion damage calculator (e.g. for wind charges, mace wind bursts).
pub struct SimpleExplosionDamageCalculator {
    pub explodes_blocks: bool,
    pub damages_entities: bool,
    pub knockback_multiplier: Option<f32>,
    pub immune_blocks: Option<&'static Tag>,
}

impl SimpleExplosionDamageCalculator {
    #[must_use]
    pub const fn new(
        explodes_blocks: bool,
        damages_entities: bool,
        knockback_multiplier: Option<f32>,
        immune_blocks: Option<&'static Tag>,
    ) -> Self {
        Self {
            explodes_blocks,
            damages_entities,
            knockback_multiplier,
            immune_blocks,
        }
    }
}

impl ExplosionDamageCalculator for SimpleExplosionDamageCalculator {
    fn get_block_explosion_resistance(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        block: &Block,
        fluid: &pumpkin_data::fluid::FluidState,
    ) -> Option<f32> {
        if let Some(immune_tag) = self.immune_blocks {
            return block.has_tag(immune_tag).then_some(3_600_000.0);
        }
        if block.default_state.is_air() && fluid.is_empty {
            None
        } else {
            Some(fluid.blast_resistance.max(block.blast_resistance))
        }
    }

    fn should_block_explode(
        &self,
        _explosion: &Explosion,
        _world: &World,
        _pos: &BlockPos,
        _block: &Block,
        _power: f32,
    ) -> bool {
        self.explodes_blocks
    }

    fn should_damage_entity(&self, _explosion: &Explosion, _entity: &dyn EntityBase) -> bool {
        self.damages_entities
    }

    fn get_knockback_multiplier(&self, _entity: &dyn EntityBase) -> f32 {
        self.knockback_multiplier.unwrap_or(1.0)
    }
}

pub struct Explosion<'a> {
    power: f32,
    pos: Vector3<f64>,
    block_interaction: BlockInteraction,
    damage_calculator: Option<Arc<dyn ExplosionDamageCalculator>>,
    preserve_rails: bool,
    source: Option<&'a dyn EntityBase>,
    /// The entity credited with the explosion when it isn't the source itself.
    indirect_source: Option<Arc<dyn EntityBase>>,
    source_is_indirect: bool,
    damage_type: Option<DamageType>,
    small_particle: Particle,
    large_particle: Particle,
    sound: Sound,
    from_wind_charge: bool,
    /// Vanilla `fire`: scatter fire over the crater of a block-destroying explosion.
    create_fire: bool,
}

impl<'a> Explosion<'a> {
    #[must_use]
    pub const fn new(power: f32, pos: Vector3<f64>, block_interaction: BlockInteraction) -> Self {
        Self {
            power,
            pos,
            block_interaction,
            damage_calculator: None,
            preserve_rails: false,
            source: None,
            indirect_source: None,
            source_is_indirect: false,
            damage_type: None,
            small_particle: Particle::Explosion,
            large_particle: Particle::ExplosionEmitter,
            sound: Sound::EntityGenericExplode,
            from_wind_charge: false,
            create_fire: false,
        }
    }

    #[must_use]
    pub fn with_damage_calculator(
        mut self,
        calculator: Arc<dyn ExplosionDamageCalculator>,
    ) -> Self {
        self.damage_calculator = Some(calculator);
        self
    }

    #[must_use]
    pub const fn preserving_rails(mut self) -> Self {
        self.preserve_rails = true;
        self
    }

    /// Sets the entity that exploded and who gets the credit for it:
    /// primed TNT credits whoever lit it, a living entity itself and a projectile its living owner.
    #[must_use]
    pub fn with_source(mut self, source: &'a dyn EntityBase) -> Self {
        self.source = Some(source);
        self.source_is_indirect = source.get_living_entity().is_some();
        self.indirect_source = if self.source_is_indirect {
            None
        } else {
            let world = source.get_entity().world.load();
            let owner = source.cast_any().downcast_ref::<TNTEntity>().map_or_else(
                || {
                    source
                        .get_projectile_owner()
                        .and_then(|owner| owner.get(&world))
                },
                |tnt| tnt.owner.get(&world),
            );
            owner.filter(|owner| owner.get_living_entity().is_some())
        };
        self
    }

    /// Credits the explosion to `cause` instead of the source's owner, for sources that pass their own damage source.
    #[must_use]
    pub fn with_indirect_source(mut self, cause: Option<Arc<dyn EntityBase>>) -> Self {
        self.source_is_indirect = false;
        self.indirect_source = cause;
        self
    }

    #[must_use]
    pub const fn with_damage_type(mut self, damage_type: DamageType) -> Self {
        self.damage_type = Some(damage_type);
        self
    }

    #[must_use]
    pub const fn center(&self) -> Vector3<f64> {
        self.pos
    }

    #[must_use]
    pub const fn power(&self) -> f32 {
        self.power
    }

    #[must_use]
    pub const fn source(&self) -> Option<&'a dyn EntityBase> {
        self.source
    }

    #[must_use]
    pub fn indirect_source(&self) -> Option<&dyn EntityBase> {
        if self.source_is_indirect {
            self.source
        } else {
            self.indirect_source.as_deref()
        }
    }

    /// Credited explosions with a source deal `player_explosion` damage, which changes the death message.
    #[must_use]
    pub fn damage_type(&self) -> DamageType {
        self.damage_type.unwrap_or(
            if self.source.is_some() && self.indirect_source().is_some() {
                DamageType::PLAYER_EXPLOSION
            } else {
                DamageType::EXPLOSION
            },
        )
    }

    #[must_use]
    pub const fn with_particles_and_sound(
        mut self,
        small_particle: Particle,
        large_particle: Particle,
        sound: Sound,
    ) -> Self {
        self.small_particle = small_particle;
        self.large_particle = large_particle;
        self.sound = sound;
        self
    }

    #[must_use]
    pub const fn from_wind_charge(mut self) -> Self {
        self.from_wind_charge = true;
        self
    }

    /// Makes the explosion light fires in its crater (vanilla `fire`, Bukkit `setFire`).
    #[must_use]
    pub const fn with_fire(mut self) -> Self {
        self.create_fire = true;
        self
    }

    #[must_use]
    pub const fn creates_fire(&self) -> bool {
        self.create_fire
    }

    /// Mirrors `ServerExplosion.isSmall`.
    /// Whether running this explosion reads the world's blocks (anything but `Keep`).
    #[must_use]
    pub const fn touches_blocks(&self) -> bool {
        !matches!(self.block_interaction, BlockInteraction::Keep)
    }

    #[must_use]
    pub const fn is_small(&self) -> bool {
        self.power < 2.0 || matches!(self.block_interaction, BlockInteraction::Keep)
    }

    #[must_use]
    pub const fn particle(&self) -> Particle {
        if self.is_small() {
            self.small_particle
        } else {
            self.large_particle
        }
    }

    #[must_use]
    pub const fn sound(&self) -> Sound {
        self.sound
    }

    fn protects_rail(&self, world: &World, pos: &BlockPos, block: &Block) -> bool {
        self.preserve_rails && (Self::is_rail(block) || Self::is_rail(world.get_block(&pos.up())))
    }

    fn is_rail(block: &Block) -> bool {
        block.id == Block::RAIL.id
            || block.id == Block::POWERED_RAIL.id
            || block.id == Block::DETECTOR_RAIL.id
            || block.id == Block::ACTIVATOR_RAIL.id
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn get_blocks_to_destroy(
        &self,
        world: &World,
    ) -> FxHashMap<BlockPos, (&'static Block, &'static BlockState)> {
        let mut map = FxHashMap::default();

        let mut chunk_cache: FxHashMap<
            pumpkin_util::math::vector2::Vector2<i32>,
            Option<Arc<ChunkData>>,
        > = FxHashMap::default();

        let default_calc = DefaultExplosionDamageCalculator;
        let calc: &dyn ExplosionDamageCalculator = match &self.damage_calculator {
            Some(c) => c.as_ref(),
            None => &default_calc,
        };

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    if x > 0 && x < 15 && y > 0 && y < 15 && z > 0 && z < 15 {
                        continue;
                    }

                    let mut dir_x = f64::from(x) / 7.5 - 1.0;
                    let mut dir_y = f64::from(y) / 7.5 - 1.0;
                    let mut dir_z = f64::from(z) / 7.5 - 1.0;

                    let length = (dir_x * dir_x + dir_y * dir_y + dir_z * dir_z).sqrt();
                    dir_x /= length;
                    dir_y /= length;
                    dir_z /= length;

                    let mut pos_x = self.pos.x;
                    let mut pos_y = self.pos.y;
                    let mut pos_z = self.pos.z;

                    let random_val = rand::random::<f32>();
                    let mut h = self.power * random_val.mul_add(0.6, 0.7);

                    while h > 0.0 {
                        let block_pos = BlockPos::floored(pos_x, pos_y, pos_z);

                        if !world.is_in_build_limit(block_pos) {
                            break;
                        }

                        let (chunk_pos, relative) = block_pos.chunk_and_chunk_relative_position();

                        let chunk_opt = chunk_cache.entry(chunk_pos).or_insert_with(|| {
                            world
                                .level
                                .read_chunk_sync(&chunk_pos, std::clone::Clone::clone)
                        });

                        let state_id = if let Some(chunk) = chunk_opt {
                            chunk
                                .section
                                .get_block_absolute_y(
                                    relative.x as usize,
                                    block_pos.0.y,
                                    relative.z as usize,
                                )
                                .unwrap_or(Block::AIR.default_state.id)
                        } else {
                            Block::AIR.default_state.id
                        };

                        let (block, state) = BlockState::from_id_with_block(state_id);

                        let (_fluid, fluid_state) = Fluid::from_state_id(state_id).map_or_else(
                            || {
                                if block.is_waterlogged(state_id) {
                                    (&Fluid::FLOWING_WATER, &Fluid::FLOWING_WATER.states[0])
                                } else {
                                    (&Fluid::EMPTY, &Fluid::EMPTY.states[0])
                                }
                            },
                            |raw_fluid| {
                                let f = raw_fluid.to_flowing();
                                (f, &f.states[0])
                            },
                        );

                        if !state.is_air() || !fluid_state.is_empty {
                            let protects_rail = self.protects_rail(world, &block_pos, block);
                            let resistance = if protects_rail {
                                Some(0.0)
                            } else {
                                calc.get_block_explosion_resistance(
                                    self,
                                    world,
                                    &block_pos,
                                    block,
                                    fluid_state,
                                )
                            };

                            // The exploding entity adjusts resistance
                            // unless the explosion brought its own calculator.
                            let resistance = match (self.source, &self.damage_calculator) {
                                (Some(source), None) => resistance
                                    .map(|r| source.get_block_explosion_resistance(block, r)),
                                _ => resistance,
                            };
                            if let Some(resistance) = resistance {
                                h -= (resistance + 0.3) * 0.3;
                            }

                            if h > 0.0
                                && !protects_rail
                                && calc.should_block_explode(self, world, &block_pos, block, h)
                            {
                                map.insert(block_pos, (block, state));
                            }
                        }

                        pos_x += dir_x * 0.3;
                        pos_y += dir_y * 0.3;
                        pos_z += dir_z * 0.3;
                        h -= 0.225_000_01;
                    }
                }
            }
        }
        map
    }

    fn damage_entities(&self, world: &Arc<World>) {
        // Explosion is too small
        if self.power < 1.0e-5 {
            return;
        }

        let radius = self.power as f64 * 2.0;
        let min_x = (self.pos.x - radius - 1.0).floor() as i32;
        let max_x = (self.pos.x + radius + 1.0).floor() as i32;
        let min_y = (self.pos.y - radius - 1.0).floor() as i32;
        let max_y = (self.pos.y + radius + 1.0).floor() as i32;
        let min_z = (self.pos.z - radius - 1.0).floor() as i32;
        let max_z = (self.pos.z + radius + 1.0).floor() as i32;

        let search_box = BoundingBox::new(
            Vector3::new(min_x as f64, min_y as f64, min_z as f64),
            Vector3::new(max_x as f64, max_y as f64, max_z as f64),
        );

        let entities = world.get_all_at_box(&search_box);

        let default_calc = DefaultExplosionDamageCalculator;
        let calc: &dyn ExplosionDamageCalculator = match &self.damage_calculator {
            Some(c) => c.as_ref(),
            None => &default_calc,
        };

        for entity_base in entities {
            if entity_base.is_immune_to_explosion() {
                continue;
            }

            // Skip spectators (no damage, no knockback)
            if entity_base.is_spectator() {
                continue;
            }

            let entity = entity_base.get_entity();
            if self
                .source
                .is_some_and(|source| source.get_entity().entity_id == entity.entity_id)
            {
                continue;
            }

            let distance = (entity.pos.load().squared_distance_to_vec(&self.pos)).sqrt() / radius;
            if distance > 1.0 {
                continue;
            }

            let should_damage = calc.should_damage_entity(self, entity_base.as_ref());
            let knockback_multiplier = calc.get_knockback_multiplier(entity_base.as_ref()) as f64;

            let exposure = if !should_damage && knockback_multiplier == 0.0 {
                0.0
            } else {
                Self::calculate_exposure(&self.pos, entity, world) as f64
            };

            if exposure == 0.0 {
                continue;
            }

            let damage_refused = if should_damage
                && self
                    .source
                    .is_none_or(|source| source.do_teams_allow_damage(entity_base.as_ref()))
            {
                let damage =
                    calc.get_entity_damage_amount(self, entity_base.as_ref(), exposure as f32);
                let damaged = entity_base.damage_with_context(
                    entity_base.as_ref(),
                    damage,
                    self.damage_type(),
                    None,
                    self.source,
                    self.indirect_source(),
                );
                !damaged
            } else {
                false
            };

            // Paper parity: a damage event cancelled by a plugin also cancels the knockback.
            if !knockback_allowed(damage_refused) {
                continue;
            }

            // Calculate and apply knockback
            let dir_pos = if entity.entity_type == &EntityType::TNT {
                entity.pos.load()
            } else {
                entity.get_eye_pos()
            };
            let direction = (dir_pos - self.pos).normalize();

            let knockback_resistance = entity_base.get_living_entity().map_or(0.0, |living| {
                living.get_attribute_value(&Attributes::EXPLOSION_KNOCKBACK_RESISTANCE)
            });
            let knockback_power =
                (1.0 - distance) * exposure * knockback_multiplier * (1.0 - knockback_resistance);
            let knockback = direction * knockback_power;
            // Vanilla `ServerExplosion.hurtEntities`: creative flyers get no knockback.
            if !entity_base
                .get_player()
                .is_some_and(|player| player.is_creative() && player.is_flying())
            {
                // A living entity's goes through `EntityKnockbackEvent`
                let knockback = if entity_base.get_living_entity().is_some() {
                    entity.fire_knockback_events(knockback_power, knockback, None)
                } else {
                    Some(knockback)
                };
                if let Some(knockback) = knockback {
                    entity.add_velocity(knockback);
                }
            }

            self.on_explosion_hit(entity_base.as_ref());
        }
    }

    /// A pushed redirectable projectile now belongs to whoever caused the explosion, and a player remembers the
    /// impulse for fall damage.
    fn on_explosion_hit(&self, entity_base: &dyn EntityBase) {
        if is_redirectable(entity_base.get_entity())
            && let Some(owner) = entity_base.get_projectile_owner()
        {
            owner.set_uuid(
                self.indirect_source()
                    .map(|cause| cause.get_entity().entity_uuid),
            );
        } else if let Some(player) = entity_base.get_player() {
            player
                .living_entity
                .set_ignore_fall_damage_from_current_impulse(
                    self.from_wind_charge,
                    player.position(),
                );
        }
    }

    fn calculate_exposure(
        explosion_pos: &Vector3<f64>,
        entity: &Entity,
        world: &Arc<World>,
    ) -> f32 {
        let bbox = entity.bounding_box.load();

        let step_x = 1.0 / ((bbox.max.x - bbox.min.x) * 2.0 + 1.0);
        let step_y = 1.0 / ((bbox.max.y - bbox.min.y) * 2.0 + 1.0);
        let step_z = 1.0 / ((bbox.max.z - bbox.min.z) * 2.0 + 1.0);

        if step_x < 0.0 || step_y < 0.0 || step_z < 0.0 {
            return 0.0;
        }

        let offset_x = (1.0 - (1.0 / step_x).floor() * step_x) / 2.0;
        let offset_z = (1.0 - (1.0 / step_z).floor() * step_z) / 2.0;

        let mut visible_points = 0;
        let mut total_points = 0;

        let mut k = 0.0;
        while k <= 1.0 {
            let mut l = 0.0;
            while l <= 1.0 {
                let mut m = 0.0;
                while m <= 1.0 {
                    let n = bbox.min.x + (bbox.max.x - bbox.min.x) * k;
                    let o = bbox.min.y + (bbox.max.y - bbox.min.y) * l;
                    let p = bbox.min.z + (bbox.max.z - bbox.min.z) * m;

                    let vec3d = Vector3::new(n + offset_x, o, p + offset_z);

                    if world
                        .clip(&pumpkin_world::clip::ClipContext::new(
                            vec3d,
                            *explosion_pos,
                            pumpkin_world::clip::ClipBlock::Collider,
                            pumpkin_world::clip::ClipFluid::None,
                        ))
                        .is_miss
                    {
                        visible_points += 1;
                    }

                    total_points += 1;
                    m += step_z;
                }
                l += step_y;
            }
            k += step_x;
        }

        if total_points == 0 {
            return 0.0;
        }

        visible_points as f32 / total_points as f32
    }

    /// Returns the removed block count
    /// Runs the explosion. `blocks` is the (possibly plugin-trimmed) block set from
    /// [`Self::get_blocks_to_destroy`]; `None` computes it here.
    pub fn explode(
        &self,
        world: &Arc<World>,
        blocks: Option<FxHashMap<BlockPos, (&'static Block, &'static BlockState)>>,
    ) -> u32 {
        self.damage_entities(world);

        match self.block_interaction {
            BlockInteraction::Keep => 0,
            BlockInteraction::TriggerBlock => {
                let blocks = blocks.unwrap_or_else(|| self.get_blocks_to_destroy(world));
                for (pos, (block, _state)) in &blocks {
                    let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);
                    if let Some(pumpkin_block) = pumpkin_block {
                        pumpkin_block.explode(ExplodeArgs {
                            world,
                            block,
                            position: pos,
                            explosion: self,
                        });
                    }
                }
                0
            }
            BlockInteraction::Destroy | BlockInteraction::DestroyWithDecay => {
                let center_pos = BlockPos::floored(self.pos.x, self.pos.y, self.pos.z);
                let mut event =
                    crate::plugin::api::events::block::block_explode::BlockExplodeEvent::new(
                        center_pos,
                        if self.power > 0.0 {
                            1.0 / self.power
                        } else {
                            1.0
                        },
                    );
                if let Some(server) = world.server.upgrade() {
                    server.plugin_manager.fire_blocking(&server, &mut event);
                }
                if event.cancelled {
                    return 0;
                }

                let blocks = blocks.unwrap_or_else(|| self.get_blocks_to_destroy(world));
                let decay_drops = self.block_interaction == BlockInteraction::DestroyWithDecay;
                let explosion_radius = decay_drops.then_some(self.power);

                for (pos, (block, state)) in &blocks {
                    let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);

                    if pumpkin_block.is_none_or(|s| s.should_drop_items_on_explosion()) {
                        let is_raining = world.is_raining();
                        let is_thundering = world.is_thundering();
                        let params = LootContextParameters {
                            block_state: Some(state),
                            explosion_radius,
                            position: Some(pumpkin_util::math::vector3::Vector3::new(
                                pos.0.x as f64,
                                pos.0.y as f64,
                                pos.0.z as f64,
                            )),
                            world_time: world.level_info.load().day_time as u64,
                            is_raining: Some(is_raining),
                            is_thundering: Some(is_thundering),
                            ..Default::default()
                        };
                        drop_loot(world, block, pos, false, &params);
                    }

                    world.set_block_state(pos, BlockStateId::AIR, BlockFlags::NOTIFY_ALL);
                    world.close_container_screens_at(pos);

                    if let Some(pumpkin_block) = pumpkin_block {
                        pumpkin_block.explode(ExplodeArgs {
                            world,
                            block,
                            position: pos,
                            explosion: self,
                        });
                    }
                }
                if self.create_fire {
                    for pos in fire_positions(
                        blocks.keys().copied(),
                        || rand::random::<u32>() % 3,
                        |pos| world.get_block_state(pos).is_air(),
                        |pos| {
                            world
                                .get_block_state(&pos.down())
                                .is_side_solid(BlockDirection::Up)
                        },
                    ) {
                        let fire = if SoulFireBlock::is_soul_base(world.get_block(&pos.down())) {
                            Block::SOUL_FIRE.default_state.id
                        } else {
                            FireBlock.get_state_for_position(world.as_ref(), &Block::FIRE, &pos)
                        };
                        world.set_block_state(&pos, fire, BlockFlags::NOTIFY_ALL);
                    }
                }
                blocks.len() as u32
            }
        }
    }
}

/// An explosion pushes an entity unless it tried to damage it and the damage was refused.
const fn knockback_allowed(damage_refused: bool) -> bool {
    !damage_refused
}

/// Vanilla `ServerExplosion.interactWithBlocks` fire rule: each destroyed position has a 1 in 3
/// chance (`roll() == 0`) to burn, if it is now air above a solid face.
fn fire_positions(
    destroyed: impl IntoIterator<Item = BlockPos>,
    mut roll: impl FnMut() -> u32,
    is_air: impl Fn(&BlockPos) -> bool,
    solid_below: impl Fn(&BlockPos) -> bool,
) -> Vec<BlockPos> {
    destroyed
        .into_iter()
        .filter(|pos| roll() == 0 && is_air(pos) && solid_below(pos))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        BlockInteraction, Explosion, ExplosionDamageCalculator, SimpleExplosionDamageCalculator,
    };
    use pumpkin_data::{Block, fluid::Fluid, particle::Particle, sound::Sound, tag};
    use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

    #[test]
    fn tnt_minecart_rail_protection_covers_every_rail_type() {
        for rail in [
            &Block::RAIL,
            &Block::POWERED_RAIL,
            &Block::DETECTOR_RAIL,
            &Block::ACTIVATOR_RAIL,
        ] {
            assert!(Explosion::is_rail(rail));
        }
        assert!(!Explosion::is_rail(&Block::STONE));
    }

    #[test]
    fn refused_damage_cancels_the_knockback() {
        assert!(!super::knockback_allowed(true));
        assert!(super::knockback_allowed(false));
    }

    #[test]
    fn fire_is_scattered_on_one_in_three_air_positions_above_solid() {
        let positions: Vec<BlockPos> = (0..6).map(|x| BlockPos::new(x, 64, 0)).collect();
        let mut rolls = [0, 1, 0, 2, 0, 0].into_iter();
        let burning = super::fire_positions(
            positions.clone(),
            || rolls.next().unwrap(),
            |pos| pos.0.x != 4, // x = 4 is not air
            |pos| pos.0.x != 5, // x = 5 has no solid floor
        );
        // x = 1 and 3 fail the roll, 4 is not air, 5 has no floor.
        assert_eq!(burning, vec![positions[0], positions[2]]);

        assert!(
            !Explosion::new(1.0, Vector3::new(0.0, 0.0, 0.0), BlockInteraction::Destroy)
                .creates_fire()
        );
        assert!(
            Explosion::new(1.0, Vector3::new(0.0, 0.0, 0.0), BlockInteraction::Destroy)
                .with_fire()
                .creates_fire()
        );
    }

    #[tokio::test]
    async fn wind_burst_explosion_uses_large_gust_particle_and_immune_block_resistance()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let explosion = Explosion::new(
            3.5,
            Vector3::new(0.0, 64.0, 0.0),
            BlockInteraction::TriggerBlock,
        )
        .with_particles_and_sound(
            Particle::GustEmitterSmall,
            Particle::GustEmitterLarge,
            Sound::EntityWindChargeWindBurst,
        );
        assert!(!explosion.is_small());
        assert_eq!(explosion.particle(), Particle::GustEmitterLarge);
        assert_eq!(explosion.sound(), Sound::EntityWindChargeWindBurst);

        let calc = SimpleExplosionDamageCalculator::new(
            true,
            false,
            Some(1.2),
            Some(&tag::Block::MINECRAFT_BLOCKS_WIND_CHARGE_EXPLOSIONS),
        );
        let pos = BlockPos::new(0, 64, 0);
        let empty_fluid = &Fluid::EMPTY.states[0];
        assert_eq!(
            calc.get_block_explosion_resistance(
                &explosion,
                &world,
                &pos,
                &Block::BEDROCK,
                empty_fluid
            ),
            Some(3_600_000.0)
        );
        assert_eq!(
            calc.get_block_explosion_resistance(
                &explosion,
                &world,
                &pos,
                &Block::LEVER,
                empty_fluid
            ),
            None
        );
        assert!(calc.should_block_explode(&explosion, &world, &pos, &Block::LEVER, 1.0));

        world.level.shutdown().await;
        Ok(())
    }
}
