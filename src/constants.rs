/**
 * Header defined as 24 bytes total.
 * 8 bytes for filename length
 * 8 bytes for file size
 * 8 bytes reserved for later
 */
pub const PACKAGE_HEADER_SIZE: usize = 24;
pub const CHUNK_SIZE: usize = 64 * 1024; // 64kb
