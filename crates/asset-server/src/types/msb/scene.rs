use bevy::{
    core::Name,
    prelude::{Bundle, Handle, SpatialBundle, Transform},
};

use crate::types::flver::FlverAsset;

#[derive(Bundle)]
pub struct PointBundle {
    name: Name,
    position: Transform,
}

impl PointBundle {
    pub fn new<S: ToString>(name: S, transform: Transform) -> Self {
        Self {
            name: Name::new(name.to_string()),
            position: transform,
        }
    }
}

#[derive(Bundle)]
pub struct PartBundle {
    flver: Handle<FlverAsset>,
    name: Name,
    spatial: SpatialBundle,
}

impl PartBundle {
    pub fn new<S: ToString>(name: S, flver: Handle<FlverAsset>, transform: Transform) -> Self {
        Self {
            flver,
            name: Name::new(name.to_string()),
            spatial: SpatialBundle::from(transform),
        }
    }
}
