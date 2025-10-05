use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
};
use fstools_formats::msb::{parts::PartData, Msb, MsbError};
use thiserror::Error;

use crate::types::flver::FlverAsset;

#[derive(Asset, Reflect, Debug)]
pub struct MsbAsset {
    pub points: Vec<Handle<MsbPointAsset>>,
    pub parts: Vec<Handle<MsbPartAsset>>,
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

#[derive(Clone, TypePath)]
pub struct MsbAssetLoader;

impl MsbAssetLoader {
    // TODO: probably not the right place for this
    // TODO: it seems for models the orientation is inverted on some axis still?
    fn make_msb_transform(
        translation: Vec3,
        rotation: impl Into<Option<Vec3>>,
        _scale: impl Into<Option<Vec3>>,
    ) -> Transform {
        let translation = Vec3::new(translation.x, translation.y, -translation.z);

        // 2. Convert to Radians
        let rot = rotation.into().unwrap_or_default();
        let rx = rot.x.to_radians();
        let ry = rot.y.to_radians();
        let rz = rot.y.to_radians();
        let rotation = Quat::from_euler(EulerRot::YXZ, -ry, -rx, -rz);

        Transform {
            translation,
            rotation,
            scale: Vec3::ONE,
        }
    }
}

impl AssetLoader for MsbAssetLoader {
    type Asset = MsbAsset;
    type Settings = ();
    type Error = MsbAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data = vec![];
        reader.read_to_end(&mut data).await?;

        let msb = Msb::parse(&data)?;

        let models = msb
            .models()
            .expect("Could not get model set from MSB")
            .map(|m| {
                let mut name = m
                    .expect("Could not get name bytes from model entry")
                    .name
                    .to_string();
                if name.starts_with('m') {
                    let msb_name = load_context.path().to_string();
                    name = format!(
                        "{}_{}",
                        &msb_name[21..33], // Lets fucking pray
                        &name[1..],
                    );
                }

                let model_name = format!("vfs://{name}.flver");

                load_context.load(model_name)
            })
            .collect::<Vec<Handle<FlverAsset>>>();

        Ok(MsbAsset {
            points: msb
                .points()
                .expect("Could not get point set from MSB")
                .flat_map(|p| {
                    let point = p.as_ref().expect("Could not get point entry from MSB");
                    load_context.labeled_asset_scope(point.name.to_string(), |_| {
                        Ok::<_, MsbAssetLoaderError>(MsbPointAsset {
                            name: point.name.to_string(),
                            position: Vec3::new(
                                point.position[0].get(),
                                point.position[1].get(),
                                point.position[2].get(),
                            ),
                        })
                    })
                })
                .collect(),

            parts: msb
                .parts()
                .expect("Could not get parts set from MSB")
                .filter_map(|p| {
                    let part = p.as_ref().expect("Could not get point entry from MSB");

                    if let PartData::DummyAsset(_) = part.part {
                        return None;
                    }

                    Some(
                        load_context.labeled_asset_scope(part.name.to_string(), |_| {
                            Ok::<_, MsbAssetLoaderError>(MsbPartAsset {
                                name: part.name.to_string(),
                                transform: MsbAssetLoader::make_msb_transform(
                                    Vec3::new(
                                        part.position[0].get(),
                                        part.position[1].get(),
                                        part.position[2].get(),
                                    ),
                                    Vec3::new(
                                        part.rotation[0].get(),
                                        part.rotation[1].get(),
                                        part.rotation[2].get(),
                                    ),
                                    Vec3::new(
                                        part.scale[0].get(),
                                        part.scale[1].get(),
                                        part.scale[2].get(),
                                    ),
                                ),
                                model: models[part.model_index.get() as usize].clone(),
                            })
                        }),
                    )
                })
                .flatten()
                .collect(),
        })
    }
}
