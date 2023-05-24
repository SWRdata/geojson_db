use super::{GeoBBox, GeoNode};
use memmap2::Mmap;
use std::{error::Error, ffi::OsStr, fs::File, path::PathBuf, result::Result, str::from_utf8, time::Instant};

type BboxExtractor = Box<dyn Fn(&str) -> GeoBBox>;

pub struct GeoFile {
	_file: File,
	mmap: Mmap,
	extractor: BboxExtractor,
}
impl GeoFile {
	pub fn load(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
		let extractor: BboxExtractor = match filename.extension().and_then(OsStr::to_str) {
			Some("geojsonl") => Box::new(make_bbox::from_geojson),
			Some("geojson") => Box::new(make_bbox::from_geojson),
			Some("csv") => make_bbox::make_from_csv(',', 0, 1),
			Some("tsv") => make_bbox::make_from_csv('\t', 0, 1),
			_ => {
				return Err(Box::new(std::io::Error::new(
					std::io::ErrorKind::InvalidInput,
					format!("Unsupported file extension: {}", filename.to_string_lossy()),
				)))
			}
		};
		let file = File::open(filename).unwrap();
		let mmap = unsafe { Mmap::map(&file)? };
		Ok(Self {
			_file: file,
			mmap,
			extractor,
		})
	}

	pub fn read_range(&self, start: usize, length: usize) -> &[u8] {
		&self.mmap[start..start + length]
	}

	pub fn get_entries(&mut self) -> Result<Vec<GeoNode>, Box<dyn Error>> {
		let mut entries: Vec<GeoNode> = Vec::new();
		let mut line_no: usize = 0;
		let file_size: f64 = self.mmap.len() as f64 / 100.;
		let start = Instant::now();
		let mut current_pos: usize = 0;
		let extractor = &self.extractor;

		for i in 0..self.mmap.len() {
			if self.mmap[i] == 10 {
				// new line break

				line_no += 1;

				if line_no % 1000000 == 0 {
					println!(
						"get_entries: {}, {:.1}%, {:.0}/s, {:.1}MB/s",
						line_no,
						current_pos as f64 / file_size,
						line_no as f64 / start.elapsed().as_secs_f64(),
						current_pos as f64 / 1048576. / start.elapsed().as_secs_f64()
					)
				}
				let line = from_utf8(&self.mmap[current_pos..i])?;
				entries.push(GeoNode::new_leaf(extractor(line), current_pos, i - current_pos));
				current_pos = i + 1;
			}
		}

		Ok(entries)
	}
}

mod make_bbox {
	use super::BboxExtractor;
	use crate::geo::GeoBBox;
	use geojson::Feature;
	use std::str::FromStr;

	// Create a GeoBBox from a geojson String
	pub fn from_geojson(line: &str) -> GeoBBox {
		let feature = Feature::from_str(line).unwrap();
		from_geometry(&feature.geometry.unwrap())
	}

	// Create a GeoBBox from a geojson::Geometry value
	fn from_geometry(geometry: &geojson::Geometry) -> GeoBBox {
		match &geometry.value {
			geojson::Value::Point(c) => from_vec(c),
			geojson::Value::MultiPoint(c) => from_vec2(c),
			geojson::Value::LineString(c) => from_vec2(c),
			geojson::Value::MultiLineString(c) => from_vec3(c),
			geojson::Value::Polygon(c) => from_vec3(c),
			geojson::Value::MultiPolygon(c) => from_vec4(c),
			geojson::Value::GeometryCollection(c) => {
				let mut bbox = GeoBBox::new_empty();
				c.iter()
					.for_each(|geometry| bbox.include_bbox(&from_geometry(geometry)));
				bbox
			}
		}
	}

	// Create a GeoBBox from a 4D vector, encapsulating all points in the vector
	fn from_vec4(v4: &[Vec<Vec<Vec<f64>>>]) -> GeoBBox {
		let mut bbox = GeoBBox::new_empty();
		v4.iter().for_each(|v3| bbox.include_bbox(&from_vec3(v3)));
		bbox
	}

	// Create a GeoBBox from a 3D vector, encapsulating all points in the vector
	fn from_vec3(v3: &[Vec<Vec<f64>>]) -> GeoBBox {
		let mut bbox = GeoBBox::new_empty();
		v3.iter().for_each(|v2| bbox.include_bbox(&from_vec2(v2)));
		bbox
	}

	// Create a GeoBBox from a 2D vector, encapsulating all points in the vector
	fn from_vec2(v2: &[Vec<f64>]) -> GeoBBox {
		let mut bbox = GeoBBox::new_empty();
		v2.iter().for_each(|v1| bbox.include_point(v1[0] as f32, v1[1] as f32));
		bbox
	}

	// Create a GeoBBox from a 1D vector, treating both x and y as same
	fn from_vec(v1: &[f64]) -> GeoBBox {
		GeoBBox::new_point(v1[0] as f32, v1[1] as f32)
	}

	pub fn make_from_csv(sep: char, col_x: usize, col_y: usize) -> BboxExtractor {
		Box::new(move |line: &str| -> GeoBBox {
			let fields: Vec<&str> = line.split(sep).collect();
			let x: f32 = fields[col_x].parse().unwrap();
			let y: f32 = fields[col_y].parse().unwrap();
			GeoBBox::new_point(x, y)
		})
	}
}
