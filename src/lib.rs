extern crate byteorder;

use std::collections::HashMap;
use std::collections::HashSet;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

const CHUNK_SIZE: usize = 16;
const DATA_SEGMENT_SIZE: usize = 256;

/// 256 bytes of data, to be used for any purpose
#[derive(Copy, Clone)]
struct DataSegment {
    data: [u8; DATA_SEGMENT_SIZE],
}
/// The location of a Chunk in relation to the world
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
struct ChunkLocation {
    x: u32,
    y: u32,
    z: u32,
}
/// The location of a single voxel in relation to the world
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
struct GlobalLocation {
    x: u32,
    y: u32,
    z: u32,
}
/// The location of a single voxel in relation to its chunk
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
struct VoxelLocation {
    x: u32,
    y: u32,
    z: u32,
}
// Represents one 1 meter cube
#[derive(Copy, Clone)]
struct Voxel {
    material_id: u32,
    extra_data: Option<DataSegment>,
}
/// The type of a voxel
#[derive(Clone)]
struct VoxelType {
    material_id: u32,
    /// 256 character name
    name: DataSegment,
    /// if the voxel is solid enough to be stood upon
    solid: bool,
}
/// Represents a collection of voxels that may be loaded and unloaded together
#[derive(Clone)]
struct Chunk {
    /// the voxels contained within this chunk
    voxels: [[[u32; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    /// Extra data
    extra_data: Option<DataSegment>,
}

/// Represents many chunks that form a world
#[derive(Clone)]
struct Dimension {
    /// The chunks that are actually loaded
    loaded_chunks: HashMap<ChunkLocation, Chunk>,
    /// List of all chunk locations that are specially defined
    all_chunk_locations: HashSet<ChunkLocation>,
    ///the folder where the dimension will be saved
    disk_cache: Option<String>,
}

/// TODO please flesh out the scope struct. It represents an arbitrary 3d
/// portion of the world that is backed up by chunks, kinda like a world

impl DataSegment {
    fn new() -> DataSegment {
        DataSegment {
            data: [0; DATA_SEGMENT_SIZE],
        }
    }

    /// Creates data segment from string, truncating it if it is too long
    fn from(string: &str) -> DataSegment {
        let mut data = DataSegment::new();
        for i in 0..(std::cmp::min(string.len(), data.data.len()) - 1) {
            data.data[i] = string.as_bytes()[i];
        }
        data
    }
}

impl Voxel {
    /// Creates new voxel from voxel type
    fn new(voxel_type: VoxelType) -> Voxel {
        Voxel {
            material_id: voxel_type.material_id,
            extra_data: None,
        }
    }

    fn from_id(material_id: u32) -> Voxel {
        Voxel {
            material_id: material_id,
            extra_data: None,
        }
    }

    /// Returns the voxeltype of the
    fn get_type(&self) -> VoxelType {
        match self.material_id {
            0 => VoxelType {
                material_id: 0,
                name: DataSegment::from("unknown"),
                solid: true,
            },
            1 => VoxelType {
                material_id: 1,
                name: DataSegment::from("air"),
                solid: false,
            },
            2 => VoxelType {
                material_id: 2,
                name: DataSegment::from("water"),
                solid: false,
            },
            3 => VoxelType {
                material_id: 3,
                name: DataSegment::from("stone"),
                solid: true,
            },
            _ => panic!("material id undefined"),
        }
    }
}

impl Chunk {
    /// a new chunk initialized to all zeros
    fn new() -> Chunk {
        Chunk::from_value(0)
    }

    fn from_value(value: u32) -> Chunk {
        Chunk::from_value_with_extra_data(value, None)
    }

    fn from_value_with_extra_data(value: u32, extra_data: Option<DataSegment>) -> Chunk {
        Chunk {
            voxels: [[[value; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            extra_data: extra_data,
        }
    }

    fn from_buf_reader(stream: &mut BufReader<File>) -> Chunk {
        let mut chunk = Chunk::new();
        chunk.read(stream);
        chunk
    }

    fn get(&self, location: VoxelLocation) -> Voxel {
        Voxel::from_id(self.voxels[location.z as usize][location.y as usize][location.x as usize])
    }

    fn set(&mut self, location: VoxelLocation, voxel: Voxel) -> () {
        self.voxels[location.z as usize][location.y as usize][location.x as usize] =
            voxel.material_id;
    }

    /// Reads from saved file
    fn read(&mut self, stream: &mut BufReader<File>) -> () {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                stream
                    .read_u32_into::<NativeEndian>(&mut self.voxels[z as usize][y as usize])
                    .unwrap();
            }
        }
        //TODO please read extra data at end into extra_data, if it exists
    }

    /// writes to file
    fn write(stream: &mut BufWriter<File>, chunk: Chunk) -> () {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    stream
                        .write_u32::<NativeEndian>(chunk.voxels[z as usize][y as usize][x as usize])
                        .unwrap();
                }
            }
        }
        if chunk.extra_data.is_some() {
            stream.write_all(&chunk.extra_data.unwrap().data).unwrap();
        }
        stream.flush().unwrap();
    }
}

impl Dimension {
    fn new() -> Dimension {
        Dimension {
            loaded_chunks: HashMap::new(),
            all_chunk_locations: HashSet::new(),
            disk_cache: None, //TODO please set disk cache and figure this out
        }
    }

