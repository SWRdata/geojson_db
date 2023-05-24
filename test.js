"use strict";

import { readFileSync, writeFileSync } from 'node:fs';
import Geofile from './index.js';
console.log(Geofile);

console.time('open');
let file = new Geofile('./data/adresse.csv')
//let file = new Geofile('./data/points.geojsonl')
console.timeEnd('open');

console.log(file);

//let bbox = [13.324571, 52.513766, 13.345771, 52.524473];
//let bbox = [13.3, 52.5, 13.4, 52.6];
//let bbox = [-180, -90, 180, 90];
let bbox = [7, 49, 8, 50];

let count = 0;
console.time('iterate');
for (let entry of file.find(bbox)) count++;
console.timeEnd('iterate');
//console.log(count);

/*
console.time('manual');
let data = readFileSync('data/points.geojsonl', 'utf8');
data = data.split('\n').filter(l => l.length > 0).map(l => JSON.parse(l));
console.timeEnd('manual');
console.log(data.length);
*/


//writeFileSync('test.geojson', JSON.stringify({ type: 'FeatureCollection', features: Array.from(file.find(bbox)) }));



