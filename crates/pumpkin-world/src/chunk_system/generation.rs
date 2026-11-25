use crate::ProtoChunk;
use crate::generation::generator::WorldGenerator;
use crate::world::WorldPortalExt;
use pumpkin_config::lighting::LightingEngineConfig;

use super::{Cache, Chunk, StagedChunkEnum};

pub fn generate_single_chunk(
    generator: &WorldGenerator,
    block_registry: &dyn WorldPortalExt,
    chunk_x: i32,
    chunk_z: i32,
    target_stage: StagedChunkEnum,
) -> Chunk {
    generate_single_chunk_with_radius(
        generator,
        block_registry,
        chunk_x,
        chunk_z,
        target_stage,
        target_stage.get_direct_radius(),
    )
}

pub fn generate_single_chunk_with_radius(
    generator: &WorldGenerator,
    block_registry: &dyn WorldPortalExt,
    chunk_x: i32,
    chunk_z: i32,
    target_stage: StagedChunkEnum,
    radius: i32,
) -> Chunk {
    let mut cache = Cache::new(chunk_x - radius, chunk_z - radius, radius * 2 + 1);

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let new_x = chunk_x + dx;
            let new_z = chunk_z + dz;

            let proto_chunk = Box::new(ProtoChunk::new(new_x, new_z, generator));

            cache.chunks.push(Chunk::Proto(proto_chunk));
        }
    }

    let stages = [
        StagedChunkEnum::Biomes,
        StagedChunkEnum::StructureStart,
        StagedChunkEnum::StructureReferences,
        StagedChunkEnum::Noise,
        StagedChunkEnum::Surface,
        StagedChunkEnum::Carvers,
        StagedChunkEnum::Features,
        StagedChunkEnum::Lighting,
        StagedChunkEnum::Spawn,
        StagedChunkEnum::Full,
    ];

    for &stage in &stages {
        if stage as u8 > target_stage as u8 {
            break;
        }

        if matches!(
            stage,
            StagedChunkEnum::Biomes
                | StagedChunkEnum::StructureStart
                | StagedChunkEnum::StructureReferences
        ) {
            cache.advance_all(
                stage,
                generator,
                block_registry,
                &LightingEngineConfig::Default,
            );
        } else {
            cache.advance(
                stage,
                generator,
                block_registry,
                &LightingEngineConfig::Default,
            );
        }
    }

    let mid = ((cache.size * cache.size) >> 1) as usize;
    cache.chunks.swap_remove(mid)
}

#[cfg(test)]
mod tests {
    use crate::chunk::ChunkHeightmapType;
    use crate::chunk_system::Chunk;
    use crate::chunk_system::{
        StagedChunkEnum, generate_single_chunk, generation::generate_single_chunk_with_radius,
    };
    use crate::generation::get_world_gen;
    use crate::world::WorldPortalExt;
    use pumpkin_data::BlockStateId;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;
    use std::sync::Arc;

    struct BlockRegistry;
    impl WorldPortalExt for BlockRegistry {
        fn can_place_at(
            &self,
            _block: &pumpkin_data::Block,
            _state: &pumpkin_data::BlockState,
            _block_accessor: &dyn crate::world::BlockAccessor,
            _block_pos: &pumpkin_util::math::position::BlockPos,
        ) -> bool {
            true
        }

        fn mirror(
            &self,
            block: &pumpkin_data::Block,
            state_id: BlockStateId,
            mirror: pumpkin_data::Mirror,
        ) -> &'static pumpkin_data::BlockState {
            block.mirror(state_id, mirror)
        }

