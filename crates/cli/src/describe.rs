use std::{error::Error, io::Cursor};

use fstools_dvdbnd::DvdBnd;
use fstools_formats::{
    bnd4::BND4,
    entryfilelist::EntryFileList,
    flver::reader::FLVER,
    msb,
    msb::{
        event,
        event::EventType,
        parts,
        parts::PartType,
        point,
        point::PointType,
        MsbError, MsbParam, MsbVersion,
        MsbVersion::{EldenRing, Nightreign},
    },
};

use crate::GameType;

pub fn describe_bnd(
    dvd_bnd: &DvdBnd,
    name: &str,
    nested_bnd_names: &[String],
) -> Result<(), Box<dyn Error>> {
    let (dcx, data) = dvd_bnd.read_file(nested_bnd_names, name)?;
    let bnd = BND4::from_reader(&mut Cursor::new(data))?;

    println!("Compression type: {}", dcx);
    println!("Files: {}", bnd.files.len());

    for idx in 0..bnd.files.len() {
        println!("File[{idx}] {}", bnd.files[idx].path);
    }

    Ok(())
}

pub fn describe_entryfilelist(dvd_bnd: &DvdBnd, name: &str) -> Result<(), Box<dyn Error>> {
    let reader = dvd_bnd.open(name).expect("Could not open dvdbnd entry");
    let container = EntryFileList::from_bytes(reader.data())?;

    println!("Container: {container:#?}");
    let mut unk1s = container.content_iter()?;
    for unk1 in unk1s.by_ref() {
        println!(" - Unk1: {:?}", unk1?);
    }

    let mut unk2s = unk1s.next_section()?;
    for unk2 in unk2s.by_ref() {
        println!(" - Unk2: {:?}", unk2?);
    }

    let mut unkstrings = unk2s.next_section()?;
    for string in unkstrings.by_ref() {
        println!(" - {:?}", string?);
    }

    Ok(())
}

pub fn describe_flver(
    dvd_bnd: &DvdBnd,
    name: &str,
    nested_bnd_names: &[String],
) -> Result<(), Box<dyn Error>> {
    let (dcx, data) = dvd_bnd.read_file(nested_bnd_names, name)?;
    let flver = FLVER::from_reader(&mut Cursor::new(data))?;

    println!("Compression type: {}", dcx);
    println!("Version: 0x{:X}", flver.version);
    println!("Bounding Box Min: {}", flver.bounding_box_min);
    println!("Bounding Box Max: {}", flver.bounding_box_max);
    println!("Faces: {}", flver.face_count);
    println!("Index Buffers: {}", flver.face_sets.len());
    println!("Vertex Buffers: {}", flver.vertex_buffers.len());
    println!("Bones: {}", flver.bones.len());
    println!("Dummies: {}", flver.dummies.len());

    println!("Materials: {}", flver.materials.len());
    for idx in 0..flver.materials.len() {
        println!("Material[{idx}] {}", flver.materials[idx].mtd);
    }

    println!("Meshes: {}", flver.meshes.len());
    for idx in 0..flver.meshes.len() {
        print!("Mesh[{idx}]");
        print!(" bone: {},", flver.meshes[idx].default_bone_index);
        print!(" material: {},", flver.meshes[idx].material_index);
        print!(" dynamic: {},", flver.meshes[idx].dynamic);
        print!(
            " Index Buffers: {:?},",
            flver.meshes[idx].face_set_indices.as_slice()
        );
        println!(
            " Vertex Buffers: {:?}",
            flver.meshes[idx].vertex_buffer_indices.as_slice()
        );
    }

    Ok(())
}

