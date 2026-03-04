use std::collections::HashMap;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext, RenderAssetUsages},
    image::ImageLoaderSettings,
    math::Affine2,
    pbr::StandardMaterial,
    reflect::TypePath,
    utils::default,
};
use fstools_formats::matbin::Matbin;
use serde::{Deserialize, Serialize};
use tracing::info;

pub struct MatbinSampler {}

#[derive(TypePath)]
pub struct MatbinAssetLoader;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MatbinSettings {
    pub textures: HashMap<String, String>,
}

impl AssetLoader for MatbinAssetLoader {
    type Asset = StandardMaterial;
    type Settings = MatbinSettings;
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data).await?;

        let matbin = Matbin::parse(&data).ok_or(std::io::Error::other("invalid data"))?;
        let textures: HashMap<String, String> = matbin
            .samplers()
            .filter_map(|sampler| {
                let sampler = sampler.ok()?;
                let name = sampler.name.to_utf8();
                if sampler.path.is_empty() {
                    return None;
                }

                let fixed_name = name.rsplit_terminator('_').next().unwrap_or(&name);
                let image_path = sampler.image_path().to_path_buf().with_extension("dds");
                let image_name = image_path.file_name().expect("infallible");

                Some((fixed_name.to_owned(), image_name.to_owned()))
            })
            .collect();

        info!(?textures, ?settings.textures);
        let image_loader = load_context
            .loader()
            .deferred()
            .with_static_type()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                settings.asset_usage = RenderAssetUsages::all();
                settings.format = bevy::image::ImageFormatSetting::FromExtension;
            });

        let material = StandardMaterial {
            base_color_texture: settings
                .textures
                .get("AlbedoMap")
                .or_else(|| textures.get("AlbedoMap"))
                .map(|path| image_loader.load(format!("vfs://texture/{path}"))),
            uv_transform: Affine2::from_scale_angle_translation(
                bevy::math::Vec2::new(1.0, -1.0),
                0.0,
                bevy::math::Vec2::new(0.0, 1.0),
            ),
            ..default()
        };

        Ok(material)
    }

    fn extensions(&self) -> &[&str] {
        &["matbin"]
    }
}
