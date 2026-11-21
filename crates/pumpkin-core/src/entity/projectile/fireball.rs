use crate::entity::projectile::owner::ProjectileOwner;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_data::damage::DamageType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_util::math::atomic_f32::AtomicF32;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{
        Entity, EntityBase,
        projectile::{HurtingProjectile, ProjectileHit, ThrownItemEntity},
    },
    server::Server,
};

pub const DEFAULT_EXPLOSION_POWER: f32 = 1.0;

pub struct FireballEntity {
    pub thrown: HurtingProjectile,
    pub item_stack: RwLock<ItemStack>,
    pub explosion_power: AtomicF32,
    incendiary: AtomicBool,
}

impl FireballEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner: ProjectileOwner::none(),
            gravity: 0.0,
        };
        Self::from_projectile(thrown)
    }

    /// Keeps the position of `entity`, the shooter places its fireballs itself.
    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity, direction: Vector3<f64>) -> Self {
        let fireball = Self::from_projectile(ThrownItemEntity::owned(entity, shooter, 0.0));
        fireball.thrown.assign_directional_movement(direction);
        fireball
    }

    fn from_projectile(thrown: ThrownItemEntity) -> Self {
        Self {
            thrown: HurtingProjectile::new(thrown, HurtingProjectile::DEFAULT_ACCELERATION_POWER),
            item_stack: RwLock::new(Self::get_default_item()),
            explosion_power: AtomicF32::new(DEFAULT_EXPLOSION_POWER),
            incendiary: AtomicBool::new(false),
        }
    }

    #[must_use]
    pub fn get_default_item() -> ItemStack {
        ItemStack::new(1, &Item::FIRE_CHARGE)
    }

    pub fn get_item(&self) -> ItemStack {
        self.item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn set_item(&self, source: ItemStack) {
        let new_item = if source.item_count == 0 {
            Self::get_default_item()
        } else {
            let mut item = source;
            item.item_count = 1;
            item
        };
        *self
            .item_stack
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = new_item.clone();

        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::fireball::ITEM_STACK,
            ItemStackSerializer::from(new_item),
        );
    }

    pub fn get_explosion_power(&self) -> f32 {
        self.explosion_power.load(Ordering::Relaxed)
    }

    pub fn set_explosion_power(&self, power: f32) {
        self.explosion_power.store(power, Ordering::Relaxed);
    }

    /// Whether the explosion sets fire to its crater (Paper `Fireball.setIsIncendiary`).
    pub fn is_incendiary(&self) -> bool {
        self.incendiary.load(Ordering::Relaxed)
    }

    pub fn set_incendiary(&self, incendiary: bool) {
        self.incendiary.store(incendiary, Ordering::Relaxed);
    }

    /// Sets the acceleration (blocks per tick) and points the fireball along `direction`.
    pub fn set_speed(&self, speed: f64, direction: Vector3<f64>) {
        self.thrown.set_acceleration_power(speed);
        self.thrown.assign_directional_movement(direction);
    }
}

impl EntityBase for FireballEntity {
    fn can_hit(&self) -> bool {
        super::is_redirectable(self.get_entity())
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown.owner)
    }

    fn on_deflection(&self, by_attack: bool) {
        self.thrown.on_deflection(by_attack);
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.thrown.write_nbt(nbt);
        let mut item = NbtCompound::new();
        self.get_item().write_item_stack(&mut item);
        nbt.put_compound("Item", item);
        nbt.put_byte("ExplosionPower", self.get_explosion_power() as i8);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.thrown.read_nbt(nbt);
        self.set_item(
            nbt.get_compound("Item")
                .and_then(ItemStack::read_item_stack)
                .unwrap_or_else(Self::get_default_item),
        );
        self.set_explosion_power(f32::from(nbt.get_byte("ExplosionPower").unwrap_or(1)));
    }

    fn init_data_tracker(&self) {
        let entity = self.get_entity();
        let stack = self
            .item_stack
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        entity.set_synced_data(
            pumpkin_data::tracked_data::fireball::ITEM_STACK,
            ItemStackSerializer::from(stack.clone()),
        );
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        self.thrown.process_tick(
            caller,
            server,
            HurtingProjectile::INERTIA,
            HurtingProjectile::LIQUID_INERTIA,
            true,
            |_| true,
        );
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let world = self.get_entity().world.load();
        if let ProjectileHit::Entity {
            ref entity,
            hit_pos,
            ..
        } = hit
        {
            let owner = self.thrown.owner.get(&world);
            let damage_type = if owner.is_some() {
                DamageType::FIREBALL
            } else {
                DamageType::UNATTRIBUTED_FIREBALL
            };
            entity.damage_with_context(
                entity.as_ref(),
                6.0,
                damage_type,
                Some(hit_pos),
                Some(self),
                owner.as_deref(),
            );
        }

        // Explodes where the fireball is, which is the hit point after the move. 1.8 explodes at
        // the position the fireball had before the step that hit (`legacy_1_8`).
        let center = if world.combat_mode().is_legacy() {
            self.get_entity().last_pos.load()
        } else {
            self.get_entity().pos.load()
        };
        let mut explosion = crate::world::explosion::Explosion::new(
            self.get_explosion_power(),
            center,
            world.get_block_interaction(crate::world::ExplosionInteraction::Mob),
        )
        .with_source(self);
        if self.is_incendiary() {
            explosion = explosion.with_fire();
        }
        world.run_explosion(&explosion);
    }
}
