#![allow(dead_code, unused_variables)]

extern crate byteorder;

use std::cmp::Ordering;

use std::hash::{Hash, Hasher};

use std::collections::hash_map::DefaultHasher;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;

use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;

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
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq)]
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

/// Represents a collection of voxels that may be loaded and unloaded together
#[derive(Clone)]
struct Chunk<T> {
    /// the voxels contained within this chunk, it's a cube
    voxels: [T; CHUNK_VOLUME],
    /// Extra data
    extra_data: Option<DataSegment>,
}

/// Represents many chunks that form a world
#[derive(Clone)]
struct Dimension<T> {
    /// The chunks that are actually loaded
    loaded_chunks: HashMap<ChunkLocation, Chunk<T>>,
    /// List of all chunk locations that are specially defined
    all_chunk_locations: HashSet<ChunkLocation>,
    ///the folder where the dimension will be saved
    disk_cache: Option<String>,
}

///Represents a particular section of a dimension
#[derive(Clone)]
struct Volume<T> {
    start_location: GlobalLocation,
    end_location: GlobalLocation,
    x_size: u32,
    y_size: u32,
    z_size: u32,
    voxels: Vec<T>,
}

impl Point3D {
    fn new(x: u32, y: u32, z: u32) -> Point3D {
        Point3D { x: x, y: y, z: z }
    }
}

impl std::ops::Add for Point3D {
    type Output = Point3D;

