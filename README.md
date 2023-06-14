# GeoJSON DB

GeoJSON DB is a high performance npm package designed to facilitate fast spatial queries on GeoJSON files.
It currently supports `.geojsonl` (line-limited GeoJSON features), `.csv` and `.tsv` files. The files can be compressed with `.br`, `.gz` or uncompressed.
Due to the in-memory architecture, make sure you have enough memory to load the full uncompressed file.

## Example Usage

```javascript
import Geofile from 'geojson_db';

let file = new Geofile('features.geojsonl.gz');

let bbox = [7, 50, 8, 51];
for (let feature of file.find(bbox)) {
   feature = JSON.parse(feature);
   console.log(feature);
}
```

You can also define options:

```javascript
let file = new Geofile('features.geojsonl.gz', {
   separator: ';', // field seperator for CSV / TSV files - default: "," / "\t"
   colX: 3, // column index of x values - default: 0
   colY: 4, // column index of y values - default: 1
   skipLines: 1,  // number of lines to skip, e.g. header line - default: 0
});
```

## Installation 

GeoJSON DB requires a [supported version of Node and Rust](https://github.com/neon-bindings/neon#platform-support).

To install the project using npm, navigate to the project directory, and run:

```sh
$ npm install
```
This command installs the project and its dependencies and also initiates the build.

## Build

To only execute the build process for an already installed project, run:

```sh
$ npm run build
```
This command uses the [cargo-cp-artifact](https://github.com/neon-bindings/cargo-cp-artifact) utility to perform the Rust build and subsequently copies the built library into `./index.node`.

## Available Scripts 

In the project directory, you can run:

* `npm install` - Installs the project, including running `npm run build`.
* `npm build` - Builds the Node addon (`index.node`) from source.

### Cargo build arguments

You can pass additional [`cargo build`](https://doc.rust-lang.org/cargo/commands/cargo-build.html) arguments to `npm build` and `npm build-*` commands. For example, to enable a [cargo feature](https://doc.rust-lang.org/cargo/reference/features.html):

```sh
npm run build -- --feature=beetle
```

* `npm build-debug` - Alias for `npm build`.
* `npm build-release` - Equivalent to `npm build` but builds the module with the [`release`](https://doc.rust-lang.org/cargo/reference/profiles.html#release) profile. Although release builds compile slower, they run faster.
* `npm test` - Executes the unit tests using `cargo test`. To learn more about [adding tests to your Rust code](https://doc.rust-lang.org/book/ch11-01-writing-tests.html), refer to the [Rust book](https://doc.rust-lang.org/book/).

## Project Structure

The project directory is structured as follows:

```
geojson_db/
├── Cargo.toml
├── README.md
├── index.node
├── package.json
├── src/
|   └── lib.rs
└── target/
```

## Documentation and Additional Learning Resources

* To learn more about Neon, visit the [Neon documentation](https://neon-bindings.com).
* To dive deeper into Rust, access the [Rust documentation](https://www.rust-lang.org).
* To explore Node further, see the [Node documentation](https://nodejs.org).