        fn rotate(
            &self,
            block: &pumpkin_data::Block,
            state_id: BlockStateId,
            rotation: pumpkin_data::Rotation,
        ) -> &'static pumpkin_data::BlockState {
            block.rotate(state_id, rotation)
        }

        fn spawn_mobs_for_chunk_generation(
            &self,
            _cache: &mut dyn crate::generation::proto_chunk::GenerationCache,
            _biome: &'static pumpkin_data::chunk::Biome,
            _chunk_x: i32,
            _chunk_z: i32,
        ) {
        }
    }

    #[test]
    fn nether_lighting_clears_stale_sky_light_and_preserves_block_light() {
        use crate::chunk_system::Cache;
        use pumpkin_config::lighting::LightingEngineConfig;
        use pumpkin_data::Block;

        let dimension = Dimension::THE_NETHER;
        let world_gen = get_world_gen(
            Seed(42),
            dimension.clone(),
            false,
            Vec::new(),
            String::new(),
        );
        let mut cache = Cache::new(-1, -1, 3);
        for x in -1..=1 {
            for z in -1..=1 {
                cache
                    .chunks
                    .push(Chunk::Proto(Box::new(crate::ProtoChunk::new(
                        x, z, &world_gen,
                    ))));
            }
        }
        let chunk = cache.chunks[4].get_proto_chunk_mut();
        chunk.stage = StagedChunkEnum::Features;
        for section in &mut chunk.light.sky_light {
            section.fill(15);
        }
        let y = dimension.min_y + 1;
        chunk.set_block_state(8, y, 8, Block::LAVA.default_state);
        cache.advance(
            StagedChunkEnum::Lighting,
            &world_gen,
            &BlockRegistry,
            &LightingEngineConfig::Default,
        );
        let chunk = cache.chunks[4].get_proto_chunk_mut();
        assert_eq!(chunk.light.sky_light.len(), dimension.height as usize / 16);
        assert!(
            chunk.light.sky_light.iter().all(|section| {
                matches!(section, crate::chunk::format::LightContainer::Empty(0))
            })
        );
        assert_eq!(
            chunk.light.block_light[0].get(8, 1, 8),
            Block::LAVA.default_state.luminance
        );
    }

    /// Builds a 3x3 `Cache` of loaded-looking chunks (light reset exactly as the relight branch of
    /// `process_loaded_chunk` does), runs the Lighting stage and returns the centre chunk.
    fn relight_3x3(fill: impl Fn(i32, i32, &mut crate::ProtoChunk)) -> crate::ProtoChunk {
        let Chunk::Proto(p) = relight_3x3_cache(fill).chunks.swap_remove(4) else {
            panic!("expected proto")
        };
        *p
    }

    /// Same as `relight_3x3` but returns the whole cache; the chunk at (x, z) is at index
    /// `(x + 1) * 3 + (z + 1)`.
    fn relight_3x3_cache(
        fill: impl Fn(i32, i32, &mut crate::ProtoChunk),
    ) -> crate::chunk_system::Cache {
        relight_3x3_cache_with(fill, |_, _| false)
    }

    /// Like `relight_3x3_cache`, but the chunks for which `is_loaded` is true are `Level` chunks
    /// (already sent to players) when the centre is relit.
    fn relight_3x3_cache_with(
        fill: impl Fn(i32, i32, &mut crate::ProtoChunk),
        is_loaded: impl Fn(i32, i32) -> bool,
    ) -> crate::chunk_system::Cache {
        use crate::chunk_system::Cache;
        use crate::chunk_system::worker_logic::reset_light_for_relight;
        use pumpkin_config::lighting::LightingEngineConfig;

        let world_gen = get_world_gen(
            Seed(42),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        let mut cache = Cache::new(-1, -1, 3);
        for x in -1..=1 {
            for z in -1..=1 {
                let mut chunk = crate::ProtoChunk::new(x, z, &world_gen);
                fill(x, z, &mut chunk);
                chunk.recompute_heightmaps_from_blocks();
                reset_light_for_relight(&mut chunk);
                chunk.stage = StagedChunkEnum::Features;
                let mut chunk = Chunk::Proto(Box::new(chunk));
                if is_loaded(x, z) {
                    chunk.upgrade_to_level_chunk(
                        world_gen.dimension(),
                        &LightingEngineConfig::Default,
                    );
                }
                cache.chunks.push(chunk);
            }
        }
        cache.advance(
            StagedChunkEnum::Lighting,
            &world_gen,
            &BlockRegistry,
            &LightingEngineConfig::Default,
        );
        cache
    }

    fn sky_at(chunk: &crate::ProtoChunk, x: i32, y: i32, z: i32) -> u8 {
        chunk.light.sky_light[((y - chunk.bottom_y() as i32) >> 4) as usize].get(
            x as usize,
            (y & 15) as usize,
            z as usize,
        )
    }

    #[test]
    fn sky_light_propagates_deep_into_covered_neighbour() {
        use pumpkin_data::Block;
        let cache = relight_3x3_cache(|x, _, c| {
            for lx in 0..16 {
                for lz in 0..16 {
                    c.set_block_state(lx, 0, lz, Block::STONE.default_state);
                    if x == -1 {
                        c.set_block_state(lx, 100, lz, Block::STONE.default_state);
                    }
                }
            }
        });
        let Chunk::Proto(n) = &cache.chunks[1] else {
            panic!("expected proto")
        };
        for (x, want) in [(15, 14), (14, 13), (12, 11), (5, 4)] {
            assert_eq!(
                sky_at(n, x, 99, 8),
                want,
                "sky under neighbour roof at x={x}"
            );
        }
    }

    #[test]
    fn light_written_into_loaded_neighbour_is_reported_for_broadcast() {
        use pumpkin_data::Block;
        use pumpkin_util::math::vector2::Vector2;
        // Covered neighbour (-1,0) is already a Level chunk: the centre relight writes sky light
        // into it, which players holding it must be told about. Untouched neighbours must not
        // be reported, nor Proto chunks (they are sent when they become Level).
        let cache = relight_3x3_cache_with(
            |x, _, c| {
                for lx in 0..16 {
                    for lz in 0..16 {
                        c.set_block_state(lx, 0, lz, Block::STONE.default_state);
                        if x == -1 {
                            c.set_block_state(lx, 100, lz, Block::STONE.default_state);
                        }
                    }
                }
            },
            |x, z| x != 0 || z != 0,
        );
        assert!(cache.light_changed.contains(&Vector2::new(-1, 0)));
        assert!(!cache.light_changed.contains(&Vector2::new(0, 0)));
    }

    #[test]
    fn sky_light_dull_pillar_does_not_freeze_neighbour_cells() {
        use pumpkin_data::Block;
        // Covered chunk (-1,0) with a pillar on its x=15 edge at z=8: the cell next to it must
        // still get light from the bright edge cells beside the pillar.
        let cache = relight_3x3_cache(|x, _, c| {
            for lx in 0..16 {
                for lz in 0..16 {
                    c.set_block_state(lx, 0, lz, Block::STONE.default_state);
                    if x == -1 {
                        c.set_block_state(lx, 100, lz, Block::STONE.default_state);
                    }
                }
            }
            if x == -1 {
                for y in 1..100 {
                    c.set_block_state(15, y, 8, Block::STONE.default_state);
                }
            }
        });
        let Chunk::Proto(n) = &cache.chunks[1] else {
            panic!("expected proto")
        };
        assert_eq!(sky_at(n, 14, 99, 8), 12, "cell beside pillar column");
        assert_eq!(sky_at(n, 10, 99, 8), 8);
    }

    #[test]
    fn relit_chunk_under_full_opaque_roof_has_no_sky_light() {
        use pumpkin_data::Block;
        let chunk = relight_3x3(|_, _, c| {
            for lx in 0..16 {
                for lz in 0..16 {
                    c.set_block_state(lx, 0, lz, Block::STONE.default_state);
                    c.set_block_state(lx, 100, lz, Block::STONE.default_state);
                }
            }
        });
        for y in [1, 30, 64, 99] {
            assert_eq!(sky_at(&chunk, 8, y, 8), 0, "sky under roof at y={y}");
        }
        assert_eq!(sky_at(&chunk, 8, 101, 8), 15, "open sky above roof");
    }

    #[test]
    fn loaded_map_with_stale_heightmaps_is_relit_from_real_blocks() {
        use pumpkin_data::Block;

        let chunk = relight_3x3(|x, z, chunk| {
            for lx in 0..16 {
                for lz in 0..16 {
                    chunk.set_block_state(lx, 0, lz, Block::STONE.default_state);
                }
            }
            if x == 0 && z == 0 {
                // 5x5 opaque slab at y=20 with a lantern on it.
                for lx in 6..=10 {
                    for lz in 6..=10 {
                        chunk.set_block_state(lx, 20, lz, Block::STONE.default_state);
                    }
                }
                chunk.set_block_state(8, 21, 8, Block::SEA_LANTERN.default_state);
                // What a stale map stores: a top far below the slab and the lantern.
                chunk.flat_surface_height_map = [0; 256];
                chunk.flat_surface_wg_height_map = [0; 256];
                chunk.flat_motion_blocking_height_map = [0; 256];
                chunk.flat_ocean_floor_height_map = [0; 256];
            }
        });
        let sky = |x: i32, y: i32, z: i32| sky_at(&chunk, x, y, z);
        let block = |x: i32, y: i32, z: i32| {
            chunk.light.block_light[((y - chunk.bottom_y() as i32) >> 4) as usize].get(
                x as usize,
                (y & 15) as usize,
                z as usize,
            )
        };
        assert_eq!(sky(8, 20, 8), 0, "no sky light inside a solid block");
        assert_eq!(sky(8, 22, 8), 15, "open sky above the slab");
        // Lateral spread under the slab, one level lost per cell (vanilla).
        assert_eq!(sky(5, 19, 8), 15);
        assert_eq!(sky(6, 19, 8), 14);
        assert_eq!(sky(7, 19, 8), 13);
        assert_eq!(sky(8, 19, 8), 12);
        assert_eq!(sky(8, 1, 8), 12);
        // The lantern sits above the stale top and must still emit.
        assert_eq!(block(8, 21, 8), 15);
        assert_eq!(block(8, 22, 8), 14);
    }

    #[test]
    fn dimensions_taller_than_their_noise_settings_generate_all_sections() {
        for (dimension, terrain_state) in [
            (
                Dimension::THE_NETHER,
                pumpkin_data::Block::NETHERRACK.default_state.id,
            ),
            (
                Dimension::THE_END,
                pumpkin_data::Block::END_STONE.default_state.id,
            ),
        ] {
            let seed = Seed(42);
            let block_registry = Arc::new(BlockRegistry);
            let world_gen =
                get_world_gen(seed, dimension.clone(), false, Vec::new(), String::new());

            let chunk = generate_single_chunk(
                &world_gen,
                block_registry.as_ref(),
                0,
                0,
                StagedChunkEnum::Full,
            );
            let Chunk::Level(chunk) = chunk else {
                panic!("full generation must return a level chunk");
            };

            assert_eq!(chunk.section.min_y, dimension.min_y);
            assert_eq!(chunk.section.count, dimension.height as usize / 16);
            assert_eq!(
                chunk.light_engine.lock().unwrap().sky_light.len(),
                chunk.section.count
            );

            let dumped = chunk.section.dump_blocks();
            assert!(dumped.contains(&terrain_state));
            let top_section = &dumped[dumped.len() - 16 * 16 * 16..];
            assert!(top_section.iter().all(|&state| state == BlockStateId::AIR));
        }
    }

    #[test]
    fn generate_chunk_should_return() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(42);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension, false, Vec::new(), String::new());

        let chunk = generate_single_chunk(
            &world_gen,
            block_registry.as_ref(),
            0,
            0,
            StagedChunkEnum::Full,
        );
        let Chunk::Level(chunk) = chunk else {
            panic!("full generation must return a level chunk");
        };
        let recalculated = chunk.calculate_heightmap();
        let generated = chunk.heightmap.lock().unwrap();
        for x in 0..16 {
            for z in 0..16 {
                for heightmap_type in [
                    ChunkHeightmapType::WorldSurface,
                    ChunkHeightmapType::MotionBlocking,
                    ChunkHeightmapType::MotionBlockingNoLeaves,
                ] {
                    assert_eq!(
                        generated.get(heightmap_type, x, z, chunk.section.min_y),
                        recalculated.get(heightmap_type, x, z, chunk.section.min_y),
                    );
                }
            }
        }
    }

    #[test]
    fn configured_seed_generates_vanilla_ancient_city_chunk() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(1_782_124_772_053_846_960);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension, false, Vec::new(), String::new());

        let chunk = generate_single_chunk(
            &world_gen,
            block_registry.as_ref(),
            31,
            -12,
            StagedChunkEnum::Features,
        );
        let super::Chunk::Proto(chunk) = chunk else {
            panic!("features stage should return a proto chunk");
        };

        let mut city_blocks = 0;
        let mut jigsaw_blocks = 0;
        for x in 496..512 {
            for z in -192..-176 {
                for y in -64..320 {
                    let block = chunk
                        .get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z))
                        .to_block_id();
                    if [
                        pumpkin_data::Block::DEEPSLATE_BRICKS.id,
                        pumpkin_data::Block::POLISHED_DEEPSLATE.id,
                        pumpkin_data::Block::REINFORCED_DEEPSLATE.id,
                        pumpkin_data::Block::SCULK.id,
                    ]
                    .contains(&block)
                    {
                        city_blocks += 1;
                    }
                    if block == pumpkin_data::Block::JIGSAW.id {
                        jigsaw_blocks += 1;
                    }
                }
            }
        }

        assert!(
            city_blocks > 0,
            "reference chunk contains no Ancient City blocks"
        );
        assert_eq!(jigsaw_blocks, 0, "jigsaw blocks were not replaced");
    }

    #[test]
    fn seed_zero_generates_the_vanilla_pillager_outpost_chunk() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(1_782_124_772_053_846_960);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension, false, Vec::new(), String::new());

        let chunk = generate_single_chunk_with_radius(
            &world_gen,
            block_registry.as_ref(),
            73,
            -82,
            StagedChunkEnum::Spawn,
            16,
        );
        let super::Chunk::Proto(chunk) = chunk else {
            panic!("spawn stage should return a proto chunk");
        };
        let mut outpost_blocks = 0;
        let mut jigsaw_blocks = 0;
        for x in 1168..1184 {
            for z in -1328..-1312 {
                for y in -64..320 {
                    let block = chunk
                        .get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z))
                        .to_block_id();
                    if [
                        pumpkin_data::Block::DARK_OAK_LOG.id,
                        pumpkin_data::Block::DARK_OAK_PLANKS.id,
                        pumpkin_data::Block::DARK_OAK_FENCE.id,
                    ]
                    .contains(&block)
                    {
                        outpost_blocks += 1;
                    }
                    if block == pumpkin_data::Block::JIGSAW.id {
                        jigsaw_blocks += 1;
                    }
                }
            }
        }

        assert!(
            outpost_blocks > 0,
            "reference chunk contains no outpost blocks"
        );
        assert_eq!(jigsaw_blocks, 0, "jigsaw blocks were not replaced");
    }

    #[test]
    fn fixed_seed_generates_vanilla_end_ship_chunk() {
        // Vanilla 26.2 places this seed's ship in chunk (-306, -275).
        let dimension = Dimension::THE_END;
        let seed = Seed(12_345);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension, false, Vec::new(), String::new());
        let chunk = generate_single_chunk_with_radius(
            &world_gen,
            block_registry.as_ref(),
            -306,
            -275,
            StagedChunkEnum::Features,
            16,
        );
        let Chunk::Proto(chunk) = chunk else {
            panic!("features stage should return a proto chunk");
        };
        let mut hash = 0xcbf29ce484222325u64;
        let mut non_air = 0;
        for y in 123..=146 {
            for x in -4896..=-4881 {
                for z in -4400..=-4393 {
                    let state =
                        chunk.get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z));
                    hash ^= u64::from(state.as_u16());
                    hash = hash.wrapping_mul(0x100000001b3);
                    non_air += usize::from(!state.to_state().is_air());
                }
            }
        }
        assert_eq!(non_air, 59);
        // Depends on block state ids, so it moves whenever the registry is renumbered.
        assert_eq!(hash, 0x2616_4d4b_ea29_023f);
        assert!(chunk.pending_block_entities.iter().any(|nbt| {
            nbt.get_string("id") == Some("minecraft:skull")
                && nbt.get_int("x") == Some(-4888)
                && nbt.get_int("y") == Some(131)
                && nbt.get_int("z") == Some(-4399)
        }));
        assert_eq!(
            chunk
                .pending_block_entities
                .iter()
                .filter(
                    |nbt| nbt.get_string("LootTable") == Some("minecraft:chests/end_city_treasure")
                )
                .count(),
            2
        );
    }

    #[test]
    fn pillager_outpost_features_shape_ground_at_vanilla_height() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(1_782_124_772_053_846_960);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension, false, Vec::new(), String::new());

        let chunk = generate_single_chunk(
            &world_gen,
            block_registry.as_ref(),
            73,
            -82,
            StagedChunkEnum::Features,
        );
        let super::Chunk::Proto(chunk) = chunk else {
            panic!("features stage should return a proto chunk");
        };

        for (x, y, z) in [(1173, 70, -1311), (1173, 70, -1305)] {
            let state = chunk.get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z));
            assert_eq!(state.to_block_id(), pumpkin_data::Block::GRASS_BLOCK.id);
        }

        let cage_chunk = generate_single_chunk(
            &world_gen,
            block_registry.as_ref(),
            73,
            -84,
            StagedChunkEnum::Features,
        );
        let super::Chunk::Proto(cage_chunk) = cage_chunk else {
            panic!("features stage should return a proto chunk");
        };
        let state =
            cage_chunk.get_block_state(&pumpkin_util::math::vector3::Vector3::new(1183, 68, -1330));
        assert_eq!(state.to_block_id(), pumpkin_data::Block::GRASS_BLOCK.id);
    }
}
