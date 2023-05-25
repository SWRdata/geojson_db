use super::{GeoBBox, GeoNode};
use brotli_decompressor::{self, BrotliDecompress};
use libflate::gzip::Decoder;
use std::{
	error::Error,
	ffi::OsStr,
	fs::{read, File},
	io::Read,
	path::{Path, PathBuf},
	result::Result,
	str::from_utf8,
	time::Instant,
};

type BboxExtractor = Box<dyn Fn(&str) -> GeoBBox>;

enum Compression {
	Brotli,
	Gzip,
	None,
}

#[derive(Debug)]
pub struct GeoFileOptions {
	pub separator: Option<String>,
	pub col_x: Option<usize>,
	pub col_y: Option<usize>,
	pub skip_lines: Option<usize>,
}

pub struct GeoFile {
	data: Vec<u8>,
	extractor: BboxExtractor,
	skip_lines: usize,
}
impl GeoFile {
	pub fn load(filename: &PathBuf, opt: GeoFileOptions) -> Result<Self, Box<dyn Error>> {
		let (basename, compression) = GeoFile::get_compression(filename);
		let extractor: BboxExtractor = GeoFile::get_extractor(&basename, &opt)?;

		let data = match compression {
			Compression::Brotli => {
				let mut temp = Vec::new();
				BrotliDecompress(&mut File::open(filename)?, &mut temp)?;
				temp
			}
			Compression::Gzip => {
				let mut temp = Vec::new();
				Decoder::new(&File::open(filename)?)?.read_to_end(&mut temp)?;
				temp
			}
			Compression::None => read(filename)?,
		};

		Ok(Self {
			data,
			extractor,
			skip_lines: opt.skip_lines.unwrap_or(0),
		})
	}

	fn get_compression(filename: &Path) -> (PathBuf, Compression) {
		match filename.extension().and_then(OsStr::to_str) {
			Some("br") => (filename.with_extension(""), Compression::Brotli),
			Some("gz") => (filename.with_extension(""), Compression::Gzip),
			_ => (filename.to_path_buf(), Compression::None),
		}
	}

	fn get_extractor(filename: &Path, opt: &GeoFileOptions) -> Result<BboxExtractor, Box<dyn Error>> {
		match filename.extension().and_then(OsStr::to_str) {
			Some("geojsonl") => Ok(Box::new(make_bbox::from_geojson)),
			Some("geojson") => Ok(Box::new(make_bbox::from_geojson)),
			Some("csv") => Ok(make_bbox::make_from_csv(
				opt.separator.clone().unwrap_or(String::from(",")),
				opt.col_x.unwrap_or(0),
				opt.col_y.unwrap_or(1),
			)),
			Some("tsv") => Ok(make_bbox::make_from_csv(
				opt.separator.clone().unwrap_or(String::from("\t")),
				opt.col_x.unwrap_or(0),
				opt.col_y.unwrap_or(1),
			)),
			_ => Err(Box::new(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				format!("Unsupported file extension: {}", filename.to_string_lossy()),
			))),
		}
	}

	pub fn read_range(&self, start: usize, length: usize) -> &[u8] {
		&self.data[start..start + length]
	}

	pub fn get_entries(&self) -> Result<Vec<GeoNode>, Box<dyn Error>> {
		let mut entries: Vec<GeoNode> = Vec::new();
		let mut line_no: usize = 0;
		let file_size: f64 = self.data.len() as f64 / 100.;
		let start = Instant::now();
		let mut current_pos: usize = 0;
		let extractor = &self.extractor;

		for i in 0..self.data.len() {
			if self.data[i] == 10 {
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

				if line_no > self.skip_lines {
					let line = from_utf8(&self.data[current_pos..i])?;
					entries.push(GeoNode::new_leaf(extractor(line), current_pos, i - current_pos));
				}

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

	pub fn make_from_csv(separator: String, col_x: usize, col_y: usize) -> BboxExtractor {
		Box::new(move |line: &str| -> GeoBBox {
			let fields: Vec<&str> = line.split(&separator).collect();
			let x: f32 = fields[col_x].parse().unwrap();
			let y: f32 = fields[col_y].parse().unwrap();
			GeoBBox::new_point(x, y)
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::PathBuf;

	// Testing GeoFileOptions struct
	#[test]
	fn geo_file_options() {
		let options = GeoFileOptions {
			separator: Some(String::from("a")),
			col_x: Some(5),
			col_y: Some(6),
			skip_lines: Some(7),
		};

		assert_eq!(options.separator.unwrap(), "a");
		assert_eq!(options.col_x.unwrap(), 5);
		assert_eq!(options.col_y.unwrap(), 6);
		assert_eq!(options.skip_lines.unwrap(), 7);
	}

	// Testing loading a gzip compressed file
	#[test]
	fn geo_file_load_csv_gzip() -> Result<(), Box<dyn Error>> {
		let filename = PathBuf::from("testdata/points.csv.gz");
		let options = GeoFileOptions {
			separator: Some(String::from(",")),
			col_x: Some(0),
			col_y: Some(1),
			skip_lines: Some(0),
		};
		let geo_file = GeoFile::load(&filename, options)?;
		let n = geo_file.data.len();

		assert_eq!(n, 1719789);

		assert_eq!(
			from_utf8(&geo_file.data[0..54])?,
			"11.39979,52.47553\n11.80435,53.68146\n11.80666,53.68135\n"
		);

		assert_eq!(
			from_utf8(&geo_file.data[n - 50..n])?,
			"\n9.82855,48.18889\n9.8237,48.18951\n9.8251,48.19072\n"
		);

		let entries = geo_file.get_entries()?;
		assert_eq!(entries.len(), 100000);

		assert_eq!(
			entries[0],
			GeoNode::new_leaf(GeoBBox::new_point(11.39979, 52.47553), 0, 17)
		);

		assert_eq!(
			entries[entries.len() - 1],
			GeoNode::new_leaf(GeoBBox::new_point(9.8251, 48.19072), 1719773, 15)
		);

		Ok(())
	}

	// Testing loading a brotli compressed file
	#[test]
	fn geo_file_load_geojsonl_brotli() -> Result<(), Box<dyn Error>> {
		let filename = PathBuf::from("testdata/polygons.geojsonl.br");
		let options = GeoFileOptions {
			separator: None,
			col_x: None,
			col_y: None,
			skip_lines: None,
		};
		let geo_file = GeoFile::load(&filename, options)?;
		let n = geo_file.data.len();

		assert_eq!(n, 2902443);

		assert_eq!(
			from_utf8(&geo_file.data[0..257])?,
			"{\"type\":\"Feature\",\"properties\":{\"land\":\"BW\",\"klasse\":\"Historische Siedlung\",\"name\":\"Römische Niederlassung\",\"name_kurz\":\"Röm. Niederlass.\",\"layerName\":\"Besondere_Flaeche\"},\"geometry\":{\"type\":\"Polygon\",\"coordinates\":[[[8.70915412902832,47.936775561951805],"
		);

		assert_eq!(
			from_utf8(&geo_file.data[n - 41..n])?,
			",[13.348388671875,52.52004009949795]]]}}\n"
		);

		let entries = geo_file.get_entries()?;
		assert_eq!(entries.len(), 3578);

		assert_eq!(
			entries[0],
			GeoNode::new_leaf(GeoBBox::new(8.709154, 8.710508, 47.935436, 47.936913), 0, 566)
		);

		assert_eq!(
			entries[entries.len() - 1],
			GeoNode::new_leaf(GeoBBox::new(13.348389, 13.359375, 52.519386, 52.520912), 2901065, 1377)
		);

		Ok(())
	}
}
