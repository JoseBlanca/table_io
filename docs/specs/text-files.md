# table_io: a text file read into cells

Written on 2 October 2026, before any code, from `docs/architecture.md`,
section 3, "The cells, by the rules of the format", which the owner
approved that day. The rules are popnei_web's for a CSV and a TSV, from
its `docs/specs/worker/individuals.md`, "The bytes and the encoding",
"The separator", "The rows and the cells" and "The decimal mark and the
numbers", approved by the owner from 25 September 2026; they become
table_io's here, written in Rust where popnei_web wrote them in
TypeScript. A **text file** is a CSV or a TSV,
or any file the import does not find to be a workbook
(`specs/import.md`, "The format"). This spec gives the module `csv` of
the library crate, behind the feature `csv`: from the bytes of a text
file to its rows of cells, each with its line, and the encoding, the
separator and the decimal mark it was read with. What the rows then
become, the header, the missing values, the first column, is
`specs/import.md`'s; the export of a CSV is `specs/export.md`'s.

## What it does

A user imports the file Excel in Spanish saves as "CSV (delimitado por
comas)", which is in Windows-1252 with `;` between its cells and a comma
in its decimals, and sees its table, `España` and `1,75`, with a line
beside it, "Read as Windows-1252, separator `;`, decimal comma", in the
application's words. What goes wrong because of this module is seen
there:

- a wrong separator gives a table one column wide, or a refusal of a line
  whose cells do not match the header;
- a wrong encoding shows `EspaÃ±a` for `España`, and a name with an
  accent no longer matches the same name elsewhere;
- a wrong decimal mark makes a column of heights text, where it should
  be float;
- a quote the file never closes, which an editor shows as nothing wrong,
  would take the rest of the file into one cell.

Each of the three, the encoding, the separator and the decimal mark, is
found unless the caller sets it, and is reported as used, so that a user
who sees a wrong guess sets it and imports again.

### The bytes and the encoding

The caller gives the bytes of the file and, for the encoding, nothing or
UTF-8 or Windows-1252. The import has checked the size of the bytes
against the caller's limit before (`specs/import.md`).

1. **UTF-16.** A file that starts with the mark of UTF-16, the bytes
   `FF FE` or `FE FF`, which is what Excel writes for "Unicode Text", tab
   separated, is decoded as UTF-16, little endian after `FF FE` and big
   endian after `FE FF`, the mark removed, whatever encoding the caller
   set, since the mark says the encoding for certain. The encoding used is
   reported as UTF-16. A file whose bytes after the mark are odd in
   number, or whose last two bytes are the first half of a character
   written in four, a high surrogate, ends in the middle of a character
   and is refused as **cut short**, with no line: such a file was most
   often cut on its way, and a table read from it would lack its last
   rows without a word. A half of a character anywhere else, a surrogate
   with no partner, is decoded as U+FFFD, the replacement character,
   shown as �.
2. **Not text.** Any other file with a byte 0 anywhere in it, the whole
   file being looked at, its mark of UTF-8 included, is refused as **not text**: a gzipped VCF, a
   `.nei` file or another binary file picked by mistake. A zip, an xlsx
   among them, does not reach here (`specs/import.md`).
3. **The mark of UTF-8.** The three bytes `EF BB BF` at the start, the
   byte order mark of UTF-8 that Excel writes in "CSV UTF-8", are
   removed, whatever the encoding. With no encoding set, the mark decides
   UTF-8, as the mark of UTF-16 decides UTF-16: the file is decoded as
   with UTF-8 set, below, a bad byte in it becoming U+FFFD, as the owner
   decided on 25 September 2026 for popnei_web. Read as Windows-1252, as a
   file without the mark would be, every accented letter of a "CSV UTF-8"
   with one damaged byte would come out wrong.
4. **UTF-8 set**: the bytes are decoded as UTF-8, each sequence that is
   not UTF-8 becoming one U+FFFD in the way of the Encoding Standard of
   the web, which is what a browser's `TextDecoder` does and what Rust's
   `String::from_utf8_lossy` does: the bytes `F0 9F 98` followed by `b`,
   the first three of an emoji cut short, are one U+FFFD and then `b`
   (tried with Rust 1.98.0 on 2 October 2026).
   **Windows-1252 set**: each byte is one character, by the table below,
   so a decoding in Windows-1252 never fails and never gives U+FFFD.
   **No encoding set and no mark**: UTF-8 when the bytes are valid UTF-8,
   `std::str::from_utf8` succeeding, and Windows-1252 otherwise, which is
   what Excel on Windows writes for "CSV" in Spanish and the other
   languages of Western Europe. A file of ASCII alone is the same text in
   both and is reported as UTF-8. A file in Windows-1252 is taken for
   UTF-8 only when its accented letters happen to form valid UTF-8, which
   needs pairs such as `Ã` followed by `±` wherever an accent is, and
   does not happen in a real text. No file is refused for its encoding,
   as the owner decided on 24 September 2026.
