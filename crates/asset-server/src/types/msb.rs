mod scene;

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::BoxedFuture,
};
use fstools_formats::msb::{Msb, MsbError};
use thiserror::Error;

use crate::types::{
    flver::FlverAsset,
    msb::scene::{PartBundle, PointBundle},
};

pub struct MsbPlugin;

impl Plugin for MsbPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MsbAsset>()
            .register_type::<MsbAsset>()
            .register_asset_loader(MsbAssetLoader);
    }
}

#[derive(Asset, Debug, Reflect)]
pub struct MsbAsset {
    scene: Handle<Scene>,
}

#[derive(Asset, Clone, TypePath, Debug)]
pub struct MsbPointAsset {
    pub name: String,
    pub position: Vec3,
}

#[derive(Asset, Clone, TypePath, Debug)]
pub struct MsbPartAsset {
    pub name: String,
    pub transform: Transform,
    pub model: Handle<FlverAsset>,
}

#[derive(Debug, Error)]
pub enum MsbAssetLoaderError {
    #[error("Could not read msb: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse msb: {0}")]
    Parser(#[from] MsbError),
}

#[derive(Default)]
pub struct MsbAssetLoader;

impl MsbAssetLoader {
    // TODO: probably not the right place for this
    // TODO: it seems for models the orientation is inverted on some axis still?
    fn make_msb_transform(
        translation: Vec3,
        rotation: Option<Vec3>,
        scale: Option<Vec3>,
    ) -> Transform {
        let translation = Mat4::from_translation(translation);
        let scale = Mat4::from_scale(scale.unwrap_or(Vec3::new(1.0, 1.0, 1.0)));

        let rotation = rotation.unwrap_or_default();
        let rotation = Mat4::from_euler(
            EulerRot::ZYX,
            rotation[0].to_radians(),
            rotation[1].to_radians(),
            rotation[2].to_radians(),
        );

        // TODO: can be const?
        let scene_transform = {
            let mut identity = Mat4::IDENTITY;

            // Invert Z
            identity.z_axis.z = -1.0;

            identity
        };

        Transform::from_matrix(scene_transform * translation * rotation * scale)
    }
}

impl AssetLoader for MsbAssetLoader {
    type Asset = MsbAsset;
    type Settings = ();
    type Error = MsbAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).await?;

            // Account for DCX compression
            let msb = Msb::parse(&buffer)?;

            let mut world = World::new();
            let mut children = vec![];

            for point in msb.points()? {
                let Ok(point) = point else { continue };
                let point_entity = world.spawn(PointBundle::new(
                    point.name.to_string_lossy(),
                    Transform::from_xyz(
                        point.position[0].get(),
                        point.position[1].get(),
                        point.position[2].get(),
                    ),
                ));

                children.push(point_entity.id());
            }

            for part in msb.parts()? {
                let Ok(part) = part else { continue };
                let part_entity = world.spawn(PartBundle::new(
                    part.name.to_string_lossy(),
                    Handle::default(),
                    Self::make_msb_transform(
                        Vec3::new(
                            part.position[0].get(),
                            part.position[1].get(),
                            part.position[2].get(),
                        ),
                        Some(Vec3::new(
                            part.rotation[0].get(),
                            part.rotation[1].get(),
                            part.rotation[2].get(),
                        )),
                        Some(Vec3::new(
                            part.scale[0].get(),
                            part.scale[1].get(),
                            part.scale[2].get(),
                        )),
                    ),
                ));

                children.push(part_entity.id());
            }

            let mut root = world.spawn(Name::from(
                load_context
                    .path()
                    .file_name()
                    .expect("requested asset must have a filename")
                    .to_string_lossy()
                    .to_string(),
            ));

            for child in children.drain(..) {
                root.add_child(child);
            }

            let scene = load_context
                .labeled_asset_scope("MainScene".to_string(), move |_| Scene::new(world));

            Ok(MsbAsset { scene })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["msb", "msb.dcx"]
    }
}
