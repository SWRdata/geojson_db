"use strict";

const { geofileOpen, geofileFind } = require("./index.node");

class Geofile {
	constructor(filename) {
		this.me = geofileOpen(filename);
	}

	find(bbox) {
		return geofileFind.call(this.me);
	}
}

module.exports = Geofile;