5. **The line of the first character not decoded.** The line of the
   first U+FFFD of the text, counted as the lines are counted below, is
   reported with the encoding, and none when the text has none. It comes
   from a bad byte of UTF-8, set or decided by the mark, and from half of
   a character of UTF-16 in the middle of the file; Windows-1252 gives
   none. A U+FFFD that the file holds itself, written by a program that
   had already lost a character, is taken the same way, since it too
   stands where a character was lost.
6. Every U+FEFF at the start of the text that is left, which a file can
   hold after its mark when it was saved twice with one, is removed;
   popnei_web removes up to three, by the bytes, by `TextDecoder` and by
   its reader, which only a file of four marks tells apart.

Windows-1252 gives each byte the character of Unicode of the same
number, U+0000 to U+00FF, except the 32 bytes from `80` to `9F`, which
are these, by the index of the
Encoding Standard, `index-windows-1252`; the five that Windows-1252 left
without a character are the control of Unicode of the same number, as the
standard and a browser's `TextDecoder` have them:

| byte | character | byte | character | byte | character | byte | character |
|---|---|---|---|---|---|---|---|
| 80 | U+20AC € | 88 | U+02C6 ˆ | 90 | U+0090 | 98 | U+02DC ˜ |
| 81 | U+0081 | 89 | U+2030 ‰ | 91 | U+2018 ‘ | 99 | U+2122 ™ |
| 82 | U+201A ‚ | 8A | U+0160 Š | 92 | U+2019 ’ | 9A | U+0161 š |
| 83 | U+0192 ƒ | 8B | U+2039 ‹ | 93 | U+201C “ | 9B | U+203A › |
| 84 | U+201E „ | 8C | U+0152 Œ | 94 | U+201D ” | 9C | U+0153 œ |
| 85 | U+2026 … | 8D | U+008D | 95 | U+2022 • | 9D | U+009D |
| 86 | U+2020 † | 8E | U+017D Ž | 96 | U+2013 – | 9E | U+017E ž |
| 87 | U+2021 ‡ | 8F | U+008F | 97 | U+2014 — | 9F | U+0178 Ÿ |

### A variants file picked by mistake

A text whose first line that is not blank, its spaces and tabs at the
start left out, starts with `##fileformat=VCF`, as every VCF starts, or
with `#CHROM`, the header of its columns, is refused as a **variants
file**, whatever its separator, as the owner decided on 25 September 2026
for popnei_web. A blank line is one of spaces and tabs alone, or of
nothing. Read as a table, a VCF would most often be one column wide,
and the user of popnei_web would be told that every individual of the
variants is missing from it. Vavilov Explorer's users have no variants
file, and the check costs them nothing (its
`docs/table_io-needs.md`, section 3).

### The lines, the quotes and the cells

- **The lines** end with `\r\n`, `\n` or `\r`, the last being what old
  versions of Excel for Mac wrote. A line number counts every line of the
  file from 1, the header's included, the blank ones and the lines
  inside a quoted cell too, so that it is the number an editor shows. A
  text that ends with a line break has no line after it.
- **Quotes** are those of RFC 4180, the standard of CSV, as Excel writes
  them. A cell that starts with `"`, once the spaces before it are
  removed, goes on to the next `"` that is not doubled, and may hold the
  separator, a line break and `""`, which is one `"`. The spaces before
  it are those below, the tabs among them when the separator is not the
  tab, so `,\t"x, y"` is the cell `x, y`. What follows the
  closing quote, up to the separator, is kept as part of the cell, `"x"y`
  being `xy`. A `"` inside a cell that does not start with one is an
  ordinary character. A quote that is never closed takes the rest of the
  file into one cell, and the file is refused as an **unclosed quote**,
  with the line where the cell starts and the separator used, since a
  wrong separator is its likeliest cause.
- **Spaces**, U+0020, at the start and the end of a cell are removed, and
  so are tabs when the separator is not a tab; inside quotes they are
  kept. A file typed by hand as `ind_01, pop1` reads `pop1`. The spaces
  before a cell are removed first, so `A, "x, y"` is two cells, `A` and
  `x, y`.
- **Every cell is text**, as written, the spaces removed. A text file has
  no numbers; a number is read out of the text with the decimal mark by
  `specs/values.md`.

What this module gives the import is the rows of cells, each with the
line it starts on, as many cells as its line has: a row of another length
than the header is refused by the import, which knows the header
(`specs/import.md`, "The rows").

### The separator

