mod geo_file;

use geo_file::*;
use neon::{
	context::Context,
	prelude::{FunctionContext, ModuleContext},
	result::{JsResult, NeonResult},
	types::{JsArray, JsBoolean, JsBox, JsNumber, JsString},
};
use std::path::PathBuf;

impl GeoFile {
	pub fn js_open(mut cx: FunctionContext) -> JsResult<JsBox<GeoFile>> {
		let filename = PathBuf::from(cx.argument::<JsString>(0)?.value(&mut cx));
		let in_memory = cx.argument::<JsBoolean>(0).unwrap_or(cx.boolean(false)).value(&mut cx);

		let geo_file = GeoFile::open(&filename, in_memory);
		return Ok(cx.boxed(geo_file));
	}
	pub fn js_find(mut cx: FunctionContext) -> JsResult<JsString> {
		let geo_file = cx.this().downcast_or_throw::<JsBox<GeoFile>, _>(&mut cx)?;

		let bbox = cx.argument::<JsArray>(0)?.to_vec(&mut cx)?;
		let bbox: Vec<f32> = bbox
			.iter()
			.map(|v| v.downcast_or_throw::<JsNumber, _>(&mut cx).unwrap().value(&mut cx) as f32)
			.collect();

		let bbox = GeoBBox::new(bbox[0], bbox[2], bbox[1], bbox[3]);

		let json = geo_file.find(&bbox);
		return Ok(cx.string(json));
	}
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	cx.export_function("geofileOpen", GeoFile::js_open)?;
	cx.export_function("geofileFind", GeoFile::js_find)?;
	Ok(())
}
