# table_io

A Rust library that imports a table from a CSV, a TSV or the first
visible sheet of an xlsx file as typed columns, integer, float, boolean
and text, and exports typed columns as a CSV or an xlsx, by the rules
both of its users share. Vavilov Explorer, a desktop application, uses
the library natively; popnei_web, the web applications of the population
genetics library [popnei](https://github.com/JoseBlanca/popnei), uses it
as a wasm package. An xlsx is read by
[calamine](https://crates.io/crates/calamine) and written by
[rust_xlsxwriter](https://crates.io/crates/rust_xlsxwriter); a CSV is read
and written by table_io's own code. Until 2 October 2026 it was xlsx_rs,
which read an xlsx into its cells for popnei_web alone.

What it is for is in `docs/objectives.md`, how it is built and released in
`docs/architecture.md`, and what each module does in its spec under
`docs/specs/`.
