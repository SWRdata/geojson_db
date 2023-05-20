use geojson::Feature;
use neon::types::Finalize;
use std::{
	fs::File,
	io::{BufRead, BufReader, Read, Seek, SeekFrom},
	path::PathBuf,
	str::FromStr,
};

pub struct GeoFile {
	data: GeoDataFile,
	index: GeoIndex,
}
unsafe impl Send for GeoFile {}

impl GeoFile {
	pub fn open(filename: &PathBuf, memory_size: usize) -> Self {
		let mut filename_index = filename.clone();
		filename_index.set_extension("index");

		let mut data = GeoDataFile::load(&filename, memory_size);

		let index = if filename_index.exists() {
			GeoIndex::load(&filename_index)
		} else {
			GeoIndex::create(&filename_index, &mut data)
		};

		return GeoFile { data, index };
	}

	pub fn find(&mut self, bbox: &GeoBBox) -> String {
		let data = &mut self.data;
		let leaves = self.index.collect_leaves(&bbox);
		let leaves: Vec<String> = leaves
			.iter()
			.map(|node| data.read_range(node.value1, node.value2))
			.collect();
		return "[".to_string() + &leaves.join(",") + "]";
	}
}

impl Finalize for GeoFile {}

struct GeoDataFile {
	reader: BufReader<File>,
}
impl GeoDataFile {
	fn load(filename: &PathBuf, memory_size: usize) -> Self {
		let file = File::open(filename).unwrap();
		let reader = BufReader::with_capacity(memory_size, file);
		return Self { reader };
	}

	fn read_range(&mut self, start: usize, length: usize) -> String {
		self.reader.seek(SeekFrom::Start(start as u64)).unwrap();
		let mut buffer = vec![0; length];
		self.reader.read_exact(&mut buffer).unwrap();
		return String::from_utf8(buffer).unwrap();
	}

	fn get_entries(&mut self) -> Vec<GeoEntry> {
		self.reader.seek(SeekFrom::Start(0)).unwrap();

		let mut current_pos = 0;
		let mut line = String::new();

		let mut entries: Vec<GeoEntry> = Vec::new();
		while self.reader.read_line(&mut line).unwrap() > 0 {
			let end_pos = self.reader.stream_position().unwrap() as usize;

			let feature = Feature::from_str(&line).unwrap();
			entries.push(GeoEntry {
				bbox: GeoBBox::from_geometry(&feature.geometry.unwrap()),
				start: current_pos,
				length: end_pos - current_pos,
			});

			current_pos = end_pos;
			line.clear();
		}
		return entries;
	}
}

struct GeoIndex {
	nodes: Vec<GeoNode>,
}
impl GeoIndex {
	fn create(filename_index: &PathBuf, geo_data: &mut GeoDataFile) -> Self {
		let mut entries = geo_data.get_entries();
		let mut index = GeoIndex { nodes: Vec::new() };
		index.create_tree(entries.as_mut_slice());
		index.save(filename_index);
		return index;
	}
	fn load(filename_index: &PathBuf) -> Self {
		todo!()
	}
	fn save(&self, filename_index: &PathBuf) {
		todo!()
	}
	fn create_tree(&mut self, entries: &mut [GeoEntry]) -> usize {
		if entries.len() == 1 {
			let entry = &entries[0];
			let index = self.nodes.len();
			self.nodes.push(GeoNode {
				bbox: entry.bbox.clone(),
				is_leaf: true,
				value1: entry.start,
				value2: entry.length,
			});
			return index;
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
			if node.bbox.overlap_bbox(bbox) {
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
	fn from_vec(v1: &Vec<f64>) -> Self {
		GeoBBox {
			x_min: v1[0],
			x_max: v1[0],
			y_min: v1[1],
			y_max: v1[1],
		}
	}
	fn from_vec2(v2: &Vec<Vec<f64>>) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v2.iter().for_each(|v1| bbox.include_point(v1[0], v1[1]));
		return bbox;
	}
	fn from_vec3(v3: &Vec<Vec<Vec<f64>>>) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v3.iter().for_each(|v2| bbox.include_bbox(&GeoBBox::from_vec2(v2)));
		return bbox;
	}
	fn from_vec4(v4: &Vec<Vec<Vec<Vec<f64>>>>) -> Self {
		let mut bbox = GeoBBox::new_empty();
		v4.iter().for_each(|v3| bbox.include_bbox(&GeoBBox::from_vec3(v3)));
		return bbox;
	}
	fn from_geometry(geometry: &geojson::Geometry) -> Self {
		match &geometry.value {
			geojson::Value::Point(c) => Self::from_vec(&c),
			geojson::Value::MultiPoint(c) => Self::from_vec2(&c),
			geojson::Value::LineString(c) => Self::from_vec2(&c),
			geojson::Value::MultiLineString(c) => Self::from_vec3(&c),
			geojson::Value::Polygon(c) => Self::from_vec3(&c),
			geojson::Value::MultiPolygon(c) => Self::from_vec4(&c),
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
		return true;
	}
}

struct GeoEntry {
	pub bbox: GeoBBox,
	start: usize,
	length: usize,
}
