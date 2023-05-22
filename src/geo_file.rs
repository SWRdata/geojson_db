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
