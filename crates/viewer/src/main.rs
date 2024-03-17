use std::{
    io::{self, Read},
    path::PathBuf,
};

use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor, WireframePlugin},
    prelude::*,
    transform::TransformSystem,
};
use bevy_basic_camera::{CameraController, CameraControllerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use clap::Parser;
use formats::msb::{MsbAsset, MsbPartAsset, MsbPointAsset};
use fstools_formats::{dcx::DcxHeader, msb::Msb};
use fstools_vfs::{FileKeyProvider, Vfs};
use vfs::VfsAssetRepositoryPlugin;

use crate::{flver::asset::FlverAsset, formats::FormatsPlugins};

pub mod flver;
mod formats;
mod vfs;

fn main() {
    let args = Args::parse();
    let er_path = args.erpath.expect("no path to Elden Ring game provided");

    let keys = FileKeyProvider::new("keys");
    let archives = [
        er_path.join("Data0"),
        er_path.join("Data1"),
        er_path.join("Data2"),
        er_path.join("Data3"),
        er_path.join("sd/sd"),
    ];

    let mut vfs = Vfs::create(archives.clone(), &keys).expect("unable to create vfs");
    // TODO: get rid of this
    let path = format!("/map/mapstudio/{}.msb.dcx", args.msb);
    let mut msb = vfs.open(path).unwrap();
    let mut dcx_file = vec![];
    msb.read_to_end(&mut dcx_file).unwrap();

    let (_, mut decoder) = DcxHeader::read(dcx_file.as_slice()).unwrap();

    let mut decompressed = Vec::with_capacity(decoder.hint_size());
    decoder.read_to_end(&mut decompressed).unwrap();

    let msb = Msb::parse(&decompressed).unwrap();

    // Load all dependencies for this MSB
    // TODO: this can be much more optimized and cleaner lmao
    msb.models()
        .unwrap()
        .map(|m| {
            let model = m.unwrap();
            let model_name = model.name.to_string_lossy();

            if model_name.starts_with("AEG") {
                let lower = model_name.to_lowercase();
                vec![format!("/asset/aeg/{}/{}.geombnd.dcx", &lower[..6], lower)]
            } else if model_name.starts_with("m") {
                let lower = model_name.to_lowercase();

                vec![format!(
                    "/map/{}/{}/{}_{}.mapbnd.dcx",
                    &args.msb[..3],
                    &args.msb,
                    &args.msb,
                    &lower[1..]
                )]
            } else {
                println!("Couldn't match asset for {}", model_name);
                vec![]
            }
        })
        .flatten()
        .for_each(|dep| match vfs.mount(&dep) {
            Ok(_) => println!("Loaded dependency {}", dep),
            Err(_) => println!("Could not load dependency {}", dep),
        });

    App::new()
        .add_plugins((VfsAssetRepositoryPlugin::new(vfs), DefaultPlugins))
        .add_plugins(FormatsPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(CameraControllerPlugin)
        .add_plugins(WireframePlugin)
        .init_resource::<AssetCollection>()
        .init_resource::<PartsModelLoading>()
        .add_systems(Startup, setup)
        .add_systems(Update, (spawn_parts, spawn_parts_models))
        .add_systems(Update, (spawn_points, render_points))
        .add_systems(
            PostUpdate,
            update_point_labels.after(TransformSystem::TransformPropagate),
        )
        .run();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    erpath: Option<PathBuf>,

    #[arg(long)]
    msb: String,
}

#[derive(Debug)]
pub enum AssetLoadError {
    Io(io::Error),
    NotFound,
}

#[derive(Resource, Default)]
pub struct AssetCollection {
    msb: Vec<Handle<MsbAsset>>,
}

fn setup(
    mut commands: Commands,
    mut assets: ResMut<AssetCollection>,
    asset_server: Res<AssetServer>,
) {
    let args = Args::parse();

    let path = format!("vfs:///map/mapstudio/{}.msb.dcx", args.msb);
    let map: Handle<MsbAsset> = asset_server.load(path);
    assets.msb.push(map);

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            color: Color::WHITE,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.5, 0.4, 0.0)),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 6., 12.0)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            ..default()
        },
        CameraController {
            walk_speed: 10.0,
            run_speed: 50.0,
            ..default()
        }
        .print_controls(),
    ));
}

#[derive(Default, Resource)]
struct PartsModelLoading(Vec<PartsModelInstance>);

struct PartsModelInstance {
    model: Handle<FlverAsset>,
    msb_transform: Transform,
}

fn spawn_parts(
    mut events: EventReader<AssetEvent<MsbPartAsset>>,
    mut loading: ResMut<PartsModelLoading>,
    parts: Res<Assets<MsbPartAsset>>,
) {
    for ev in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            let part = parts.get(*id).expect("part wasn't loaded");

            loading.0.push(PartsModelInstance {
                model: part.model.clone(),
                msb_transform: part.transform,
            });
        }
    }
}

fn spawn_parts_models(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<FlverAsset>>,
    mut loading: ResMut<PartsModelLoading>,
    flvers: Res<Assets<FlverAsset>>,
) {
    for ev in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            let instances_for_model = loading.0.iter().filter(|i| i.model.id() == *id);
            for instance in instances_for_model {
                let flver = flvers.get(*id).expect("flver wasn't loaded");

                for mesh in flver.meshes() {
                    commands.spawn((
                        PbrBundle {
                            mesh: mesh.clone(),
                            transform: instance.msb_transform,
                            ..PbrBundle::default()
                        },
                        Wireframe,
                        WireframeColor {
                            color: Color::WHITE.into(),
                        },
                    ));
                }
            }

            let _ = loading.0.retain(|i| i.model.id() != *id);
        }
    }
}

fn spawn_points(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<MsbPointAsset>>,
    points: Res<Assets<MsbPointAsset>>,
    asset_server: Res<AssetServer>,
) {
    for ev in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = ev {
            let point = points.get(*id).expect("point wasn't loaded");

            commands.spawn((
                LabelPosition { position: point.position },
                TextBundle::from_section(
                    format!("{}", point.name.to_string()),
                    TextStyle {
                        font: asset_server.load("fonts/NotoSansJP-Medium.ttf"),
                        font_size: 16.0,
                        color: Color::BLACK,
                        ..default()
                    },
                )
                .with_text_justify(JustifyText::Center),
            ));
        }
    }
}

#[derive(Component)]
struct LabelPosition {
    position: Vec3,
}

fn render_points(mut query: Query<&LabelPosition>, mut gizmos: Gizmos) {
    for point_data in query.iter_mut() {
        gizmos.sphere(point_data.position, Quat::IDENTITY, 0.1, Color::RED);
    }
}

fn update_point_labels(
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(&LabelPosition, &mut Style, &mut Visibility)>,
) {
    let (camera, camera_global_transform) = cameras.single();

    for (label, mut style, mut visibility) in query.iter_mut() {
        let distance_to_camera = (label.position - camera_global_transform.translation()).length();

        if distance_to_camera < 50.0 {
            if let Some(v) = camera.world_to_viewport(camera_global_transform, label.position) {
                style.top = Val::Px(v.y);
                style.left = Val::Px(v.x);
                *visibility = Visibility::Visible;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
