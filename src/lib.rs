extern crate byteorder;

use std::collections::HashMap;
use std::collections::HashSet;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

const CHUNK_X_SIZE: usize = 16;
const CHUNK_Y_SIZE: usize = 16;
const CHUNK_Z_SIZE: usize = 16;
const CHUNK_VOLUME: usize = CHUNK_X_SIZE * CHUNK_Y_SIZE * CHUNK_Z_SIZE;
const DATA_SEGMENT_SIZE: usize = 256;

/// 256 bytes of data, to be used for any purpose
#[derive(Copy, Clone)]
struct DataSegment {
    data: [u8; DATA_SEGMENT_SIZE],
}

///A point in 3D space
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
struct Point3D {
    x: u32,
    y: u32,
    z: u32,
}

/// The location of a Chunk in relation to the world
type ChunkLocation = Point3D;

/// The location of a single voxel in relation to the world
type GlobalLocation = Point3D;

/// The location of a single voxel in relation to its chunk
type VoxelLocation = Point3D;

/// Represents one 1 meter cube
#[derive(Copy, Clone)]
struct Voxel {
    value: u32,
    extra_data: Option<DataSegment>,
}

/// Represents a collection of voxels that may be loaded and unloaded together
#[derive(Clone)]
struct Chunk {
    /// the voxels contained within this chunk, it's a cube
    voxels: [u32; CHUNK_VOLUME],
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
    fn from_id_with_extra_data(value: u32, extra_data: DataSegment) -> Voxel {
        Voxel {
            value: value,
            extra_data: Some(extra_data),
        }
    }

    fn new(value: u32) -> Voxel {
        Voxel {
            value: value,
            extra_data: None,
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
            voxels: [value; CHUNK_VOLUME],
            extra_data: extra_data,
        }
    }

    fn from_buf_reader(stream: &mut BufReader<File>) -> Chunk {
        let mut chunk = Chunk::new();
        chunk.read(stream);
        chunk
    }

    fn get_index(location: VoxelLocation) -> usize {
        Self::get_index_xyz(location.x, location.y, location.z)
    }

    fn get_index_xyz(x: u32, y: u32, z: u32) -> usize {
        (z as usize) * CHUNK_X_SIZE * CHUNK_Y_SIZE + (y as usize) * CHUNK_X_SIZE + (x as usize)
    }

    fn get(&self, location: VoxelLocation) -> Voxel {
        Voxel::new(self.voxels[Self::get_index(location)])
    }

    fn set(&mut self, location: VoxelLocation, voxel: Voxel) -> () {
        self.voxels[Self::get_index(location)] = voxel.value;
    }

    /// Reads from saved file
    fn read(&mut self, stream: &mut BufReader<File>) -> () {
        stream
            .read_u32_into::<NativeEndian>(&mut self.voxels)
            .unwrap();
        //TODO please read extra data at end into extra_data, if it exists
    }

    /// writes to file
    fn write(stream: &mut BufWriter<File>, chunk: Chunk) -> () {
        for i in 0..CHUNK_VOLUME {
            stream.write_u32::<NativeEndian>(chunk.voxels[i]).unwrap();
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
            x: location.x / (CHUNK_X_SIZE as u32),
            y: location.y / (CHUNK_Y_SIZE as u32),
            z: location.z / (CHUNK_Z_SIZE as u32),
        }
    }

    /// Gets the location of the voxel in the chunk where this global location lies
    fn get_voxel_location(location: GlobalLocation) -> VoxelLocation {
        VoxelLocation {
            x: location.x % (CHUNK_X_SIZE as u32),
            y: location.y % (CHUNK_Y_SIZE as u32),
            z: location.z % (CHUNK_Z_SIZE as u32),
        }
    }

    /// gets voxel at location if available. It is preffered to use get_Scope for better
    /// performance
    fn get_voxel(&mut self, location: GlobalLocation) -> Voxel {
        self.get_chunk(Self::get_chunk_location(location))
            .get(Self::get_voxel_location(location))
    }
}