    fn add(self, other: Point3D) -> Point3D {
        Point3D {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl std::ops::Sub for Point3D {
    type Output = Point3D;

    fn sub(self, other: Point3D) -> Point3D {
        Point3D {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
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

impl<T: Copy + Default> Chunk<T> {
    fn new() -> Chunk<T> {
        Chunk::from_value(Default::default())
    }

    /// a new chunk initialized to all value
    fn from_value(value: T) -> Chunk<T> {
        Chunk::from_value_with_extra_data(value, None)
    }

    fn from_value_with_extra_data(value: T, extra_data: Option<DataSegment>) -> Chunk<T> {
        Chunk {
            voxels: [value; CHUNK_VOLUME],
            extra_data: extra_data,
        }
    }

    fn from_buf_reader(stream: &mut BufReader<File>) -> Chunk<T> {
        let mut chunk = Chunk::new();
        chunk.read(stream);
        chunk
    }

    fn get_index(location: VoxelLocation) -> usize {
        (location.z as usize) * CHUNK_X_SIZE * CHUNK_Y_SIZE
            + (location.y as usize) * CHUNK_X_SIZE
            + (location.x as usize)
    }

    fn get(&self, location: VoxelLocation) -> T {
        self.voxels[Self::get_index(location)]
    }

    fn set(&mut self, location: VoxelLocation, value: T) -> () {
        self.voxels[Self::get_index(location)] = value;
    }

    /// Reads from saved file
    fn read(&mut self, stream: &mut BufReader<File>) -> () {
        //TODO implement serde serialization
    }

    /// writes to file
    fn write(stream: &mut BufWriter<File>, chunk: Chunk<T>) -> () {
        //TODO implement serde serialization
    }
}

impl<T: Copy + Default> Dimension<T> {
    fn new() -> Dimension<T> {
        Dimension {
            loaded_chunks: HashMap::new(),
            all_chunk_locations: HashSet::new(),
            disk_cache: None, //TODO please set disk cache and figure this out
        }
    }

    /// Adds a chunk to the location
    fn add_chunk_in_place(&mut self, location: ChunkLocation, chunk: Chunk<T>) -> () {
        self.all_chunk_locations.insert(location);
        self.loaded_chunks.insert(location, chunk);
    }

    /// Remove chunk from location, if it exists
    fn remove_chunk_in_place(&mut self, location: ChunkLocation) -> () {
        self.all_chunk_locations.remove(&location.clone());
        self.loaded_chunks.remove(&location.clone());
    }

    /// Gets a chunk, loading it if unavailable
    fn get_chunk(&mut self, location: ChunkLocation) -> &Chunk<T> {
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

    ///Syncs the disk version to the version in memory
    fn sync_chunk(&mut self, location: ChunkLocation) -> () {
        //TODO write chunk to disk
    }

    /// writes out all chunks to disk (sync all)
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

    /// gets voxel at location if available. It is preffered to use get_Volume for better
    /// performance
    fn get_voxel(&mut self, location: GlobalLocation) -> T {
        let chunk = self.get_chunk(Self::get_chunk_location(location));
        chunk.get(Self::get_voxel_location(location))
    }
}

impl<T: Copy + Default> Volume<T> {
    fn new(start_location: GlobalLocation, end_location: GlobalLocation, value: T) -> Volume<T> {
        let x_size = end_location.x - start_location.x;
        let y_size = end_location.y - start_location.y;
        let z_size = end_location.z - start_location.z;
        Volume {
            x_size: x_size,
            y_size: y_size,
            z_size: z_size,
            start_location: start_location,
            end_location: end_location,
            voxels: vec![value; (x_size * y_size * z_size) as usize],
        }
    }

    fn get_index(&self, location: GlobalLocation) -> usize {
        (location.z * self.x_size * self.y_size + location.y * self.x_size + location.x) as usize
    }

    fn get_location(&self, index: usize) -> GlobalLocation {
        Point3D {
            z: (index as u32) / (self.x_size * self.y_size),
            y: (index as u32) % (self.x_size * self.y_size),
            x: (index as u32) % (self.y_size),
        }
    }

    fn within_bounds(&self, location: GlobalLocation) -> bool {
        self.get_index(location) < self.voxels.len()
    }

    fn get(&self, location: GlobalLocation) -> T {
        self.voxels[self.get_index(location)]
    }

    fn set(&mut self, location: GlobalLocation, value: T) -> () {
        let loc = self.get_index(location);
        self.voxels[loc] = value;
    }
}

///////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////implementation///////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////

struct VoxelType {
    id: u32,
    name: String,
    solid: bool,
}

#[derive(Clone, Copy, Default)]
struct Voxel {
    id: u32,
    extra_data: Option<DataSegment>,
}

impl Voxel {
    fn get_type(&self) -> VoxelType {
        match self.id {
            0 => VoxelType {
                id: 0,
                name: String::from("unknown"),
                solid: true,
            },
            1 => VoxelType {
                id: 1,
                name: String::from("air"),
                solid: false,
            },
            2 => VoxelType {
                id: 2,
                name: String::from("water"),
                solid: false,
            },
            3 => VoxelType {
                id: 3,
                name: String::from("stone"),
                solid: true,
            },
            _ => panic!("material id undefined"),
        }
    }
}

#[derive(Clone, Copy, Default, Hash, Eq, PartialEq)]
struct Node {
    location: GlobalLocation,
    cost: u32,
}

impl Node {
    fn calculate_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we order randomly (by hash)
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.calculate_hash().cmp(&other.calculate_hash()))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// If the current location can be travelled by a droid
fn is_traversable(map: &Volume<Voxel>, location: GlobalLocation) -> bool {
    let location_underneath = GlobalLocation::new(location.x, location.y, location.z - 1);
    //check that the current location and the location underneath are defined
    (map.within_bounds(location) && map.within_bounds(location_underneath)
     //check that current location is not solid
     && (!map.get(location).get_type().solid)
     //check that location down one must be solid
     && (map.get(location_underneath).get_type().solid))
}

fn get_djikstra_map(map: &Volume<Voxel>, weights: Vec<(GlobalLocation, u32)>) -> Volume<u32> {
    // The nodes that are on the exploring front of the djikstra map
    let mut frontier: BinaryHeap<Node> = BinaryHeap::new();
    // The nodes that used to be on the exploring front
    let mut visited: HashSet<Node> = HashSet::new();

    // insert original weights into node tree
    for (location, weight) in weights.iter() {
        frontier.push(Node {
            location: location.clone(),
            cost: weight.clone(),
        });
    }

    //while there are still pending nodes
    while frontier.len() > 0 {
        let current_node = frontier.pop().unwrap();
        visited.insert(current_node);
        for location in [
            current_node.location - GlobalLocation::new(1, 0, 0),
            current_node.location + GlobalLocation::new(1, 0, 0),
            current_node.location - GlobalLocation::new(0, 1, 0),
            current_node.location + GlobalLocation::new(0, 1, 0),
            current_node.location - GlobalLocation::new(0, 0, 1),
            current_node.location + GlobalLocation::new(0, 0, 1),
        ]
        .iter()
        {
            //if it can be traversed,
            if is_traversable(map, location.clone())
                    //if it has not been visited
                    && !visited.iter().find(|x| &x.location == location).is_some()
            {
                // add it to the priority queue
                frontier.push(Node {
                    location: location.clone(),
                    cost: current_node.cost + 1,
                });
            }
        }
    }

    // Create djikstra map
    let mut potential_map: Volume<u32> =
        Volume::new(map.start_location, map.end_location, u32::max_value());
    //overwrite map with nodes
    for node in visited.iter() {
        potential_map.set(node.location, node.cost);
    }
    potential_map
}
