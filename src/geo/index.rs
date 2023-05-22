use super::{GeoBBox, GeoFile, GeoNode};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf, result::Result};

#[derive(Serialize, Deserialize)]
pub struct GeoIndex {
	nodes: Vec<GeoNode>,
}
impl GeoIndex {
	pub fn create(filename_index: &PathBuf, geo_data: &mut GeoFile) -> Result<Self, Box<dyn Error>> {
		let mut entries = geo_data.get_entries()?;
		let mut index = GeoIndex { nodes: Vec::new() };
		index.create_tree(entries.as_mut_slice());
		index.save(filename_index)?;
		Ok(index)
	}
	pub fn load(filename_index: &PathBuf) -> Result<Self, Box<dyn Error>> {
		let bytes = fs::read(filename_index)?;
		let index = bincode::deserialize(&bytes)?;
		Ok(index)
	}
	fn save(&self, filename_index: &PathBuf) -> Result<(), Box<dyn Error>> {
		fs::write(filename_index, bincode::serialize(self)?)?;
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
	pub fn collect_leaves(&self, bbox: &GeoBBox, start_index: usize, max_count: usize) -> (Vec<&GeoNode>, usize) {
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
