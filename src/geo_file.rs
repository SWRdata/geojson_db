use geojson::Feature;
use memmap2::Mmap;
use neon::types::Finalize;
use serde::{Deserialize, Serialize};
use std::{
	error::Error,
	fs::{self, File},
	path::PathBuf,
	result::Result,
	str::{from_utf8, FromStr},
	time::Instant,
};

pub struct GeoFile {
	data: GeoDataFile,
	index: GeoIndex,
}
unsafe impl Send for GeoFile {}

impl GeoFile {
	pub fn open(filename: &PathBuf, memory_size: usize) -> Result<Self, Box<dyn Error>> {
		let mut filename_index = filename.clone();
		filename_index.set_extension("index");

		let mut data = GeoDataFile::load(filename)?;

		let index = if filename_index.exists() {
			GeoIndex::load(&filename_index)?
		} else {
			GeoIndex::create(&filename_index, &mut data)?
		};

		Ok(GeoFile { data, index })
	}

	pub fn find(
		&mut self, bbox: &GeoBBox, start_index: usize, max_count: usize,
	) -> Result<(Vec<String>, usize), Box<dyn Error>> {
		let data = &mut self.data;

		let start = Instant::now();
		let (entries, next_index) = self.index.collect_leaves(bbox, start_index, max_count);
		println!("A {:?}", start.elapsed());

		let start = Instant::now();
		let entries: Vec<String> = entries
			.iter()
			.map(|node| String::from_utf8(data.read_range(node.value1, node.value2).unwrap().to_vec()).unwrap())
			.collect();

		println!("B {:?}", start.elapsed());
		Ok((entries, next_index))
	}
}

impl Finalize for GeoFile {}

struct GeoDataFile {
	mmap: Mmap,
}
impl GeoDataFile {
	fn load(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
		let file = File::open(filename).unwrap();
		let mmap = unsafe { memmap2::Mmap::map(&file)? };
		Ok(Self { mmap })
	}

	fn read_range(&mut self, start: usize, length: usize) -> Result<&[u8], Box<dyn Error>> {
		Ok(&self.mmap[start..start + length])
	}

	fn get_entries(&mut self) -> Result<Vec<GeoEntry>, Box<dyn Error>> {
		let content = from_utf8(&self.mmap)?;

		let mut current_pos = 0;
		let mut entries: Vec<GeoEntry> = Vec::new();
		for line in content.lines() {
			let end_pos = current_pos + line.len();

			let feature = Feature::from_str(line)?;
			entries.push(GeoEntry {
				bbox: GeoBBox::from_geometry(&feature.geometry.unwrap()),
				start: current_pos,
				length: end_pos - current_pos,
			});

			current_pos = end_pos + 1; // +1 for the newline character
		}

		Ok(entries)
	}
}

#[derive(Serialize, Deserialize)]
struct GeoIndex {
	nodes: Vec<GeoNode>,
}
impl GeoIndex {
	fn create(filename_index: &PathBuf, geo_data: &mut GeoDataFile) -> Result<Self, Box<dyn Error>> {
		let mut entries = geo_data.get_entries()?;
		let mut index = GeoIndex { nodes: Vec::new() };
		index.create_tree(entries.as_mut_slice());
		index.save(filename_index)?;
		Ok(index)
	}
	fn load(filename_index: &PathBuf) -> Result<Self, Box<dyn Error>> {
		let bytes = fs::read(filename_index)?;
		let index = bincode::deserialize(&bytes)?;
		Ok(index)
	}
	fn save(&self, filename_index: &PathBuf) -> Result<(), Box<dyn Error>> {
		fs::write(filename_index, bincode::serialize(self)?)?;
		Ok(())
	}
	fn create_tree(&mut self, entries: &mut [GeoEntry]) {
		create_tree_rec(entries, &mut self.nodes);
		for i in 0..self.nodes.len() {
			if self.nodes[i].is_leaf {
				continue;
			}
			let GeoNode {
				value1, value2, next, ..
			} = self.nodes[i];
			self.nodes[value1].next = value2;
			self.nodes[value2].next = next;
		}

		fn create_tree_rec(entries: &mut [GeoEntry], nodes: &mut Vec<GeoNode>) -> usize {
			if entries.len() == 1 {
				let entry = &entries[0];
				let index = nodes.len();
				nodes.push(GeoNode {
					bbox: entry.bbox.clone(),
					is_leaf: true,
					value1: entry.start,
					value2: entry.length,
					next: 0,
				});
				index
			} else {
				let mut bbox = GeoBBox::new_empty();
				for entry in entries.iter() {
					bbox.include_bbox(&entry.bbox);
				}
				if bbox.is_horizontal() {
					// sort by x
					entries.sort_unstable_by(|a, b| {
						(a.bbox.x_min + a.bbox.x_max)
							.partial_cmp(&(b.bbox.x_min + b.bbox.x_max))
							.unwrap()
					})
				} else {
					// sort by y
					entries.sort_unstable_by(|a, b| {
						(a.bbox.y_min + a.bbox.y_max)
							.partial_cmp(&(b.bbox.y_min + b.bbox.y_max))
							.unwrap()
					})
				}
				let (part1, part2) = entries.split_at_mut(entries.len() / 2);
				let index = nodes.len();
				nodes.push(GeoNode {
					bbox,
					is_leaf: false,
					value1: 0,
					value2: 0,
					next: 0,
				});
				let value1 = create_tree_rec(part1, nodes);
				let value2 = create_tree_rec(part2, nodes);
				let mut node = nodes.get_mut(index).unwrap();
				node.value1 = value1;
				node.value2 = value2;
				index
			}
		}
	}
	fn collect_leaves(&self, bbox: &GeoBBox, start_index: usize, max_count: usize) -> (Vec<&GeoNode>, usize) {
		let mut leaves: Vec<&GeoNode> = Vec::with_capacity(max_count);
		let mut index = start_index;

		loop {
			let node = &self.nodes[index];
			if node.bbox.overlap_bbox(bbox) {
				if node.is_leaf {
					leaves.push(&node);
					index = node.next;
					if leaves.len() >= max_count {
						break;
					}
				} else {
					index = node.value1;
				}
			} else {
				index = node.next;
			}
			if index == 0 {
				break;
			}
		}
		(leaves, index)
	}
}