    /// Adds a chunk to the location
    fn add_chunk_in_place(&mut self, location: ChunkLocation, chunk: Chunk) -> () {
        self.all_chunk_locations.insert(location);
        self.loaded_chunks.insert(location, chunk);
    }

    /// Remove chunk from location, if it exists
    fn remove_chunk_in_place(&mut self, location: ChunkLocation) -> () {
        self.all_chunk_locations.remove(&location.clone());
        self.loaded_chunks.remove(&location.clone());
    }

    /// Gets a chunk, loading it if unavailable
    fn get_chunk(&mut self, location: ChunkLocation) -> &Chunk {
        if !self.chunk_defined(location) {
            panic!("chunk undefined");
        } else if !self.chunk_loaded(location) {
            self.load_chunk(location);
        }
        self.loaded_chunks.get(&location).unwrap()
    }

    /// If a chunk has been loaded
    fn chunk_loaded(&self, location: ChunkLocation) -> bool {
        self.loaded_chunks.contains_key(&location)
    }

    /// If a chunk has been defined to exist
    fn chunk_defined(&self, location: ChunkLocation) -> bool {
        self.all_chunk_locations.contains(&location)
    }

    /// Loads chunk from disk
    fn load_chunk(&mut self, location: ChunkLocation) -> () {
        //TODO load chunk from disk cache
    }

    /// writes out all chunks to disk
    fn flush(&mut self) -> () {
        //TODO implement flush, write out all chunks to disk
    }

    /// Gets the location of the chunk where this voxel lies
    fn get_chunk_location(location: GlobalLocation) -> ChunkLocation {
        ChunkLocation {
            x: location.x / (CHUNK_SIZE as u32),
            y: location.y / (CHUNK_SIZE as u32),
            z: location.z / (CHUNK_SIZE as u32),
        }
    }

    /// Gets the location of the voxel in the chunk where this global location lies
    fn get_voxel_location(location: GlobalLocation) -> VoxelLocation {
        VoxelLocation {
            x: location.x % (CHUNK_SIZE as u32),
            y: location.y % (CHUNK_SIZE as u32),
            z: location.z % (CHUNK_SIZE as u32),
        }
    }

    /// gets voxel at location if available. It is preffered to use get_Scope for better
    /// performance
    fn get_voxel(&mut self, location: GlobalLocation) -> Voxel {
        self.get_chunk(Self::get_chunk_location(location))
            .get(Self::get_voxel_location(location))
    }
}

fn main() {}
