use neon::types::Finalize;
use std::{
	fs::File,
	io::{BufReader, Read},
	ops::Range,
	path::PathBuf,
};

pub struct GeoFile {
	data: Box<dyn GeoDataTrait>,
	index: GeoIndex,
}
unsafe impl Send for GeoFile {}

impl GeoFile {
	pub fn open(filename: &PathBuf, in_memory: bool) -> Self {
		let mut filename_index = filename.clone();
		filename_index.set_extension("index");

		let data: Box<dyn GeoDataTrait> = if in_memory {
			Box::new(GeoDataMemory::load(&filename))
		} else {
			Box::new(GeoDataFile::load(&filename))
		};

		let index = if filename_index.exists() {
			GeoIndex::load(&filename_index)
		} else {
			GeoIndex::create(&filename_index, data.as_ref())
		};

		return GeoFile { data, index };
	}

	pub fn find(&self, bbox: &GeoBBox) -> String {
		let leaves = self.index.collect_leaves(&bbox);
		let leaves: Vec<String> = leaves
			.iter()
			.map(|node| self.data.get_range(node.value1, node.value2))
			.collect();
		return "[".to_string() + &leaves.join(",") + "]";
	}
}

impl Finalize for GeoFile {}

trait GeoDataTrait {
	fn load(filename: &PathBuf) -> Self
	where
		Self: Sized;
	fn get_range(&self, start: usize, end: usize) -> String;
	fn get_entries(&self) -> Vec<GeoEntry>;
}

struct GeoDataMemory {
	content: String,
}
impl GeoDataTrait for GeoDataMemory {
	fn load(filename: &PathBuf) -> Self {
		let file = File::open(filename).unwrap();
		let mut buf_reader = BufReader::new(file);
		let mut content = String::new();
		buf_reader.read_to_string(&mut content).unwrap();
		return Self { content };
	}

	fn get_range(&self, start: usize, end: usize) -> String {
		return self.content[start..end].to_string();
	}

	fn get_entries(&self) -> Vec<GeoEntry> {
		todo!()
	}
}

struct GeoDataFile {}
impl GeoDataTrait for GeoDataFile {
	fn load(filename: &PathBuf) -> Self {
		todo!()
	}

	fn get_range(&self, start: usize, end: usize) -> String {
		todo!()
	}

	fn get_entries(&self) -> Vec<GeoEntry> {
		todo!()
	}
}

struct GeoIndex {
	nodes: Vec<GeoNode>,
}
impl GeoIndex {
	fn create(filename_index: &PathBuf, geo_data: &dyn GeoDataTrait) -> Self {
		let mut entries = geo_data.get_entries();
		let mut index = GeoIndex { nodes: Vec::new() };
		index.create_tree(entries.as_mut_slice());
		return index;
	}
	fn load(filename_index: &PathBuf) -> Self {
		todo!()
	}
	fn save(filename_index: &PathBuf) {}
	fn create_tree(&mut self, entries: &mut [GeoEntry]) -> usize {
		if entries.len() == 1 {
			let entry = &entries[0];
			let index = self.nodes.len();
			self.nodes.push(GeoNode {
				bbox: entry.bbox.clone(),
				is_leaf: true,
				value1: entry.range.start,
				value2: entry.range.end,
			});
			return index;
		} else {
			let mut bbox = GeoBBox::new_empty();
			for entry in entries.iter() {
				bbox.expand(&entry.bbox);
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
			let index = self.nodes.len();
			self.nodes.push(GeoNode {
				bbox,
				is_leaf: false,
				value1: 0,
				value2: 0,
			});
			let value1 = self.create_tree(part1);
			let value2 = self.create_tree(part2);
			let mut node = self.nodes.get_mut(index).unwrap();
			node.value1 = value1;
			node.value2 = value2;
			return index;
		}
	}
	fn collect_leaves(&self, bbox: &GeoBBox) -> Vec<&GeoNode> {
		let mut leaves: Vec<&GeoNode> = Vec::new();
		collect_leaves_rec(&self.nodes, &self.nodes[0], bbox, &mut leaves);
		return leaves;

		fn collect_leaves_rec<'a>(
			nodes: &'a Vec<GeoNode>, node: &'a GeoNode, bbox: &GeoBBox, results: &mut Vec<&'a GeoNode>,
		) {
			if node.bbox.overlap(bbox) {
				if node.is_leaf {
					results.push(node);
				} else {
					collect_leaves_rec(nodes, &nodes[node.value1], bbox, results);
					collect_leaves_rec(nodes, &nodes[node.value2], bbox, results);
				}
			}
		}
	}
}

struct GeoNode {
	bbox: GeoBBox,
	is_leaf: bool,
	value1: usize,
	value2: usize,
}

#[derive(Clone)]
pub struct GeoBBox {
	x_min: f32,
	x_max: f32,
	y_min: f32,
	y_max: f32,
}

impl GeoBBox {
	pub fn new(x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Self {
		GeoBBox {
			x_min,
			x_max,
			y_min,
			y_max,
		}
	}
	fn new_empty() -> Self {
		GeoBBox {
			x_min: f32::MAX,
			x_max: f32::MIN,
			y_min: f32::MAX,
			y_max: f32::MIN,
		}
	}
	fn expand(&mut self, bbox: &GeoBBox) {
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
	fn overlap(&self, bbox: &GeoBBox) -> bool {
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
		return true;
	}
}

struct GeoEntry {
	pub bbox: GeoBBox,
	range: Range<usize>,
}
