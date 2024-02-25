// use byteorder::{ByteOrder, BE, LE};
// use zerocopy::{FromBytes, FromZeroes, U32};
//
// use crate::io_ext::zerocopy::Padding;

// pub trait FlverMesh {
//     fn version(&self) -> u32;
// }
//
// pub type FlverMeshLE = FlverMeshData<LE>;
// pub type FlverMeshBE = FlverMeshData<BE>;
//
// #[derive(FromZeroes, FromBytes)]
// #[repr(C)]
// pub struct FlverMeshData<O: ByteOrder> {
//     dynamic: u8,
//     _padding1: Padding<3>,
//     material_index: U32<O>,
//     _padding2: Padding<8>,
//     default_bone_index: U32<O>,
//     bone_count: U32<O>,
//     bounding_box_offset: U32<O>,
//     bone_offset: U32<O>,
//     face_set_count: U32<O>,
//     face_set_offset: U32<O>,
//     vertex_buffer_count: U32<O>,
//     vertex_buffer_offset: U32<O>,
// }
