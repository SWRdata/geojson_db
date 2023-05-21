"use strict";

import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const { geofileOpen, geofileFind } = require('./index.node');

export default class Geofile {
	constructor(filename, memory_size = 64 * 1024 * 1024) {
		this.me = geofileOpen(filename, memory_size);
	}

	find(bbox) {
		if ((!Array.isArray(bbox)) || (bbox.length !== 4)) throw Error('argument "bbox" must be an Array of 4 numbers')
		return geofileFind.call(this.me, bbox);
	}
}
