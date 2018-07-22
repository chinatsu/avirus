pub mod frame;

pub const BUFFER_SIZE: u64 = 16777216; // 2 ^ 24
pub const AVIIF_LIST: u32 = 0x00000001;
pub const AVIIF_KEYFRAME: u32 = 0x00000010;
pub const AVIIF_NO_TIME: u32 = 0x00000100;
pub const SAFE_FRAMES_COUNT: u64 = 150000;
