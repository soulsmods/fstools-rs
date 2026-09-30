use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, U16, U32};

use crate::{flver::header::FlverHeaderPart, io_ext::zerocopy::Padding};

pub enum FaceSetIndices<'a, O> {
    None,
    U8(&'a [u8]),
    U16(&'a [U16<O>]),
    U32(&'a [U32<O>]),
}

#[derive(FromZeroes, FromBytes, Debug)]
#[repr(C)]
#[allow(unused)]
pub struct FaceSet<O: ByteOrder> {
    flags: U32<O>,
    triangle_strip: u8,
    cull_back_faces: u8,
    unk06: U16<O>,
    pub(crate) index_count: U32<O>,
    pub(crate) index_offset: U32<O>,
    unk: U32<O>,
    padding0: Padding<4>,
    pub(crate) index_size: U32<O>,
    padding1: U32<O>,
}

impl<O: ByteOrder> FaceSet<O> {
    pub fn is_lod0(&self) -> bool {
        self.flags.get() == 0
    }

    /// Whether the indices in this face set form a triangle strip rather
    /// than a triangle list.  When `true`, the consumer must triangulate
    /// the index buffer before rendering (see [`triangulate_strip`]).
    pub fn is_triangle_strip(&self) -> bool {
        self.triangle_strip != 0
    }

    /// Whether back-face culling should be enabled for this face set.
    pub fn cull_back_faces(&self) -> bool {
        self.cull_back_faces != 0
    }
}

impl<O: ByteOrder> FlverHeaderPart for FaceSet<O> {}

pub fn triangulate_strip(indices: &[u32], primitive_restart: u32) -> Vec<u32> {
    if indices.len() < 3 {
        return Vec::new();
    }

    let mut triangles = Vec::with_capacity(indices.len()); // rough upper bound
    let mut flip = false;

    for i in 0..indices.len() - 2 {
        let v0 = indices[i];
        let v1 = indices[i + 1];
        let v2 = indices[i + 2];

        // Primitive restart
        if v0 == primitive_restart || v1 == primitive_restart || v2 == primitive_restart {
            flip = false;
            continue;
        }

        // Skip degenerate triangles (two or more repeated indices).
        if v0 != v1 && v1 != v2 && v2 != v0 {
            if flip {
                triangles.push(v2);
                triangles.push(v1);
                triangles.push(v0);
            } else {
                triangles.push(v0);
                triangles.push(v1);
                triangles.push(v2);
            }
        }

        flip = !flip;
    }

    triangles
}
