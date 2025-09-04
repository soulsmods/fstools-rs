use bevy::{asset::LoadContext, prelude::*};
use fstools_formats::msb::{parts, parts::PartData, Msb, MsbError, MsbVersion::EldenRing};
use thiserror::Error;

use crate::{asset_source::fast_path::FastPathAssetLoader, types::flver::FlverAsset};

#[derive(Asset, TypePath, Debug)]
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

impl FastPathAssetLoader for MsbAssetLoader {
    type Asset = MsbAsset;
    type Settings = ();
    type Error = MsbAssetLoaderError;

    async fn load_from_bytes<'a>(
        reader: &'a [u8],
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let msb = Msb::parse(reader, &EldenRing)?;

        let models = msb
            .models()
            .expect("Could not get model set from MSB")
            .map(|m| {
                let mut name = m
                    .expect("Could not get name bytes from model entry")
                    .name
                    .to_string();
                if name.starts_with('m') {
                    let msb_name = load_context.asset_path().to_string();
                    name = format!(
                        "{}_{}",
                        &msb_name[21..33], // Lets fucking pray
                        &name[1..],
                    );
                }

                let model_name = format!("vfs://{}.flver", name);

                load_context.load(model_name)
            })
            .collect::<Vec<Handle<FlverAsset>>>();

        Ok(MsbAsset {
            points: msb
                .points()
                .expect("Could not get point set from MSB")
                .map(|p| {
                    let point = p.as_ref().expect("Could not get point entry from MSB");
                    load_context.labeled_asset_scope(point.name.to_string(), |_| MsbPointAsset {
                        name: point.name.to_string(),
                        position: Vec3::new(
                            point.position[0].get(),
                            point.position[1].get(),
                            point.position[2].get(),
                        ),
                    })
                })
                .collect(),

            parts: msb
                .parts()
                .expect("Could not get parts set from MSB")
                .filter_map(|p| {
                    let part = p.as_ref().expect("Could not get point entry from MSB");

                    if let PartData::EldenRing(parts::elden_ring::PartData::DummyAsset(_)) =
                        part.part
                    {
                        return None;
                    }

                    Some(
                        load_context.labeled_asset_scope(part.name.to_string(), |_| MsbPartAsset {
                            name: part.name.to_string(),
                            transform: MsbAssetLoader::make_msb_transform(
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
                            model: models[part.model_index.get() as usize].clone(),
                        }),
                    )
                })
                .collect(),
        })
    }
}
