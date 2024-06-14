const X_BITS: u8 = 13;
const Y_BITS: u8 = 13;
const Z_BITS: u8 = 6;

const X_BIAS: i32 = 1 << (X_BITS - 1);
const Y_BIAS: i32 = 1 << (Y_BITS - 1);
const Z_BIAS: i32 = 1 << (Z_BITS - 1);

const X_SHIFT: u8 = 0;
const Y_SHIFT: u8 = X_BITS;
const Z_SHIFT: u8 = X_BITS + Y_BITS;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
pub struct VoxelChunkIndex(pub u32);

impl VoxelChunkIndex {
    pub fn from_coords(x: i32, y: i32, z: i32) -> VoxelChunkIndex {
        let x: u32 = ((x + X_BIAS) as u32) << X_SHIFT;
        let y: u32 = ((y + Y_BIAS) as u32) << Y_SHIFT;
        let z: u32 = ((z + Z_BIAS) as u32) << Z_SHIFT;
        Self(z | y | x)
    }
}
