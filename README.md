# xlsx_rs

A small Rust library that reads the first visible sheet of an xlsx file
into its cells, compiled to WebAssembly and released as a wasm package for
popnei_web, the web applications of the population genetics library
[popnei](https://github.com/JoseBlanca/popnei). The reading is done by
[calamine](https://crates.io/crates/calamine); xlsx_rs chooses the sheet,
gives each cell as a user of Excel sees it, a date as a date, a merged
value in every cell of its range, and refuses with a reason a file it
cannot read.

What it is for is in `docs/objectives.md`, how it is built and released in
`docs/architecture.md`, and what it does, cell by cell, in
`docs/specs/read.md`. There is no code yet.
