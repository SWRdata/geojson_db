use serde::{Deserialize, Serialize};

// GeoBBox struct representing a geographical bounding box with min and max coordinates
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct GeoBBox {
	x_min: f32, // minimum x-coordinate
	x_max: f32, // maximum x-coordinate
	y_min: f32, // minimum y-coordinate
	y_max: f32, // maximum y-coordinate
}

// Implementation of methods for GeoBBox struct
impl GeoBBox {
	// Create a new GeoBBox given min and max coordinates
	pub fn new(x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Self {
		GeoBBox {
			x_min,
			x_max,
			y_min,
			y_max,
		}
	}
	// Create a new GeoBBox from a point
	pub fn new_point(x: f32, y: f32) -> Self {
		GeoBBox {
			x_min: x,
			x_max: x,
			y_min: y,
			y_max: y,
		}
	}
	// Create an "empty" GeoBBox with extreme values
	pub fn new_empty() -> Self {
		GeoBBox {
			x_min: f32::MAX,
			x_max: f32::MIN,
			y_min: f32::MAX,
			y_max: f32::MIN,
		}
	}
	// Include a point into the GeoBBox, potentially extending it
	pub fn include_point(&mut self, x: f32, y: f32) {
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
	// Expand current GeoBBox to include another GeoBBox
	pub fn include_bbox(&mut self, bbox: &GeoBBox) {
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
	// Check if GeoBBox is wider than high
	pub fn is_horizontal(&self) -> bool {
		(self.x_max - self.x_min) > (self.y_max - self.y_min)
	}
	// Check if current GeoBBox overlaps with another GeoBBox
	pub fn overlap_bbox(&self, bbox: &GeoBBox) -> bool {
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
	// Compute the sum of x coordinates
	pub fn sum_x(&self) -> f32 {
		self.x_min + self.x_max
	}
	// Compute the sum of y coordinates
	pub fn sum_y(&self) -> f32 {
		self.y_min + self.y_max
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new() {
		let bbox = GeoBBox::new(1.0, 2.0, 3.0, 4.0);
		assert_eq!(bbox.x_min, 1.0);
		assert_eq!(bbox.x_max, 2.0);
		assert_eq!(bbox.y_min, 3.0);
		assert_eq!(bbox.y_max, 4.0);
	}

	#[test]
	fn test_new_point() {
		let bbox = GeoBBox::new_point(1.0, 2.0);
		assert_eq!(bbox.x_min, 1.0);
		assert_eq!(bbox.x_max, 1.0);
		assert_eq!(bbox.y_min, 2.0);
		assert_eq!(bbox.y_max, 2.0);
	}

	#[test]
	fn test_new_empty() {
		let bbox = GeoBBox::new_empty();
		assert_eq!(bbox.x_min, f32::MAX);
		assert_eq!(bbox.x_max, f32::MIN);
		assert_eq!(bbox.y_min, f32::MAX);
		assert_eq!(bbox.y_max, f32::MIN);
	}

	#[test]
	fn test_include_point() {
		let mut bbox = GeoBBox::new_empty();
		bbox.include_point(1.0, 2.0);
		assert_eq!(bbox.x_min, 1.0);
		assert_eq!(bbox.x_max, 1.0);
		assert_eq!(bbox.y_min, 2.0);
		assert_eq!(bbox.y_max, 2.0);
	}

	#[test]
	fn test_include_bbox() {
		let mut bbox = GeoBBox::new(1.0, 2.0, 1.0, 2.0);
		let bbox2 = GeoBBox::new(0.0, 3.0, 0.0, 3.0);
		bbox.include_bbox(&bbox2);
		assert_eq!(bbox.x_min, 0.0);
		assert_eq!(bbox.x_max, 3.0);
		assert_eq!(bbox.y_min, 0.0);
		assert_eq!(bbox.y_max, 3.0);
	}

	#[test]
	fn test_is_horizontal() {
		let bbox = GeoBBox::new(1.0, 3.0, 1.0, 2.0);
		assert_eq!(bbox.is_horizontal(), true);
	}

	#[test]
	fn test_overlap_bbox() {
		let bbox = GeoBBox::new(1.0, 3.0, 1.0, 3.0);
		let bbox2 = GeoBBox::new(2.0, 4.0, 2.0, 4.0);
		assert_eq!(bbox.overlap_bbox(&bbox2), true);
	}

	#[test]
	fn test_sum_x() {
		let bbox = GeoBBox::new(1.0, 2.0, 1.0, 2.0);
		assert_eq!(bbox.sum_x(), 3.0);
	}

	#[test]
	fn test_sum_y() {
		let bbox = GeoBBox::new(1.0, 2.0, 1.0, 2.0);
		assert_eq!(bbox.sum_y(), 3.0);
	}
}
