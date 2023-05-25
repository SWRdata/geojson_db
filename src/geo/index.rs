use super::{GeoBBox, GeoFile, GeoNode};
use serde::{Deserialize, Serialize};
use std::{
	error::Error,
	fs::{read, write, File},
	io::{BufWriter, Seek, Write},
	path::Path,
	result::Result,
	time::Instant,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoIndex {
	nodes: Vec<GeoNode>,
}
impl GeoIndex {
	pub fn create(geo_data: &mut GeoFile, filename_index: &Path, filename_table: &Path) -> Result<Self, Box<dyn Error>> {
		let mut entries = geo_data.get_entries()?;
		let mut index = GeoIndex { nodes: Vec::new() };
		index.create_tree(entries.as_mut_slice());
		index.rewrite_table(geo_data, filename_table)?;
		index.save(filename_index)?;
		Ok(index)
	}
	pub fn load(filename_index: &Path) -> Result<Self, Box<dyn Error>> {
		let bytes = read(filename_index)?;
		let index = bincode::deserialize(&bytes)?;
		Ok(index)
	}
	fn save(&self, filename_index: &Path) -> Result<(), Box<dyn Error>> {
		write(filename_index, bincode::serialize(self)?)?;
		Ok(())
	}
	fn rewrite_table(&mut self, geo_data: &mut GeoFile, filename_table: &Path) -> Result<(), Box<dyn Error>> {
		let mut file = BufWriter::new(File::create(filename_table)?);
		let mut pos: usize = 0;
		let start = Instant::now();

		for i in 0..self.nodes.len() {
			if i % 1000000 == 0 {
				println!(
					"rewrite_table: {}, {:.1}%, {:.0}/s, {:.1}MB/s",
					i,
					100. * i as f64 / self.nodes.len() as f64,
					i as f64 / start.elapsed().as_secs_f64(),
					file.stream_position()? as f64 / 1048576. / start.elapsed().as_secs_f64()
				)
			}

			if self.nodes[i].is_leaf {
				let node = self.nodes.get_mut(i).unwrap();
				let buffer = geo_data.read_range(node.value1, node.value2);
				node.value1 = pos;
				file.write_all(buffer)?;
				pos += node.value2;
			}
		}
		Ok(())
	}
	fn create_tree(&mut self, leaves: &mut [GeoNode]) {
		create_tree_rec(leaves, &mut self.nodes);
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

		fn create_tree_rec(leaves: &mut [GeoNode], nodes: &mut Vec<GeoNode>) -> usize {
			if leaves.len() == 1 {
				let index = nodes.len();
				nodes.push(leaves[0].clone());
				index
			} else {
				let mut bbox = GeoBBox::new_empty();
				for entry in leaves.iter() {
					bbox.include_bbox(&entry.bbox);
				}
				if bbox.is_horizontal() {
					// sort by x
					leaves.sort_unstable_by(|a, b| a.bbox.sum_x().partial_cmp(&b.bbox.sum_x()).unwrap())
				} else {
					// sort by y
					leaves.sort_unstable_by(|a, b| a.bbox.sum_y().partial_cmp(&b.bbox.sum_y()).unwrap())
				}
				let (part1, part2) = leaves.split_at_mut(leaves.len() / 2);
				let index = nodes.len();
				nodes.push(GeoNode::new_node(bbox));
				let value1 = create_tree_rec(part1, nodes);
				let value2 = create_tree_rec(part2, nodes);
				let node = nodes.get_mut(index).unwrap();
				node.value1 = value1;
				node.value2 = value2;
				index
			}
		}
	}
	pub fn query_bbox(&self, bbox: &GeoBBox, start_index: usize, max_count: usize) -> (Vec<&GeoNode>, usize) {
		let mut leaves: Vec<&GeoNode> = Vec::with_capacity(max_count);
		let mut index = start_index;

		loop {
			let node = &self.nodes[index];
			if node.bbox.overlap_bbox(bbox) {
				if node.is_leaf {
					leaves.push(node);
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::geo::GeoFileOptions;
	use assert_fs::NamedTempFile;

	#[test]
	fn test_create_and_load_geo_index() -> Result<(), Box<dyn Error>> {
		let filename = Path::new("testdata/polygons.geojsonl.br");
		let filename_index = NamedTempFile::new("temp.idx")?;
		let filename_table = NamedTempFile::new("temp.dat")?;

		let mut geo_data = GeoFile::load(&filename, GeoFileOptions::empty())?;

		let geo_index1 = GeoIndex::create(&mut geo_data, filename_index.path(), filename_table.path())?;
		let geo_index2 = GeoIndex::load(&filename_index)?;
		let bbox = GeoBBox::new(10., 10.2, 51., 51.2);
		let node1 = GeoNode {
			is_leaf: true,
			bbox: GeoBBox::new(10.1946335, 10.1953125, 51.10852, 51.10955),
			value1: 1420116,
			value2: 696,
			next: 3914,
		};
		let node2 = GeoNode {
			is_leaf: true,
			bbox: GeoBBox::new(10.1953125, 10.195699, 51.10852, 51.109306),
			value1: 1420812,
			value2: 648,
			next: 3915,
		};

		for geo_index in vec![geo_index1, geo_index2] {
			assert_eq!(geo_index.nodes.len(), 7155);

			let (leaves, index) = geo_index.query_bbox(&bbox, 0, 10);
			assert_eq!(leaves, vec![&node1, &node2]);
			assert_eq!(index, 0);

			let (leaves, index) = geo_index.query_bbox(&bbox, 0, 1);
			assert_eq!(leaves, vec![&node1]);
			assert_eq!(index, 3914);

			let (leaves, index) = geo_index.query_bbox(&bbox, 3914, 1);
			assert_eq!(leaves, vec![&node2]);
			assert_eq!(index, 3915);

			let (leaves, index) = geo_index.query_bbox(&bbox, 3915, 1);
			assert_eq!(leaves, Vec::<&GeoNode>::new());
			assert_eq!(index, 0);
		}

		Ok(())
	}
}