With a separator set, the file is read with it. With none set, the module
counts the cells of each row with each of the three separators, a tab,
`;` and `,`, with the quotes above, and without making the cells. The
blank rows, whose cells are all empty, are skipped here as the import
skips them; the header is the first row that is not blank; and the
header is counted without the run of empty cells at its end whose columns
hold no value in any row, a value being any cell but an empty one, `NA`,
`-`, or no cell at all, as `specs/import.md`, "The header", drops it.

A separator **fits** the file when it gives the header two cells or more
and every row as many cells as the header. A row with more cells than the
header fits too when every cell it has past the whole header, its empty
cells at the end included, is empty; a separator with which a
quote is never closed does not fit, since whether a `"` opens a cell
depends on the separator before it. Of the separators that fit, the one
that gives the header the most cells is taken, and on a tie the tab, then
`;`, then `,`, since a tab is the least likely of the three to be inside
a value. When none fits, the one that gives the header the most cells is
taken, with the same order on a tie, and the import then refuses the row
that does not fit, naming its line; when every separator gives the
header one cell, the file is read with `,`, as a file of one column.

A Spanish Excel file, `Individuo;Población;Altura` over rows such as
`ind_001;España;1,75`, gives three cells to every line with `;`, and with
`,` one to the header and two to the rows, so `;` is taken.

### The decimal mark

With a decimal mark set, it is used. With none set, it is the point when
the separator is `,`. Otherwise it is found once the import has found the
header and checked the rows, as popnei_web's `readCsv` does: the module
counts, among the cells below the header and outside the first column, those that are numbers written
with a comma, a text that holds a comma and is a number with the comma by
`specs/values.md`, and those written with a point, the same with the
point; the comma is taken when the first count is larger, and the point
otherwise, a file of whole numbers among them. So a file of `1,75` and
`1,82` is read with a comma, and a file that mixes `1,75` and `1.82` is
read with the mark most of its cells use, and the others are text
(`specs/values.md`, "The type of a column").

## The refusals

