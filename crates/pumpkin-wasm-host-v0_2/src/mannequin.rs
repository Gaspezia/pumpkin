use std::sync::Arc;

use pumpkin_core::entity::decoration::mannequin::MannequinEntity as InternalMannequinEntity;
use pumpkin_data::data_component_impl::{ProfileImpl, ProfileProperty};
use pumpkin_util::text::TextComponent as PumpkinText;
use pumpkin_wasm_host_common::state::PluginHostState;
use wasmtime::component::Resource;

use crate::{
    pumpkin::plugin::{
        common::Hand,
        mannequin::{
            Host, HostMannequinEntity, MannequinEntity, MannequinProfile,
            ProfileProperty as WitProfileProperty,
        },
        text::TextComponent,
        uuid::Uuid,
        world::Entity,
    },
    uuid::UuidExt,
};

impl Host for PluginHostState {}

fn uuid_to_words(id: &Uuid) -> [i32; 4] {
    let u = Uuid::from_wit(id).as_u128();
    [
        (u >> 96) as i32,
        (u >> 64) as i32,
        (u >> 32) as i32,
        u as i32,
    ]
}

fn words_to_uuid(id: [i32; 4]) -> Uuid {
    let u = id
        .iter()
        .fold(0u128, |acc, w| (acc << 32) | u128::from(*w as u32));
    Uuid::to_wit(&uuid::Uuid::from_u128(u))
}

pub(crate) fn profile_from_wit(profile: MannequinProfile) -> ProfileImpl {
    ProfileImpl {
        name: profile.name,
        id: profile.id.as_ref().map(uuid_to_words),
        properties: profile
            .properties
            .into_iter()
            .map(|p| ProfileProperty {
                name: p.name,
                value: p.value,
                signature: p.signature,
            })
            .collect(),
        ..Default::default()
    }
}

pub(crate) fn profile_to_wit(profile: ProfileImpl) -> MannequinProfile {
    MannequinProfile {
        name: profile.name,
        id: profile.id.map(words_to_uuid),
        properties: profile
            .properties
            .into_iter()
            .map(|p| WitProfileProperty {
                name: p.name,
                value: p.value,
                signature: p.signature,
            })
            .collect(),
    }
}

impl HostMannequinEntity for PluginHostState {
    fn from_entity(
        &mut self,
        entity: /* borrow */ Resource<Entity>,
    ) -> wasmtime::Result<Option<Resource<MannequinEntity>>> {
        if let Ok(mannequin) = Arc::downcast::<InternalMannequinEntity>(self.get(&entity)?.clone())
        {
            Ok(Some(self.add(mannequin)?))
        } else {
            Ok(None)
        }
    }

    fn get_entity(
        &mut self,
        mannequin: Resource<MannequinEntity>,
    ) -> wasmtime::Result<Resource<Entity>> {
        self.add(self.get(&mannequin)?.clone() as _)
    }

    fn get_profile(
        &mut self,
        mannequin: Resource<MannequinEntity>,
    ) -> wasmtime::Result<MannequinProfile> {
        Ok(profile_to_wit(self.get(&mannequin)?.get_profile()))
    }

    fn set_profile(
        &mut self,
        mannequin: Resource<MannequinEntity>,
        profile: MannequinProfile,
    ) -> wasmtime::Result<()> {
        self.get(&mannequin)?.set_profile(profile_from_wit(profile));
        Ok(())
    }

    fn set_layers(
        &mut self,
        mannequin: Resource<MannequinEntity>,
        mask: u8,
    ) -> wasmtime::Result<()> {
        self.get(&mannequin)?.set_layers_mask(mask);
        Ok(())
    }

    fn set_main_hand(
        &mut self,
        mannequin: Resource<MannequinEntity>,
        hand: Hand,
    ) -> wasmtime::Result<()> {
        self.get(&mannequin)?.set_main_hand_id(match hand {
            Hand::Left => 0,
            Hand::Right => 1,
        });
        Ok(())
    }

    fn set_immovable(
        &mut self,
        mannequin: Resource<MannequinEntity>,
        immovable: bool,
    ) -> wasmtime::Result<()> {
        self.get(&mannequin)?.set_immovable(immovable);
        Ok(())
    }

