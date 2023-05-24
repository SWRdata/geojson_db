use super::GeoBBox;
use serde::{Deserialize, Serialize};

/// The GeoNode struct represents a node in a tree structure that is used for spatial indexing.
#[derive(Serialize, Deserialize, Clone)]
pub struct GeoNode {
	/// The bounding box of the geographic area covered by this node.
	pub bbox: GeoBBox,
	/// Indicates whether this node is a leaf node (true) or not (false).
	pub is_leaf: bool,
	/// For nodes, this represents the index to the left child. For leaves, it's the offset in the file.
	pub value1: usize,
	/// For nodes, this represents the index to the right child. For leaves, it's the length in the file.
	pub value2: usize,
	/// Index to the next sibling node. This refers to the next node in the traversal order that is not part of the current branch.
	///
	/// The following rules are used to determine the next sibling:
	/// 1. Check if the current node is a left or right child of its parent.
	///    - If the current node is a left child, then the "next sibling" is the right child of the parent. In this case, we're done.
	///    - If the current node is a right child, move upwards by treating the parent of the current node as the new current node, and then repeat the process.
	/// 2. If no "next sibling" can be found using the steps above, set 'next' to 0. This value signifies the end of the traversal.
	///
	/// This field aids in the efficient traversal of the tree structure.
	pub next: usize,
}

impl GeoNode {
	/// Creates a new leaf node.
	///
	/// # Arguments
	///
	/// * `bbox` - A GeoBBox instance representing the bounding box of the geographic area.
	/// * `start` - The offset in the file for this leaf node.
	/// * `length` - The length in the file for this leaf node.
	///
	/// # Returns
	///
	/// A new GeoNode instance.
	pub fn new_leaf(bbox: GeoBBox, start: usize, length: usize) -> Self {
		Self {
			bbox,
			is_leaf: true,
			value1: start,
			value2: length,
			next: 0,
		}
	}

	/// Creates a new empty node.
	///
	/// # Arguments
	///
	/// * `bbox` - A GeoBBox instance representing the bounding box of the geographic area.
	///
	/// # Returns
	///
	/// A new GeoNode instance.
	pub fn new_node(bbox: GeoBBox) -> Self {
		Self {
			bbox,
			is_leaf: false,
			value1: 0,
			value2: 0,
			next: 0,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_leaf() {
		let bbox = GeoBBox::new(1.0, 2.0, 3.0, 4.0);
		let leaf = GeoNode::new_leaf(bbox.clone(), 10, 20);
		assert_eq!(leaf.bbox, bbox);
		assert_eq!(leaf.is_leaf, true);
		assert_eq!(leaf.value1, 10);
		assert_eq!(leaf.value2, 20);
		assert_eq!(leaf.next, 0);
	}

	#[test]
	fn test_new_node() {
		let bbox = GeoBBox::new(1.0, 2.0, 3.0, 4.0);
		let node = GeoNode::new_node(bbox.clone());
		assert_eq!(node.bbox, bbox);
		assert_eq!(node.is_leaf, false);
		assert_eq!(node.value1, 0);
		assert_eq!(node.value2, 0);
		assert_eq!(node.next, 0);
	}
}