#[derive(Serialize, Deserialize)]
struct GeoNode {
	bbox: GeoBBox,
	is_leaf: bool,
	// NODE: index to left child
	// LEAF: offset in file
	value1: usize,
	// NODE: index to right child
	// LEAF: length in file
	value2: usize,
	// index to next sibling
	next: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GeoBBox {
	x_min: f64,
	x_max: f64,
	y_min: f64,
	y_max: f64,
}

impl GeoBBox {
	pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
		GeoBBox {
			x_min,
			x_max,
			y_min,
			y_max,
		}
	}
	fn new_empty() -> Self {
		GeoBBox {
			x_min: f64::MAX,
			x_max: f64::MIN,
			y_min: f64::MAX,
			y_max: f64::MIN,
		}
	}
	fn from_vec(v1: &[f64]) -> Self {
		GeoBBox {
			x_min: v1[0],
			x_max: v1[0],
			y_min: v1[1],
			y_max: v1[1],
		}
	}
	fn from_vec2(v2: &[Vec<f64>]) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v2.iter().for_each(|v1| bbox.include_point(v1[0], v1[1]));
		bbox
	}
	fn from_vec3(v3: &[Vec<Vec<f64>>]) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v3.iter().for_each(|v2| bbox.include_bbox(&GeoBBox::from_vec2(v2)));
		bbox
	}
	fn from_vec4(v4: &[Vec<Vec<Vec<f64>>>]) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v4.iter().for_each(|v3| bbox.include_bbox(&GeoBBox::from_vec3(v3)));
		bbox
	}
	fn from_geometry(geometry: &geojson::Geometry) -> Self {
		match &geometry.value {
			geojson::Value::Point(c) => Self::from_vec(c),
			geojson::Value::MultiPoint(c) => Self::from_vec2(c),
			geojson::Value::LineString(c) => Self::from_vec2(c),
			geojson::Value::MultiLineString(c) => Self::from_vec3(c),
			geojson::Value::Polygon(c) => Self::from_vec3(c),
			geojson::Value::MultiPolygon(c) => Self::from_vec4(c),
			geojson::Value::GeometryCollection(c) => {
				let mut bbox = GeoBBox::new_empty();
				c.iter()
					.for_each(|geometry| bbox.include_bbox(&GeoBBox::from_geometry(geometry)));
				bbox
			}
		}
	}
	fn include_point(&mut self, x: f64, y: f64) {
		if self.x_min > x {
			self.x_min = x
		}
		if self.x_max < x {
			self.x_max = x
		}
		if self.y_min > y {
			self.y_min = y
		}
		if self.y_max < y {
			self.y_max = y
		}
	}
	fn include_bbox(&mut self, bbox: &GeoBBox) {
		if self.x_min > bbox.x_min {
			self.x_min = bbox.x_min
		}
		if self.x_max < bbox.x_max {
			self.x_max = bbox.x_max
		}
		if self.y_min > bbox.y_min {
			self.y_min = bbox.y_min
		}
		if self.y_max < bbox.y_max {
			self.y_max = bbox.y_max
		}
	}
	fn is_horizontal(&self) -> bool {
		(self.x_max - self.x_min) > (self.y_max - self.y_min)
	}
	fn overlap_bbox(&self, bbox: &GeoBBox) -> bool {
		if self.x_min > bbox.x_max {
			return false;
		}
		if self.x_max < bbox.x_min {
			return false;
		}
		if self.y_min > bbox.y_max {
			return false;
		}
		if self.y_max < bbox.y_min {
			return false;
		}
		true
	}
}

struct GeoEntry {
	pub bbox: GeoBBox,
	start: usize,
	length: usize,
}