In the order the module meets them, each a case of the import's refusal
(`specs/import.md`, "The refusals", which gives the order with the
import's own):

1. **Cut short**, a UTF-16 file that ends in the middle of a character.
   No data.
2. **Not text**, a byte 0 in a file that is not UTF-16. No data.
3. **A variants file**, a VCF. No data.
4. **An unclosed quote**: the line where the cell that is never closed
   starts, and the separator used.
A text file is bounded by its bytes alone, which the import checks
(`specs/import.md`, "The limit of cells").

## The Rust interface

The options of a text file, given with each import, and what the import
reports it used. In the library crate, at its root, also in a build
without the feature `csv`, since the import takes them whatever the file:

```rust
/// The encodings a caller can set; UTF-16 is decided by its mark alone.
pub enum Encoding { Utf8, Windows1252 }

/// The encodings a text file can be read with.
pub enum FoundEncoding { Utf8, Utf16, Windows1252 }

pub enum Separator { Tab, Semicolon, Comma }

/// What the caller sets for a text file; None is found from the file.
pub struct TextOptions {
    pub encoding: Option<Encoding>,
    pub separator: Option<Separator>,
    pub decimal: Option<DecimalMark>,
}

/// How a text file was read: each option as set or as found.
pub struct TextRead {
    pub encoding: FoundEncoding,
    pub separator: Separator,
    pub decimal: DecimalMark,
    /// The line of the first character that could not be decoded, from 1.
    pub undecoded_line: Option<u32>,
}
```

`DecimalMark` is `specs/values.md`'s. The functions of the module are
private: an application imports a file with `import_table`
(`specs/import.md`).

## The cases

- **A file with a title line above the header**, `Tabla 1;;` as Excel
  writes it: a header of three cells of which two are empty, which the
  import refuses with "column 2 has values but no name in the header",
  which leads the user to the first line. A title of one cell, `Tabla 1`,
  gives the header one cell with every separator, so the file is read
  with `,`: it is refused at the first row with a comma, a decimal comma
  among them, or else reads as one column.
- **The separator set to `,` on a file of `;`**: a file with a decimal
  comma is refused at its first row with one, as a row of the wrong
  length. A file with no comma reads as one column; nothing refuses it,
  since such a file is valid, and the user sees a table of one column.
- **A decimal comma set with the separator `,`**: the numbers can only
  be written in quotes, `"1,75"`, and they are read so.
- **The encoding set to UTF-8 on a Windows-1252 file**: every byte that
  is not valid is U+FFFD, `Espa�a`, and the line of the first is
  reported.
- **A file saved by Excel for Mac**: which encoding it writes for "CSV"
  has not been checked. If it is Mac Roman, which older versions wrote,
  an accented name is read as other characters by both encodings offered.
  A file of the owner's settles it (below).

## How it runs

An import of a text file holds the bytes, the text, and the rows of
cells, and then the table the import makes of them. The separators are
tried by counting, without making cells, so the cells are made once. For
a file of 20 MB that is 20 MB of bytes, about 20 MB of text, the cells, a
`String` of 24 bytes natively and 12 in the wasm, and its characters,
each, and the table: an
estimate, not measured. The speed of a file of 100,000 rows and 50
columns is measured as `objectives.md`, goal 7, says.

## How it is verified

With cargo test, natively, at `import_table`, the highest function at
which a text file can be seen, with bytes written into the test as
literals; the tables of the cases that `specs/import.md` gives, among
them every row of popnei_web's table at `readCsv`.

Over the bytes and the encoding:

- the Spanish Excel file in Windows-1252, `Individuo;Población;Altura`
  and four rows such as `ind_001;España;1,75`, with `ó` as the byte `F3`
  and `ñ` as `F1`, and `\r\n`: read as Windows-1252, `;`, comma; the
  header `Población`; the cells `España`; `Altura` a float;
- the same file as UTF-8 with its mark: read as UTF-8, the same table;
- the same file with the encoding set to UTF-8: the header
  `Poblaci�n`, the cells `Espa�a`, and the line of the first not decoded
  1;
- the same file as UTF-16 little endian, `FF FE` and two bytes a
  character, and as big endian, `FE FF`: read as UTF-16 and the same
  table, also with the encoding set to Windows-1252;
- the UTF-16 little endian file with one byte more, and with its last
  character the first half of one written in four, the bytes `3D D8`:
  cut short;
- a text with one byte 0 in its last line, and the bytes `00 01 02 03`:
  not text;
- the Spanish file as UTF-8 with its mark and the byte `FF` in its third
  line, no encoding set: read as UTF-8, the line not decoded 3, and a
  cell with �; the same file without the bad byte: none;
- each of the 256 bytes between `a` and `b` in the second line of
  `h\r\na?b\r\n`, read with the encoding set to Windows-1252 and the
  separator to `;`: the name of one individual, `a`, the character of the
  table above or of Unicode's first 256, and `b`; but for the byte `00`,
  not text, `0A` and `0D`, which end the line and give the names `a` and
  `b`, and `3B`, the separator, a ragged row at line 2. The table is what node 26.8.2's `new
  TextDecoder("windows-1252")` gave for the 32 bytes on the owner's Mac
  on 2 October 2026, every one the same.

Over the lines, the quotes, the separator and the decimal mark, the rows
of popnei_web's table at `readCsv` that are about them, with the same
texts and the separator, the decimal mark, the cells and the refusal they
give there, and in addition: `"x"y` the cell `xy`; `a,"b""c"` the cells
`a` and `b"c`; a text that ends with no line break and one that ends with
`\r\n` give the same table; the line of an unclosed quote that opens on
line 5 after a quoted cell over lines 2 and 3 is 5.

One property, at `import_table`, over tables made by a generator of the
tests with a fixed seed, no dependency added: a table written as a CSV,
with any of the three separators and of the three line endings, every
cell that holds the separator, a quote, a line break, or a space or a
tab at its ends in quotes, reads back as itself with that separator set,
and with none set when the header has two columns or more and no cell
holds any of the three separators. The tables made are those the import
gives back unchanged: names of the header distinct and not empty; names
of the first column distinct, not empty, and not starting with `#CHROM`
or `##fileformat=VCF`; other cells missing, written empty, or texts that
are not empty, `NA` or `-`; no row whose cells are all empty. The export of `specs/export.md` is checked the same way.

Made by the owner, in `tests/data/`, each read with no option set and
asserted against what the owner says Excel shows; the tests wait for the
files, marked `#[ignore]`:

- `excel_es.csv`, saved by Excel in Spanish on Windows as "CSV
  (delimitado por comas)", with an accent in a header and in a value and
  a decimal: Windows-1252, `;`, comma;
- `excel_es_utf8.csv`, the same sheet saved as "CSV UTF-8": UTF-8, `;`,
  comma;
- `excel_mac.csv`, the same sheet saved as "CSV" by Excel for Mac: the
  encoding it writes, which settles the case above;
- `excel_unicode.txt`, saved as "Unicode Text": UTF-16, tab;
- `libreoffice.csv`, saved by LibreOffice with its defaults: what it
  writes.

## Open points

None. The rules are popnei_web's, approved by the owner. Two changes
to them come from the other specs: the format is found from the bytes and
not the name (`specs/import.md`), and a boolean is written as text
`TRUE` and not `true` (`specs/values.md`).

## Not in this spec

- The header, the blank rows, the missing values, the first column, a
  row of the wrong length, and the order of all the refusals:
  `specs/import.md`.
- What a text means, a number, a whole number, a boolean, and the type of
  a column: `specs/values.md`.
- The export of a CSV: `specs/export.md`.
- The size of a file: the caller's limit, checked by the import.