    fn set_description(
        &mut self,
        mannequin: Resource<MannequinEntity>,
        text: Option<Resource<TextComponent>>,
    ) -> wasmtime::Result<()> {
        let mannequin = self.get(&mannequin)?.clone();
        match text {
            Some(text) => {
                let text = self.take(text)?;
                mannequin.set_description_state(text, false);
            }
            None => {
                mannequin.set_description_state(PumpkinText::text(""), true);
            }
        }
        Ok(())
    }

    fn drop(&mut self, rep: Resource<MannequinEntity>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arc_swap::ArcSwap;
    use pumpkin_config::world::LevelConfig;
    use pumpkin_core::{
        block::registry::default_registry,
        entity::{
            Entity as CoreEntity, EntityBase,
            decoration::display::TextDisplayEntity as InternalText,
        },
        world::World,
    };
    use pumpkin_data::{dimension::Dimension, entity::EntityType};
    use pumpkin_util::{math::vector3::Vector3, world_seed::Seed};
    use pumpkin_world::{level::Level, world_info::LevelData};
    use std::sync::Weak;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn world(folder: &tempfile::TempDir) -> Arc<World> {
        let dimension = Dimension::OVERWORLD.clone();
        Arc::new(World::load(
            Level::from_root_folder(
                &LevelConfig::default(),
                folder.path().to_path_buf(),
                0,
                dimension.clone(),
            ),
            Arc::new(ArcSwap::from_pointee(LevelData::default(Seed(0)))),
            dimension,
            default_registry(),
            Weak::new(),
        ))
    }

    #[test]
    fn uuid_words_roundtrip() {
        let id = Uuid::to_wit(&uuid::Uuid::from_u128(
            0x8000_0001_0000_0002_ffff_ffff_0000_0004,
        ));
        let back = words_to_uuid(uuid_to_words(&id));
        assert_eq!((back.high, back.low), (id.high, id.low));
        assert_eq!(uuid_to_words(&id), [i32::MIN + 1, 2, -1, 4]);
    }

    #[tokio::test]
    async fn from_entity_and_set_profile() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let pos = Vector3::new(0.0, 64.0, 0.0);
        let mannequin = InternalMannequinEntity::new(CoreEntity::new(
            world.clone(),
            pos,
            &EntityType::MANNEQUIN,
        ));
        let other: Arc<dyn EntityBase> = InternalText::new(CoreEntity::new(
            world.clone(),
            pos,
            &EntityType::TEXT_DISPLAY,
        ));

        let mut host = PluginHostState::new();
        let other_res: Resource<Entity> = host.add(other)?;
        assert!(
            HostMannequinEntity::from_entity(&mut host, Resource::new_borrow(other_res.rep()))?
                .is_none()
        );

        let as_base: Arc<dyn EntityBase> = mannequin.clone();
        let res: Resource<Entity> = host.add(as_base)?;
        let handle = HostMannequinEntity::from_entity(&mut host, Resource::new_borrow(res.rep()))?
            .expect("mannequin");

        let id = Uuid::to_wit(&uuid::Uuid::from_u128(
            0x0000_0001_0000_0002_0000_0003_0000_0004,
        ));
        let profile = MannequinProfile {
            name: Some("Notch".into()),
            id: Some(id),
            properties: vec![WitProfileProperty {
                name: "textures".into(),
                value: "AAAA".into(),
                signature: Some("sig".into()),
            }],
        };
        HostMannequinEntity::set_profile(&mut host, Resource::new_borrow(handle.rep()), profile)?;
        let stored = mannequin.get_profile();
        assert_eq!(stored.name.as_deref(), Some("Notch"));
        assert_eq!(stored.id, Some([1, 2, 3, 4]));
        assert_eq!(stored.properties[0].signature.as_deref(), Some("sig"));
        let back = HostMannequinEntity::get_profile(&mut host, Resource::new_borrow(handle.rep()))?;
        assert_eq!(back.id.map(|u| (u.high, u.low)), Some((id.high, id.low)));

        HostMannequinEntity::set_main_hand(
            &mut host,
            Resource::new_borrow(handle.rep()),
            Hand::Left,
        )?;
        assert_eq!(mannequin.get_main_hand_id(), 0);
        HostMannequinEntity::set_description(&mut host, Resource::new_borrow(handle.rep()), None)?;
        assert!(mannequin.get_description().is_none());

        world.level.shutdown().await;
        Ok(())
    }
}