pub fn describe_matbin(
    dvd_bnd: &DvdBnd,
    name: &str,
    nested_bnd_names: &[String],
) -> Result<(), Box<dyn Error>> {
    let (dcx, data) = dvd_bnd.read_file(nested_bnd_names, name)?;
    let matbin = fstools_formats::matbin::Matbin::parse(&data)
        .expect("Could not parse data as matbin");

    println!("Compression type: {}", dcx);
    println!("Shader: {}", matbin.shader_path().expect("No shader path"));
    println!("Source: {}", matbin.source_path().expect("No source path"));
    let mut params = matbin.parameters();
    let param_count: usize = matbin.parameters().count();
    println!("Parameters: {}", param_count);
    for idx in 0..param_count {
        if let Some(Ok(param)) = params.next() {
            println!("Parameter[{idx}] {0} = {1:?}", param.name, param.value);
        }
    }
    let mut samplers = matbin.samplers();
    let sampler_count: usize = matbin.samplers().count();
    println!("Samplers: {}", sampler_count);
    for idx in 0..sampler_count {
        if let Some(Ok(sampler)) = samplers.next() {
            println!("Sampler[{idx}] {0}: {1}", sampler.name, sampler.path);
        }
    }

    Ok(())
}

pub fn describe_msb(
    dvd_bnd: &DvdBnd,
    name: &str,
    nested_bnd_names: &[String],
    game_type: &GameType,
) -> Result<(), Box<dyn Error>> {
    let (dcx, data) = dvd_bnd.read_file(nested_bnd_names, name)?;
    let version: MsbVersion = match game_type {
        GameType::ErPc => EldenRing,
        GameType::NrPc => Nightreign,
    };
    let msb = msb::Msb::parse(&data, &version).expect("Could not parse data as msb");

    println!("Compression type: {}", dcx);

    if let Ok(models) = msb.models() {
        let models_vec = Vec::from_iter(models);
        println!("Models: {}", models_vec.len());
        for idx in 0..models_vec.len() {
            if let Some(Ok(model)) = models_vec.get(idx) {
                println!("      Model[{idx}] {}", model.name());
            }
        }
    }

    match version {
        EldenRing => {
            if let Ok(events) = msb.events() {
                println!("Events: {}", events.count());
                for ty in event::elden_ring::EventType::variants() {
                    print_msb_param_group(msb.events(), EventType::EldenRing(ty.0), ty.1);
                }
            }

            if let Ok(points) = msb.points() {
                println!("Points: {}", points.count());
                for ty in point::elden_ring::PointType::variants() {
                    print_msb_param_group(msb.points(), PointType::EldenRing(ty.0), ty.1);
                }
            }

            if let Ok(parts) = msb.parts() {
                println!("Parts: {}", parts.count());
                for ty in parts::elden_ring::PartType::variants() {
                    print_msb_param_group(msb.parts(), PartType::EldenRing(ty.0), ty.1);
                }
            }
        }
        Nightreign => {
            if let Ok(events) = msb.events() {
                println!("Events: {}", events.count());
                for ty in event::nightreign::EventType::variants() {
                    print_msb_param_group(msb.events(), EventType::Nightreign(ty.0), ty.1);
                }
            }

            if let Ok(points) = msb.points() {
                println!("Points: {}", points.count());
                for ty in point::nightreign::PointType::variants() {
                    print_msb_param_group(msb.points(), PointType::Nightreign(ty.0), ty.1);
                }
            }

            if let Ok(parts) = msb.parts() {
                println!("Parts: {}", parts.count());
                for ty in parts::nightreign::PartType::variants() {
                    print_msb_param_group(msb.parts(), PartType::Nightreign(ty.0), ty.1);
                }
            }
        }
    }

    if let Ok(routes) = msb.routes() {
        let route_vec = Vec::from_iter(routes);
        println!("Routes: {}", route_vec.len());
        for idx in 0..route_vec.len() {
            if let Some(Ok(route)) = route_vec.get(idx) {
                println!("      Route[{idx}] {}", route.name());
            }
        }
    }

    Ok(())
}

fn print_msb_param_group<'a, P, T>(
    params: Result<impl Iterator<Item = Result<P, MsbError>>, MsbError>,
    group_type: T,
    group_name: &str,
) where
    P: MsbParam<'a, P, T>,
{
    let param_group = P::of_type(params, group_type);
    if !param_group.is_empty() {
        println!("  {0}: {1}", group_name, param_group.len());
    }
    for param in param_group {
        println!(
            "      {0}[{1}] {2}",
            group_name,
            param.type_index(),
            param.name()
        );
    }
}
