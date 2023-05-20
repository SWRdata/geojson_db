"use strict";

import { geofileOpen, geofileFind } from './index.node';

class Geofile {
	constructor(filename, memory_size = 64*1024*1024) {
		this.me = geofileOpen(filename, memory_size);
	}

	find(bbox) {
		return geofileFind.call(this.me);
	}
}

module.exports = Geofile;
