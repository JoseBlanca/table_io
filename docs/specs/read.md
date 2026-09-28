# xlsx_rs: an xlsx read into cells

Written on 27 September 2026, for stage 4 of popnei_web's `docs/build-order.md`, the
Individuals step and the PCA, and revised the same day to agree with the
specs written beside it, and after two reviews of its claims against the
source of calamine 0.36.1 and two crates of trial; and when the specs
of stage 4 were made to agree, the last column of a sheet too large
given in Excel's letters, "XFD"; and with the owner's answer of 27
September 2026 to Open 2, the date whose time the format hides.
Revised on 28 September 2026, when the owner decided that the reader of
xlsx is a project of its own, **xlsx_rs**, with the conventions and the
skills of popnei (popnei_web's `docs/architecture.md`, section 6, "The files wasm,
the package of xlsx_rs"). The owner decided the same day that its
repository is `github.com/JoseBlanca/xlsx_rs`, under the owner's
account and public, as popnei's is; that its releases are made by hand,
as popnei's are, and later by a workflow shared with popnei; and
approved calamine 0.36.1 as its dependency, and rust_xlsxwriter 0.99.1
for its tests (popnei_web's `docs/architecture.md`, section 13, points 11 to 13).
Revised again on 28 September 2026 for three more answers of the
owner: an error `#N/A` of Excel is a missing value for popnei_web, and,
by a later answer, so is every other error calamine knows, Open 1,
which popnei_web's reader makes of the text xlsx_rs gives; and the
Individuals step says that the first sheet was read without naming it.
This is xlsx_rs's first spec. It was written in popnei_web, as
`docs/specs/worker/files.md` there, because the repository of xlsx_rs did
not exist yet, and it was approved by the owner on 28 September 2026 with
the specs of popnei_web's stage 4; it moved here, whole, on 28 September
2026, changed only in its paths and in what it says of popnei_web. The
same day the owner added to "How it is verified" the test that no file
makes it panic and the declarations kept in git, and decided that the
test of the package reads a file written by the tests until the owner's
own exist. Revised again the same day, after the review of the code
built from "What xlsx_rs reads before calamine": its bounds count at
least what calamine could hold, in every part it could read, instead of
reading the parts as calamine reads them. The code built from it is in
`crates/`.

This spec gives the Rust library that reads the first sheet of an xlsx,
the file Excel saves by default, into its cells, and the wasm package
xlsx_rs builds of it and releases, which popnei_web installs from the
URL of a release and calls the **files wasm**: the second wasm module of
its light worker, the thread of the tab that reads the files of the user
and holds no popnei, which downloads the package the first time the user
loads an xlsx. In popnei_web's stage 4 it only reads; writing an xlsx,
and the zip of the report, come with stage 6 of popnei_web. A document
of popnei_web, its first user, is named with "popnei_web's" before its
path; a path with nothing before it is xlsx_rs's.

What stays in popnei_web, and was moved out of this spec on 28 September
2026 into popnei_web's `docs/specs/worker/individuals.md`, "The package of xlsx_rs,
loaded on first need", the spec of the light worker's reading of the
individuals file, which already held the function it is given to: the
light worker's import of the package on first need, the refusal
`xlsxReaderNotLoaded` when it cannot be downloaded, and what the user
sees meanwhile; `readXlsxCells` of popnei_web's `src/worker/xlsxCells.ts`, which
calls the package's `readXlsx` and makes of what it returns the cells,
or a refusal of `IndividualsFileError`, the union of every way a file of
the individuals is refused; and their tests, under Vitest and in the
flow of the Individuals step in the browser. What the cells become, the
table of the project with the types of its columns, is that spec's too;
the messages that carry a read are in popnei_web's
`docs/specs/worker/messages.md`.

calamine, the Rust library that reads the file, gives each cell, as its
reader of the cells one by one gives them, as one of the values of its
type `DataRef`, which is what xlsx_rs matches on: empty, a number, a
text of the workbook's table of texts, which an xlsx keeps once for all
the cells that hold it (`SharedString`), a text the cell holds itself
(`String`), a boolean, a date or a time, a date written in ISO 8601, or
an error such as `#N/A`. `DataRef` also has a whole number and a
duration in ISO 8601, which calamine's reader of an xlsx never gives,
and which xlsx_rs turns into a number and a text all the same: a whole
number is the number up to 2^53 either side of 0, which a JavaScript
number holds exactly, and its digits as text beyond, so that no cell
holds a number other than the file's; and the one error calamine knows
that its reader of an xlsx never gives, `GettingData`, is the text
`#GETTING_DATA`, as a file writes it. calamine's
other type of value, `Data`, is what it gives when it reads a whole
sheet at once, which xlsx_rs does not do (below). xlsx_rs turns each
value into one of the four kinds of cell the table holds (`Cell`,
popnei_web's `docs/specs/worker/protocol.md`): empty, text, number or boolean. The
rules below are read from the source of calamine 0.36.1, the version
popnei_web's `docs/technology.md` measured, and tried on 27 September 2026 in two
crates of trial built from them on the owner's Mac, with Rust 1.98.0.

## What it does

A user loads `pops.xlsx` in the Individuals step and sees the table of
its first sheet, as they see it in Excel. What goes wrong because of the
reader is seen there, or is not seen at all:

- a date read as a number shows `45425` where Excel shows `13/05/2024`;
- an identifier `001` that Excel shows with a format of three digits is
  stored as the number 1, and the individual `1` matches no individual of
  the variants;
- a value merged over ten rows in Excel, the population of ten
  individuals, would belong to the first of them only;
- a sheet with one cell of an error calamine does not know is not read
  at all, since calamine refuses it (below, "The refusals").

### The sheet read

The first sheet is the first worksheet in the order of the tabs that is
not hidden, the leftmost tab the user sees. It may not be the sheet
Excel opens the file on, which is the one that was active when the file
was saved; xlsx_rs does not read that one, since calamine 0.36.1 does
not give it, and the Individuals step says that the first sheet was read,
in a fixed line that names no sheet, as the owner decided on 28
September 2026 (popnei_web's `docs/specs/worker/individuals.md`, **Open 1**). A hidden first sheet,
which holds the lists of a form or old data, is passed over; the option
not taken, the first sheet whether hidden or not, would read a table the
user does not see. A chart sheet is not a worksheet and is
passed over too. Every row and column of the sheet is read, those hidden
or filtered out included, since a hidden row still holds an individual.

xlsx_rs reads the cells one by one, in the order of the file, with
calamine's `Xlsx::worksheet_cells_reader`, and not with its
`worksheet_range`, which builds the whole rectangle of the sheet in
memory before xlsx_rs can look at its size. The cells it keeps are those with a
value. The rectangle it gives runs from the first row and the first
column that hold a value to the last ones, so a sheet whose table starts
at C3 gives the table and not two empty rows and columns before it; the
number of its first row, as Excel numbers them from 1, and of its first
column, A being 1, cross with the cells, so that a refusal names a row
and a column as the user finds them in Excel
(popnei_web's `docs/specs/worker/individuals.md`, "The xlsx"). A cell that is missing
from the file, as a sparse row leaves them, is empty.

A sheet whose rectangle has more than 2,000,000 cells, rows times
columns, is refused, as `sheetTooLarge`. xlsx_rs keeps the rectangle
of the values it has read so far as it goes, and refuses the sheet at
the first cell that makes it larger than the limit, without reading
further: a note in column XFD, the last of Excel, at row 200, over a
table at A1, is refused when the note is read, and not after the rest of
the sheet. So xlsx_rs never holds more than 2,000,000 cells with a
value, and the refusal names the last row and the last column the
rectangle had reached, where the user looks for the values outside the
table. The limit is `MAX_SHEET_CELLS` of
popnei_web's `docs/specs/worker/individuals.md`, which the light worker gives
xlsx_rs with each read, so that the number is written in one place. A CSV
of 20 MB, the limit of a file of the individuals, holds about 1,800,000
cells of ten characters, each with its separator, eleven bytes: the
limit of an xlsx lets it hold a table as large as a CSV can, and a
larger rectangle is a stray value far from the table, or not a table of
individuals. It is an estimate: no sheet of that size has been read.

### Each cell

What each value of calamine's `DataRef` becomes. A text crosses as a
JavaScript string, a number as a JavaScript number, a boolean as a
boolean, an empty cell as `null`.

| in the file | calamine gives | the cell |
|---|---|---|
| nothing, or a text with no character | `Empty`, `SharedString("")`, `String("")` | empty |
| a text | `SharedString`; `String` for the text a formula saved, for a text written in the cell itself, which programs other than Excel may write, and for a value of no type that is not a number | the text as it is, spaces at its ends and line breaks in it kept when the file marks the text to be kept so, `xml:space="preserve"`, as Excel and rust_xlsxwriter do; calamine removes the spaces, tabs and line breaks at the ends of a text the file does not mark, as its `read_string_with_bufs` does, so a text of spaces alone is then empty; the reader removes the spaces (popnei_web's `docs/specs/worker/individuals.md`) |
| text with several fonts in it | `SharedString`, its parts joined | the text |
| a number, of any format that is not a date: `1,75`, `50 %`, `001` | `Float` | the number, `1.75`, `0.5`, `1` |
| a whole number | `Float`: an xlsx stores every number alike, and calamine gives `Int` only for other formats | the number |
| a number that is not finite, which Excel does not write | `Float` | the text JavaScript writes for it, `NaN`, `Infinity`, `-Infinity` |
| `TRUE` or `FALSE`, `VERDADERO` or `FALSO` in Spanish | `Bool` | the boolean |
| a date: a number with a format of date or time, of 1 day or more, whose time, rounded to the millisecond, is 0 | `DateTime` whose `ExcelDateTime` is not a duration | `2024-05-13` |
| a date and a time: the same, with a time that is not 0, whether the format shows it or not (**Open 2**, below, decided by the owner) | `DateTime` | `2024-05-13 12:00:00`, and `2024-05-13 12:00:00.250` when its milliseconds are not 0 |
| a time alone: a number with a format of date or time that rounds to less than 1 day and not below 0, a number a little below 0 that rounds to 0 milliseconds among them | `DateTime` | `14:30:00` |
| a format of date on a number below 0, or on a day after 31 December 9999, which Excel shows as `#######` | `DateTime`, with the parts of a wrong date | the number |
| a duration, a format such as `[h]:mm:ss` | `DateTime` whose `ExcelDateTime` is a duration | hours, minutes and seconds, the hours not wrapped at 24 nor written with a leading 0: 1.5 days is `36:00:00`, and a negative one `-0:30:00` |
| a date written as ISO 8601 text, a cell of the type `d`, which other programs than Excel may write | `DateTimeIso` | the text as it is |
| an error: `#N/A`, `#DIV/0!`, `#NAME?`, `#NULL!`, `#NUM!`, `#REF!`, `#VALUE!` | `Error` | the text of the error as Excel writes it in English; popnei_web's reader takes a text equal to one of the seven as missing, and so also the same text typed in a cell, which it cannot tell from an error (**Open 1**, below, decided by the owner) |
| a formula | the value saved with it | the cell of that value, by the rows above |

A date becomes text in ISO 8601, year, month, day, and not the number
Excel stores, 45425 for 13 May 2024, which a user would not recognise,
nor the date as Excel shows it, which depends on the language of the
computer. It is text in the table, so a column of dates is categorical,
and a year is a number and not a date.

calamine gives no format of a cell, only that the format is one of a
date or a time, or one of a duration, the `ExcelDateTime` that `DateTime`
holds. So xlsx_rs cannot tell a date shown with its time from one
shown without it, and decides by the number: a number with no time, as a
date typed by hand has, gives the date alone, and a number with a time
gives both, whatever Excel shows, as the owner decided on 27 September
2026 (**Open 2**, below).

calamine keeps the year of its parts in 16 bits, so a day far past
9999, 23,981,957 among them, comes back as a plausible year below 10000,
2024 for that one; so xlsx_rs does not ask calamine for the parts of a
day past 2,958,466, the first day after 9999 in the 1900 system, and
gives the number for any later one, in both systems (found with work
package 3, 28 September 2026). A time that rounds to a whole day,
0.99999999999 with a format of time, is a day of 1 and gives the date
`1900-01-01`, or `1904-01-02`, where Excel shows `00:00:00`: calamine
does not give the format, and a number of 1 day or more is a date; a
column of times may then hold one date, which the user sees in the
table.

A time alone, a date and a time, and a duration are written with their
milliseconds, `.250`, when the rounded number has any, and without them
when it has none; decided with work package 3 on 28 September 2026, since
a time alone and a duration of the table above had not said.

xlsx_rs first rounds the number to a whole number of milliseconds, and
splits that into its days and the milliseconds of its last day, with the
arithmetic of whole numbers. 45425.9999999999, a hundredth of a
millisecond before midnight, is then 14 May 2024 at 0:00, where
calamine's parts of the number as it is give 13 May at hour 24, as the
second trial saw. The parts of the day come from calamine, an
`ExcelDateTime::new` of the days in the date system of the workbook,
which xlsx_rs reads itself (below, "What xlsx_rs reads before
calamine"), and its `to_ymd_hms_milli`, with no library of
dates: xlsx_rs takes calamine without its feature `chrono`. The date
system is the 1900 one or the 1904 one of old Excel for Mac, so the same
date shown in Excel gives the same text in both, and calamine reproduces
Excel's 29 February 1900, a day that did not exist, as Excel does: 60
gives `1900-02-29` in the trial.

calamine works the parts out only from the first day of the system to
the end of 9999. It turns the days into a whole number with no sign, so
a number below 0 gives 31 December 1899 in the 1900 system and 1 January
1904 in the other; and a day after 9999 gives the year 10000 or later,
from 2,958,466 in the 1900 system and from 2,957,004 in the 1904 one, as
the second trial saw. So xlsx_rs gives the number itself for a number
below 0 and for a year after 9999 in the parts, a rule that holds in both
systems. A time alone, a number that rounds to less than a day, is also
a date for calamine, of 31 December 1899 or 1 January 1904; xlsx_rs
writes the time only. A duration is written from its whole number of
milliseconds, since calamine gives its parts as a date of January 1900.

A number whose format calamine takes for a date, one with a `d`, `m`,
`y`, `h` or `s` outside quotes, such as `0.0m`, comes as a date too. It
was seen in none of the files tried; a user sees it in the table, as a
date where Excel shows a number.

A formula gives the value Excel saved with the file, which Excel,
LibreOffice and Google Sheets always save. A file written by a program
that calculates no formula has none: rust_xlsxwriter, the library of the
report, saves 0 unless it is given the value, which the trial saw, and
another program may save nothing, which reads as an empty cell. Such a
file is read with those values, and the user sees them in the table.

### Merged cells

A range of merged cells holds its value in its first cell, the one at its
top left, and the others are empty in the file; Excel shows the value
over the whole range. So xlsx_rs gives every cell of a merged range the
value of its first cell, within the rectangle of the values, with
calamine's `Xlsx::merge_cells_by_sheet_name`: a population merged over
ten rows is the population of the ten individuals, which is what the user
sees. The option not taken, the value in the first cell alone, which
calamine, pandas and R's readxl give, would leave nine of the ten in no
population, told only by the warning of individuals with no population.
A name of a column merged over two columns gives two columns of one
name, which the reader refuses, "two columns are named Origen".

Every cell of the range inside the rectangle takes the value of its
first cell, or is empty when that cell has none, as when it lies outside
the rectangle, whatever the file holds in it: Excel and LibreOffice show
the first cell's value over the range, and LibreOffice can keep the
values of the other cells hidden in the file. Such a hidden value still
counts for the rectangle. A range the file writes from its last cell to
its first, `B4:B2`, which Excel does not write, is the same range,
`B2:B4`. Two ranges that overlap inside the rectangle, which Excel does
not let a user make, make the file unreadable, `ReadError::Unreadable` with xlsx_rs's message
"overlapping merged ranges": which value their shared cells took would
depend on the order of the ranges in the file; an overlap outside the
rectangle changes no cell and is let be. These three were decided
after the review of work package 2, on 28 September 2026.

### The refusals

A file xlsx_rs cannot read as an xlsx is refused with the kind of
`IndividualsFileError` that says why, so that the user is told what to do
with it; the kinds and their words are in popnei_web's
`docs/specs/worker/individuals.md`, "The refusals and their words". xlsx_rs says which it is by a code,
and the light worker makes the refusal of it. In the order it looks:

1. **An older file of Office**, whose bytes start with the mark of a
   compound file of Office, `D0 CF 11 E0 A1 B1 1A E1`: an xlsx saved with
   a password, which Excel encrypts inside such a file, is `encrypted`,
   found by xlsx_rs in the bytes of the file, which hold the name of its
   part `EncryptedPackage`, written as a compound file writes its names,
   in UTF-16 (little-endian); any other is a workbook of Excel 97–2003,
   an `.xls` whose name was changed, or another file of the old Office,
   and is `oldExcel`. An `.xls` whose name ends in `.xls` never reaches
   xlsx_rs: the Individuals step does not load it
   (popnei_web's `docs/specs/steps/individuals.md`).
   calamine is not asked, as this spec had it until the review of 28
   September 2026 found that its reader of compound files panics on one
   cut short, at 11,234 of the 40,960 lengths of an encrypted xlsx, which
   in the wasm ends the light worker. What the search costs: an `.xls`
   that holds the text `EncryptedPackage` in its cells, stored in UTF-16,
   is refused as `encrypted` and not as `oldExcel`; both are refused.
2. **Not a zip file**, whose bytes do not start with `PK` and the bytes 3
   and 4, as every xlsx does, since an xlsx is a zip of XML files:
   `notXlsx`. It is most often a CSV saved with the name `.xlsx`, which
   the words say how to mend. An empty file is one.
3. **A zip that calamine cannot open as a workbook, or whose sheet it
   cannot read**: a file cut short, damaged, or another format in a zip,
   a sheet of LibreOffice in its own format, `.ods`, or Excel's binary
   workbook, `.xlsb`, with the name `.xlsx`. The message of the zip
   crate, of the reader of XML or of calamine, whichever failed, goes into
   the refusal `files`, which says the file could not be read as a
   workbook and may be damaged, and the message is written to the
   console, where it helps the one who reports it: "invalid Zip archive:
   Could not find EOCD", the record that ends every zip, for the first
   500 bytes of an xlsx, since xlsx_rs opens the zip itself ("What
   xlsx_rs reads before calamine").
4. **No worksheet that is not hidden**, which Excel does not let a user
   save: `files`, with xlsx_rs's own message, "no visible worksheet",
   as `ReadError::Unreadable`.
5. **A cell with an error calamine does not know**: calamine 0.36.1 knows
   the seven errors of the table above, and refuses the whole sheet at
   any other, with `XlsxError::CellError`, whose text is the error.
   rust_xlsxwriter writes `#GETTING_DATA`, which the trial refused with
   "Unsupported cell error value '#GETTING_DATA'". xlsx_rs gives it as
   `cellError`, with the text of the error, so that the user is told
   which formula to mend; calamine does not say which cell, and neither
   can xlsx_rs, so the words say how to find the cells with an error in
   Excel. The newest versions of Excel have other errors, `#SPILL!` and
   `#CALC!` among them, but whether they reach calamine as such is not
   known: Excel may save them as `#VALUE!`, with the real error in a
   part of the file calamine does not read, and then the cell is the
   text `#VALUE!`, which popnei_web reads as missing, and nothing is
   refused. The owner's `spill.xlsx`, saved by Excel for Mac on 28
   September 2026 with `=SEQUENCE(3)` blocked by a value under it, is so:
   its cell is the text `#VALUE!`. `#SPILL!` does not reach calamine, and
   this refusal is left for an error Excel stores as itself that calamine
   does not know.
6. **A sheet too large**: `sheetTooLarge`, with the name of the sheet and
   the last row and column the rectangle had reached, above. It and a
   cell of point 5 come as the cells are read, so the one met first in
   the order of the file is the refusal.
7. **A sheet with no value**: `emptySheet`, with the name of the sheet,
   known once every cell is read. The likeliest cause is a table on the
   second sheet of a workbook whose first holds notes that were deleted,
   or a first sheet left empty; the words say that only the first sheet
   is read.

A sheet whose rows are all blank but its header, or whose values are
all spaces, is not refused here: xlsx_rs gives its cells, and the
reader refuses it as a CSV of the same rows, `empty`.

A panic of xlsx_rs, or of calamine inside it, is a trap of the wasm,
which ends the light worker (popnei_web's `.claude/skills/coding/worker.md`, "Errors
are values"); the lints of xlsx_rs, popnei's, deny what panics in its own
code, and calamine cannot be read line by line for it. A sheet so large in memory that the
wasm cannot grow, within 20 MB of zip, ends the same way. The read then
fails because its worker failed (popnei_web's `docs/specs/worker/messages.md`).

Two bounds keep a file written to be small in the zip and large in
memory from reaching that trap, both found by the review of work package
2 on 28 September 2026 and both `ReadError::Unreadable` with xlsx_rs's
message, since no program a user saves with writes such a file:

- **More cells written than the limit.** A file may write the same cell
  more than once; the rectangle then stays small while every copy is
  kept, 1.29 GB for a zip of 6.5 MB that wrote one cell 40,000,000 times
  in the review's trial. A read that is given more cells than
  `max_cells`, counted as written, is refused, "cells written more than
  once"; below that, a cell written again takes the last value, as
  calamine gives them.
- **More text than 200,000,000 bytes, `MAX_TEXT_BYTES`.** One text is
  copied into every cell that holds it, and a text of 100,000 characters
  merged over 10,000 cells took 1.15 GB for a zip of 5.7 KB. The bytes of
  the texts of the cells, merged ones counted in each cell, are summed
  as they are made, and past the bound the read is refused, "too much
  text". 200 MB of UTF-8 is 400 MB or more as JavaScript's strings, the
  largest a tab can be asked to hold; a table of individuals of 20 MB of
  CSV holds about 20 MB of text.

What xlsx_rs does not bound, since calamine reads it whole before
xlsx_rs sees it: the number of merged ranges, 16 bytes each, 650 MB for a
zip of 5.1 MB with 40,000,000 of them in the trial, and the workbook's
table of texts, for which calamine reserves room for as many texts as
the file says it holds, `uniqueCount`, before it reads them: a file of
6 KB saying 400,000,000 trapped the package under node in the review of
work package 4, and one saying 150,000,000 left the wasm holding 1.8 GB.
xlsx_rs reads both itself before calamine does, and bounds them, as the
next section says.

### What xlsx_rs reads before calamine

Decided on 28 September 2026, when the owner left the question to the
writer, after the reviews of the plan of the reading and the owner's own
files had found four things calamine 0.36.1 gets wrong or reads with no
bound. xlsx_rs opens the zip itself, with the `zip` crate, and reads
four things of its parts with the `quick-xml` crate, a reader of XML,
both at the versions calamine uses; then it gives the same bytes to
calamine, which reads the sheet as the sections above say. Each failure
is `ReadError::Unreadable` with xlsx_rs's message, which popnei_web shows
as a file that could not be read and may be damaged. Revised the same
day, after the review of the code built from it, as the next paragraphs
but one say.

An xlsx is a zip of parts, each an XML file: the workbook,
`workbook.xml`, which lists the sheets and holds the settings of the
file; a part of relationships beside it, which gives the part of each
sheet; the table of texts, `sharedStrings.xml`, which holds once each
text the cells hold; the styles, `styles.xml`, where the formats of the
numbers are; and a part for each sheet. calamine finds the folder of the
workbook from the relationship of type `officeDocument` in the package's
own relationships, `_rels/.rels`, and each other part by a name made of
that folder and the part's usual name, matched ignoring case and reading
`\` as `/` (`build_zip_path_cache` and `cached_zip_path` of its
`utils.rs`).

The first version of this section had xlsx_rs find each part as calamine
finds it and read it as calamine reads it, and the review of 28
September 2026 found that every small difference between the two
readings let a file past a bound: files of about 2.5 KB, written by
hand, trapped the package under node. Three differences did it:

- **An attribute split another way.** calamine reads the attributes of
  an element with a function of its own, `RawAttrIter` of its
  `attrs.rs`, which splits the text of a tag into names and values and
  which takes a form feed, the byte `0C`, for a space before
  a name and trims it after one; quick-xml's reader of attributes, which
  xlsx_rs used, keeps it in the name. With `\x0CuniqueCount=` calamine
  reserved room for the texts and xlsx_rs saw no count; with `\x0CType=`
  in `_rels/.rels` xlsx_rs found no workbook and checked nothing.
- **A part cut short.** xlsx_rs stopped at the end of a table of texts
  with no `</sst>` before it compared its count, after calamine had
  already reserved room for 400,000,000 texts.
- **The sheets listed by another rule.** calamine reads what is inside a
  defined name, the element `definedName`, as text, and reads the
  workbook up to the end tag named `workbook`; xlsx_rs listed a `sheet`
  inside a `definedName` and stopped when a stray end tag closed its
  count of open elements, and so counted the merged ranges of a part
  calamine did not read: 50 merged ranges were read with `max_cells` 10.

So the bounds no longer copy calamine's reading. Each counts, in every
part calamine could read for it and to the end of that part, at least
what calamine could hold; the attributes xlsx_rs reads are read by a
copy of calamine's `RawAttrIter`, under calamine's MIT license, with
calamine's copyright kept above it; and the one part xlsx_rs still has
to find as calamine finds it, the workbook, for its date system, is
refused when xlsx_rs cannot find it.

1. **Every part read to its end.** Each part of a zip carries a checksum
   of its bytes, which the zip crate checks when the part has been read
   to its end, and only then. calamine stops reading a sheet at its last
   cell, so a sheet damaged inside its compressed bytes was read with
   cells wrong or missing and no error: 145 of 4,072 copies of a file
   written by rust_xlsxwriter, with one byte of its sheet's compressed
   data changed, in the spec's review of 28 September 2026. xlsx_rs reads
   every part to its end, and any error of the zip crate refuses the file
   with the zip crate's message, "Invalid checksum" for those 145, all of
   them caught. So a file that is not a zip past its first bytes, the
   first 500 bytes of an xlsx among them, is refused with the zip crate's
   message and no longer calamine's. The bytes read so are counted, and
   past 1,000,000,000, `MAX_UNZIPPED_BYTES`, the file is refused, "the
   file unzips to more than 1,000,000,000 bytes": `individuals_10000.xlsx`
   unzips to 6.6 times its size, so a zip of 20 MB, popnei_web's limit,
   of the same kind is about 133 MB, and a zip can be written to unzip to
   many GB. The same reading counts the merged ranges of each part, point
   4, and xlsx_rs lets go of its zip before calamine opens the file, so
   that the list of the parts is not held twice: held twice, a zip of
   20 MB of 220,000 empty parts took 261 MB natively, where calamine
   alone takes 166 MB (review of 28 September 2026).
2. **The date system.** An xlsx says it counts its dates from 1904 in
   the attribute `date1904` of the element `workbookPr` of the workbook.
   Excel 365 also writes, inside the element `extLst` of the same part,
   an element of another namespace with the same name,
   `<x15:workbookPr chartTrackingRefBase="1"/>`, and calamine, which
   matches by the name alone, reads that one last and takes the file to
   count from 1900: the owner's `excel_1904.xlsx`, saved by Excel for Mac
   on 28 September 2026, gave 13 May 2024 as `2020-05-12`, four years and
   a day early, and so would every date of every workbook of the 1904
   system that Excel saves. xlsx_rs takes the date system from the last
   `workbookPr` that is a direct child of the root element of the
   workbook, whatever the namespace of either, so that a file Excel saves
   as "Strict Open XML", a choice of its Save As, is read the same: the
   namespace, a URL at the root of the part that says which vocabulary
   its elements are of, is another there;
   `date1904` `1` or `true` means 1904, as calamine reads it, the first
   attribute of that name. It does not use calamine's
   `Xlsx::has_1904_epoch`.
   The workbook is `workbook.xml` in the folder calamine's
   `read_package_relationships` finds in `_rels/.rels`, and xlsx_rs reads
   that part as it does: the elements of local name `Relationship` that
   start after the first start of `Relationships` and before the next
   end of an element of that local name; in each, the attributes `Type`
   and `Target` as calamine's macro `get_attrs!` reads them, with
   `RawAttrIter`, one found at each attribute of either name, a repeated
   one counted again and taking the place of the first, up to two found
   or to an attribute it cannot read, which refuses the file; and the
   target of the last relationship whose type ends in
   `/relationships/officeDocument` and that has a target, its entities
   read, up to its last `/`, with no `/` first. So
   `Type="…/officeDocument" Type="x" Target="…"` is a relationship of
   type `x` with no target, and one after `</Relationships>` is not read.
   A file whose `_rels/.rels` is missing or names no workbook so is
   refused, "the file names no workbook": calamine refuses it too, unless
   its reading and xlsx_rs's came apart, and then xlsx_rs could take the
   date system from another part than calamine's. A workbook named but
   missing gives the system of 1900, and calamine then finds no sheet. The
   workbook is read up to the end of its root element, found by counting
   the elements open, so a file whose end tags do not match its start
   tags, which no program a user saves with writes, may have its date
   system taken from another element; no bound depends on it.
3. **The parts calamine holds whole.** calamine reads four parts whole
   into memory when it opens the file, before the first cell: the
   workbook, with every sheet and every defined name it lists; the
   relationships; the styles; and the table of texts, for which it first
   reserves room for as many texts as the file says it holds, its
   attribute `uniqueCount`. In the review, a table of texts of 6 KB
   saying 400,000,000 trapped the package; one of 80,000,000 empty texts,
   1.9 MB zipped, took 1.93 GB natively; a workbook listing 20,000,000
   defined names, 3.2 MB zipped, 1.61 GB. xlsx_rs bounds every part whose
   name ends as one of these, `\` read as `/` and ignoring case, in any
   folder: the part calamine reads is always one of them, since its name
   is the folder calamine found followed by the usual name. So:
   - each part named `_rels/.rels`, or ending in `workbook.xml`,
     `workbook.xml.rels` or `styles.xml`, is refused past 50,000,000
     bytes unzipped, `MAX_SETTINGS_PART_BYTES`, "a part of the file is
     too large". One Excel saves is a few KB, but Excel lets a workbook
     gather tens of thousands of styles or of defined names no longer
     used, and such a part can pass 10 MB, the bound of the first version
     of this section, which the spec's review of 28 September 2026 found
     too close; no such file has been measured. calamine holds of the
     styles little more than a byte for each, and of a workbook of 50 MB
     of defined names about 175 MB natively, estimated from the 80 bytes
     of memory for each name of the 1.61 GB above and from the shortest
     name, `<definedName name="a"/>`, 23 bytes, 2,170,000 names in 50 MB;
   - each of these parts and each table of texts is refused when it is
     in an encoding other than UTF-8, "a part of the file is not in
     UTF-8": whenever quick-xml's reader of the part, reading it to its
     end or first error, would decode it with another, from a byte order
     mark at its start or from the `encoding` of its declaration
     `<?xml … ?>`. calamine decodes the texts of a part in the encoding it
     declares, and one declared `windows-1252` turns a byte `80` into
     `€`, 3 bytes of UTF-8, so every bound of this point, counted in the
     bytes of the part, would hold 3 times as much: in the spec's review,
     paths of sheets counted at 96,171,600 bytes took calamine 300 MB.
     Excel, LibreOffice and Google Sheets write UTF-8, and so do the
     twelve files of `tests/data/`;
   - calamine keeps, for each sheet the workbook lists, the path of its
     part, the folder of the workbook followed by the target of the
     sheet's relationship, and every sheet may name the same
     relationship: 2,000 sheets naming one target of 500 KB, a zip of
     8,586 bytes, took 1.0 GB natively in the spec's review. xlsx_rs
     counts, in each part ending in `workbook.xml`, the bytes `sheet`
     that come right after `<` or `:`, as every element `sheet` has them
     at the start of its name; and takes the longest tag, from `<` to
     `>`, that quick-xml reads in each part ending in `workbook.xml.rels`
     and in `_rels/.rels`, since a target and the folder calamine takes
     from them are inside one tag. When the largest count times the sum
     of the two longest tags passes 100,000,000 bytes,
     `MAX_SHEET_PATH_BYTES`, the file is refused, "the workbook lists too
     many sheets". A workbook of 1,000 sheets as Excel writes it counts
     1,001, with tags of about 150 bytes: 300 KB;
   - each part ending in `sharedStrings.xml`, a table of texts, is refused
     past 400,000,000 bytes unzipped, `MAX_TEXT_TABLE_BYTES`, twice
     `MAX_TEXT_BYTES`, "too much text";
   - in each table of texts, xlsx_rs reads the XML, with quick-xml
     configured as calamine's `xml_reader` configures its own, so that
     the two meet the same elements, to the end of the part or to its
     first error of XML. It counts every element whose local name, the
     name after any prefix, is `si`, each a text, those inside another
     `si` and those outside the root counted too, and refuses the file at
     the first past 10,000,000, `MAX_TEXTS`, "too many texts", where a
     table of individuals has at most as many texts as cells, 2,000,000.
     It reads the attributes `uniqueCount` of every element of local name
     `sst`, each up to the first attribute `RawAttrIter` cannot read, and
     refuses the file at one past 10,000,000, "too many texts", since
     calamine reserves that room as soon as it meets `<sst>`. calamine
     counts fewer: the `si` of the first `sst`, not those inside another,
     up to `</sst>`; and reserves room for the first `uniqueCount` of the
     first `sst`. A `uniqueCount` larger than the texts, or a table cut
     short, is otherwise let be: the first version of this section
     refused a `uniqueCount` larger than the texts, a file calamine
     reads, and the room calamine reserves for it is within the same
     bound. At its largest calamine's list of texts takes about 302 MB
     in the wasm: without a `uniqueCount` it grows by doubling to
     16,777,216 texts of 12 bytes, 201 MB, while the list of 100 MB it
     outgrew is still held (recomputed in the spec's review); with one,
     120 MB. An error of XML is not a refusal of xlsx_rs here: calamine
     refuses the file at an error before `</sst>`, and never reads one
     after it;
   - a `uniqueCount` is a number when it is one or more digits and
     nothing else, as calamine reads it, with the function `parse` of
     the crate `atoi_simd`, a reader of whole numbers, called so that it
     takes any zeros first and no sign. xlsx_rs compares it with
     10,000,000 whatever its size, so one past 4,294,967,295, which
     calamine in the wasm cannot read and reserves nothing for, is
     refused, below. One that is missing or is not a number, `+2` among
     them, is let be, since calamine reserves nothing for it.

   xlsx_rs reads the XML of a part from the zip as it unzips it, in the
   reading of point 1 that counts its bytes, and holds one element of it
   at a time, as calamine does; it never holds a whole part. So reading a
   table of texts takes at most the memory calamine's reading of it takes
   after.
4. **The merged ranges.** calamine reads every merged range of the sheet,
   each an element `mergeCell`, into memory, 16 bytes each, before xlsx_rs
   sees one: 40,000,000 in a zip of 5.1 MB held 650 MB, in the review of
   work package 2. The part of the sheet is the one the relationships of
   the workbook give it, which may name any part of the zip, so xlsx_rs
   does not look for it. As it reads each part to its end, point 1, it
   counts the times the nine bytes `mergeCell` come right after `<` or
   `:`, and more than `max_cells` in one part refuse the file, "too many
   merged ranges". An element `mergeCell`, with a prefix or without, has
   those bytes at the start of its name, since quick-xml reads the bytes
   of a part as they are and does not decode another encoding into them,
   so the count is at least what calamine can hold; it is more by one for
   the start of `mergeCells` around the ranges, by the end tag of one
   with a prefix, `</x:mergeCell>`, and by `<mergeCell` written as text
   in a comment. The bytes are counted as they pass, a match split
   between two reads of the zip counted too, and none is held. The bound is generous, not a law: ranges
   that do not overlap inside the rectangle cannot be more than its
   cells, but ranges outside it are let be; 2,000,000 of them hold 32 MB.

Where xlsx_rs refuses what calamine would read, accepted on 28 September
2026 after the review, since no program a user saves with writes such a
file:

- a part in a compression the zip crate does not read, even one calamine
  never opens;
- a workbook whose other sheets, not read, hold more merged ranges than
  `max_cells`, 2,000,000 in popnei_web; and a sheet read with a few
  ranges fewer, when the element `mergeCells`, a `mergeCell` with no
  attribute `ref`, which calamine skips, or the others of point 4 bring
  the count past it;
- a `uniqueCount` past 10,000,000 in a table of fewer texts, for which
  calamine would reserve the room, and one past 4,294,967,295, the
  largest number a `usize` of the wasm holds, which calamine there does
  not read as a number and reserves nothing for;
- a part named as one of point 3 in a folder calamine does not read,
  held to the same bound;
- a part of point 3 in an encoding other than UTF-8.

Not bounded: the time. calamine's `read_styles` reads the format of a
style once for each style that uses it (`detect_custom_number_format`
of its `formats.rs`), so a part of styles of 5 MB of one format and
250,000 styles using it, within the bound, is about 1.25 × 10^12
characters read, minutes or more in the worker; computed in the spec's
review, not measured. It is slow and not a trap.

What it costs. Every part is unzipped twice, once by xlsx_rs and once by
calamine, the package relationships, the workbook and the tables of
texts parsed once more, and every part searched for `mergeCell`. The
plan measures under node 26.8.2 on the owner's Mac the read of
`individuals_10000.xlsx`, and of a sheet of 2,000,000 cells, which took
874 ms there before, and the owner is told if either more than doubles;
the numbers are in the report of the plan, `docs/reports/own-parts.md`.
The two crates are already in the package through calamine; what
xlsx_rs's own code adds to its size is measured with the plan and given
there. Two direct dependencies more, `zip` and `quick-xml`, pinned to
calamine's versions and upgraded with it.

The tests, at `read_first_sheet`, each with a file written by the test
unless it is named:

- the owner's `excel_1904.xlsx`, which gives `2024-05-13` and
  `14:30:00`; a file written by the test in Strict Open XML, of the
  1904 system; the rules of point 2,
  each in a file: a `workbookPr` with a prefix, `date1904="0"`, two
  `workbookPr` children of the root, of which the last is taken, the last
  of two `officeDocument` relationships, one with its `Type` written
  twice, which is not read, and one after `</Relationships>`, not read
  either; and a `_rels/.rels` that names no workbook, "the file names no
  workbook";
- a file written by rust_xlsxwriter, each byte of its sheet's compressed
  data changed in turn, no copy giving cells other than the file's;
- each bound of points 1 and 3 at its value, a file that reaches it
  read and one a byte or a text past it refused: 1,000,000,000 bytes
  unzipped, 50,000,000 bytes of each of the four settings parts,
  400,000,000 bytes of a table of texts, 10,000,000 texts, a
  `uniqueCount` of 10,000,000, and 100,000,000 bytes of the paths of the
  sheets. A test that takes more
  than 10 s in `cargo test` on the owner's Mac is marked `#[ignore]`,
  with that reason, and run before each release; the messages of these
  refusals are also tested with small files, through a function private
  to the crate that takes the bounds as arguments, by tests inside the
  crate;
- a table of texts whose `uniqueCount` is 400,000,000 with two texts,
  refused; the same cut short before `</sst>`, refused; a `uniqueCount`
  of 1,000 with two texts, read; one of `00010000001`, refused, and of
  `+400000000`, read;
- a part of relationships declared `windows-1252`, and a table of texts
  that starts with the byte order mark of UTF-16: each refused;
- the file of the spec's review, 2,000 sheets naming one target of
  500 KB: refused;
- a table of texts named in capitals, `XL/SHAREDSTRINGS.XML`, checked
  as the other, and a workbook in a folder `data/`, whose table of texts,
  settings parts and merged ranges are checked as those of `xl/`;
- a sheet with more merged ranges than `max_cells`; the same in a
  workbook that lists a decoy sheet inside a `definedName` before it,
  and in one with a stray end tag `</x>` before its sheets: each refused;
- the three files of the review that trapped the package, each with a
  form feed before the name of an attribute, `\x0CuniqueCount=` in the
  table of texts, `\x0CType=` in `_rels/.rels` and `\x0Cname=` on the
  sheet read, each holding what it held in the review: each refused.

Under node, the owner's 1904 file, and a file of 2.6 KB whose
`uniqueCount` of 400,000,000 trapped the package, written by the test
helper of hand-made files and committed as
`tests/data/unique_count.xlsx`, now refused.

## The Rust interface

The plain functions, which `cargo test` calls natively, in the library
crate of xlsx_rs, and the exported one, a thin wrapper over them, in its
crate of the binding to wasm-bindgen, as popnei has `popnei` and
`popnei-js`: the functions wasm-bindgen generates cannot run natively.

A cell as it crosses to JavaScript, and the sheet read:

```rust
/// A cell of the sheet: a date, a time, a duration and an error are text
/// by then, as "Each cell" gives them.
pub enum SheetCell {
    Empty,
    Text(String),
    Number(f64), // finite
    Bool(bool),
}

/// The rectangle of the first worksheet that is not hidden, from the
/// first row and column that hold a value to the last ones.
pub struct Sheet {
    pub name: String,
    pub first_row: u32,     // as Excel numbers the rows, from 1
    pub first_column: u32,  // column A is 1
    pub num_rows: u32,
    pub num_columns: u32,
    pub cells: Vec<SheetCell>, // row after row, num_rows × num_columns
}
```

What xlsx_rs refuses, each with what its words need, and a file that
calamine cannot read, with calamine's message:

```rust
pub enum Refusal {
    NotXlsx,
    OldExcel,
    Encrypted,
    EmptySheet { sheet: String },
    CellError { error: String },          // "#GETTING_DATA"
    /// The rectangle of the values read when it passed the limit, in the
    /// numbers of Sheet.
    SheetTooLarge {
        sheet: String,
        first_row: u32,
        first_column: u32,
        num_rows: u32,
        num_columns: u32,
    },
}

pub enum ReadError {
    Refused(Refusal),
    Unreadable(String), // calamine's message, for the console
}

/// Reads the first worksheet that is not hidden of the xlsx `bytes`,
/// refusing it at the first cell that makes the rectangle of the values
/// larger than `max_cells` cells.
pub fn read_first_sheet(bytes: &[u8], max_cells: u32) -> Result<Sheet, ReadError>;
```

The exported function. It returns the sheet or the refusal as a value,
and throws only for a file calamine cannot read, as popnei's coding skill has
every exported function do, with a `Result` whose error wasm-bindgen
turns into a JavaScript `Error` with its message
(popnei_web's `.claude/skills/coding/worker.md`, "Errors are values"). The refusals
are values and not errors because each carries fields its words need,
which the message of an `Error` would carry only as text to be taken
apart again. The result is a struct that stays in the wasm's memory;
JavaScript reads its fields through functions wasm-bindgen generates,
each of which copies the field out (`getter_with_clone`), and the light
worker then calls its `free()`: JavaScript's garbage collector frees it
too, through a `FinalizationRegistry` wasm-bindgen registers, but at a
time nobody chooses, and a read holds up to 2,000,000 cells.

```rust
#[wasm_bindgen(getter_with_clone)]
pub struct XlsxRead {
    /// "" for a sheet read; otherwise "notXlsx", "oldExcel",
    /// "encrypted", "emptySheet", "cellError" or "sheetTooLarge".
    #[wasm_bindgen(readonly)] pub refusal: String,
    /// The text of the error, for "cellError"; "" otherwise.
    #[wasm_bindgen(readonly)] pub detail: String,
    /// The name of the sheet, for a sheet read, "emptySheet" and
    /// "sheetTooLarge".
    #[wasm_bindgen(readonly)] pub sheet: String,
    /// The rectangle of the sheet read, or of "sheetTooLarge".
    #[wasm_bindgen(readonly, js_name = firstRow)] pub first_row: u32,
    #[wasm_bindgen(readonly, js_name = firstColumn)] pub first_column: u32,
    #[wasm_bindgen(readonly, js_name = numRows)] pub num_rows: u32,
    #[wasm_bindgen(readonly, js_name = numColumns)] pub num_columns: u32,
    /// Row after row: null, a string, a number or a boolean; empty for a
    /// refusal.
    #[wasm_bindgen(readonly)] pub cells: Vec<JsValue>,
}

#[wasm_bindgen(js_name = readXlsx)]
pub fn read_xlsx(bytes: &[u8], max_cells: u32) -> Result<XlsxRead, JsError>;
```

What wasm-bindgen declares of it in the package, `wasm/xlsx_rs.d.ts`,
which the TypeScript of popnei_web's light worker reads: this is the
contract between the two projects. A second crate of trial, of 27
September 2026, with this struct and this function, generated these
lines, the fields in the order of their names, with wasm-bindgen 0.2.128
(the comments are added here):

```ts
export class XlsxRead {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly cells: any[];
    readonly detail: string;
    readonly firstColumn: number;
    readonly firstRow: number;
    readonly numColumns: number;
    readonly numRows: number;
    readonly refusal: string;
    readonly sheet: string;
}

/** Throws an Error with calamine's message for a file it cannot read. */
export function readXlsx(bytes: Uint8Array, max_cells: number): XlsxRead;

/** Fetches and compiles xlsx_rs_bg.wasm, from beside xlsx_rs.js when it is
    called with no argument, as the light worker calls it. */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
```

JavaScript cannot make an `XlsxRead` of its own, `private
constructor()`. `[Symbol.dispose]` frees it as `free()` does; the
light worker calls `free()`, in a `finally`.

wasm-bindgen copies the bytes of the `Uint8Array` into the memory of the
wasm, 20 MB at most, and `cells` makes a JavaScript array of the values
each time it is read, so the light worker reads it once.

## The package

What xlsx_rs releases, as popnei releases its own
(`/Users/jose/devel/popnei/js/popnei/package.json`): a package named
`xlsx_rs`, in `js/xlsx_rs/` of its repository, of `"type": "module"`,
whose `exports` give `./wasm/xlsx_rs.js` with its declarations
`./wasm/xlsx_rs.d.ts`, and whose `files` hold `wasm/`, the README and
the license. Its `build` compiles the binding crate with `cargo build
--release --target wasm32-unknown-unknown`, then runs `wasm-bindgen
--target web --remove-name-section --out-dir wasm --out-name xlsx_rs`
over it, which writes the JavaScript, its declarations and
`xlsx_rs_bg.wasm`; the build compiles with `--remap-path-prefix`, so that
no path of the folders of the machine that built it is left in the
`.wasm`, where 38 were before the review of 28 September 2026. `npm pack`
builds and tests the package before it packs it, so that a release
cannot pack a `wasm/` left by the build of another commit. The package is packed with `npm pack`, and the
`.tgz`, `xlsx_rs-0.1.0.tgz`, is attached to a pre-release of a tag
`js-v0.1.0-dev.1`, then `dev.2` and on, each used once and never moved,
since popnei_web's lockfile keeps the hash of the file of each URL. The
release is made by hand, as popnei's are, until the two have a workflow
they share, as the owner decided on 28 September 2026
(popnei_web's `docs/architecture.md`, section 13, point 13). The package is its
wasm-bindgen output, with no TypeScript of its own around it: popnei_web
calls one function and the `init` that loads the wasm.

A change of the declarations above is a change of the contract: it is a
new tag, and a change of `readXlsxCells` in popnei_web in the same work,
with a new URL in its `package.json`.

### Its size

The download is the wasm and its JavaScript, measured in the two crates
of trial of 27 September 2026, which have calamine alone and little code
of their own, each a little different, built with the profile below and
wasm-bindgen as above, with Rust 1.98.0 on the owner's Mac, gzipped with
`gzip -9`; their files were named `files` and not `xlsx_rs`, which
changes nothing of their size:

| file | raw | gzipped |
|---|---|---|
| the `.wasm`, calamine and wasm-bindgen | 533,415 and 533,519 bytes | 295,475 and 295,521 bytes |
| the JavaScript wasm-bindgen generates, in the crate with the struct `XlsxRead` above | 11,892 bytes | 2,962 bytes |
| popnei's wasm package, for comparison (popnei_web's `docs/technology.md`, section 2) | 2.16 MB | 0.71 MB |

So 0.30 MB, the reader's own code a few hundred bytes of it, which
agrees with the 0.29 MB popnei_web's `docs/technology.md` measured for calamine
inside the crate of the three libraries, and is less than half of
popnei's wasm, which every user of popnei_web downloads. The sizes are
measured again from each release, and in popnei_web from the site built
with it.

## Its dependencies

The manifests of xlsx_rs are its own, and take the lints of
`.claude/skills/coding/lints.toml`, popnei's, as popnei's `Cargo.toml`
does. What they depend on, for popnei_web's stage 4:

```toml
[dependencies]
# Pinned to the command line of wasm-bindgen, which refuses a crate of
# another version; the version popnei pins, so that the one command line
# on the owner's Mac builds both. The binding crate alone.
wasm-bindgen = "=0.2.128"
# The version docs/technology.md measured. Its default features are none;
# chrono, for dates as chrono's types, and picture, for the images of a
# workbook, are left off: xlsx_rs writes a date from calamine's own
# parts of it. The library crate.
calamine = { version = "=0.36.1", default-features = false }
# The zip and the XML xlsx_rs reads itself before calamine ("What xlsx_rs
# reads before calamine"), at the versions and with the features calamine
# takes them, so that the package holds one copy of each. Upgraded with
# calamine.
zip = { version = "=8.6.0", default-features = false, features = ["deflate"] }
quick-xml = { version = "=0.41.0", features = ["encoding"] }

[dev-dependencies]
# The xlsx files of the tests are written in memory; the library the
# writer of popnei_web's stage 6 will write with, in the tests only until then.
rust_xlsxwriter = { version = "=0.99.1", default-features = false }

[profile.release]
# As measured in docs/technology.md, section 2.
opt-level = 3
lto = true
codegen-units = 1
```

calamine brings zip, with its compression in Rust, and quick-xml; none
has C, and the trial built them for `wasm32-unknown-unknown`. The owner
approved calamine 0.36.1 and rust_xlsxwriter 0.99.1 on 28 September
2026, as dependencies of xlsx_rs, the second for its tests alone until
popnei_web's stage 6; calamine had been left unapproved on 27 September 2026 while
the owner weighed this project (popnei_web's `docs/architecture.md`, section 13,
point 12). `Cargo.lock` is committed, and `rust-toolchain.toml` names Rust
1.98.0, the stable release of 18 August 2026, current on 27 September
2026 and the one on the owner's Mac, with the target
`wasm32-unknown-unknown`; calamine 0.36.1 needs 1.88 at least. A panic
is a trap of the wasm, which ends popnei_web's light worker, so the
lints that deny `unwrap`, `expect`, `panic!` and indexing with `[]` hold
outside the tests, and the library has `#![forbid(unsafe_code)]`.

## The cases

- **An identifier `001` stored as a number with the format `000`**:
  the cell is the number 1, and the individual is named `1`, since the
  format is not read. The check against the variants names `1` as
  missing from the file. A user who types `'001`, or formats the column
  as text before typing, gets the text `001`.
- **An xlsx with its table on the second sheet** and notes on the first:
  the notes are read, and the user sees a table of notes. When the first
  sheet is empty, the refusal says only the first sheet is read.
- **Several tables in one sheet**, side by side: one rectangle, whose
  columns between the two are empty in the header, which the reader
  refuses or drops by the rules of a CSV.
- **A value far from the table**, a note in Z5 beside a table of
  columns A to F: the rectangle reaches it, the columns G to Y, with no
  name and no value, are dropped, and the reader refuses column Z, with
  a value and no name, "column Z has values but no name in the header".
  A note in Z1, in the row of the header, is instead the name of a
  column Z with no value in any row, which the user sees in the table.
  Neither reaches the limit of 2,000,000 cells: a note in column XFD,
  over a table of 123 rows or more, does, and is refused as
  `sheetTooLarge`, naming column XFD.
- **The same sheet saved as CSV and as xlsx** gives one table but for
  the dates, `13/05/2024` in the CSV and `2024-05-13` in the xlsx, the
  numbers, as Excel shows them in the CSV, `1,8` rounded to its format,
  and as it stores them in the xlsx, 1.75, and the booleans, the text
  `VERDADERO` in a CSV of Spanish Excel and a boolean in the xlsx.

## How it runs

In the light worker, one read at a time. At its largest, near its end, a
read holds at once: the bytes, 20 MB at most, in the worker's memory and
a copy in the wasm's; the texts of the workbook, which an xlsx keeps in
one table and calamine holds whole; the cells with a value, 2,000,000
at most since the size is checked as they are read, and the rectangle
made of them; and the JavaScript array of the cells, 2,000,000 at
most. The memory of a wasm grows and never shrinks, so the worker keeps
the largest a read has needed until it is ended. None of it is measured;
the Playwright test of the Individuals step measures the time of a sheet
of 10,000 rows and 20 columns in popnei_web
(popnei_web's `docs/specs/worker/individuals.md`, "How it is verified").

## How it is verified

### With cargo test, natively

At `read_first_sheet`, with the checks of xlsx_rs, which run `cargo fmt
--check`, clippy with its lints, and `cargo test`. Two sets of files.

**Written by the tests, in memory**, with rust_xlsxwriter's
`Workbook::save_to_buffer`, each case a few lines that say what the file
holds, and the literal cells it gives:

| the file | gives |
|---|---|
| a table at A1 of text, numbers, booleans and an empty cell | its cells, `Empty` for the empty one |
| the table at C3 | `first_row` 3, `first_column` 3, the same cells |
| a row with its second cell not written | `Empty` there |
| a number with the format `000` | `Number(1.0)` |
| 45425 with the format `dd/mm/yyyy`; 45425.5 with `dd/mm/yyyy hh:mm:ss`, and with `dd/mm/yyyy`; 45425.9999999999 with `dd/mm/yyyy`; 0.604166666 with `hh:mm`; 1.5 with `[h]:mm:ss`; 60, 2958465 and 2958466 with `dd/mm/yyyy`; −3 with `dd/mm/yyyy` | `2024-05-13`; `2024-05-13 12:00:00` both times (**Open 2**, decided); `2024-05-14`; `14:30:00`, `36:00:00`, `1900-02-29`, `9999-12-31`, `Number(2958466.0)`, `Number(-3.0)`; all seen as dates by calamine in the trials |
| xlsx_rs's function that makes the cell of a date, in its own module, with the 1904 system, which rust_xlsxwriter does not write: 2957003, 2957004, −3 and 0.5 | `9999-12-31`, `Number(2957004.0)`, `Number(-3.0)`, `12:00:00` |
| a formula `=1+1` saved with the value 2, one with the text `x`, one with `TRUE`, one with `#N/A`, one with `#DIV/0!`, and `=1+2` with none | `Number(2.0)`, `Text("x")`, `Bool(true)`, `Text("#N/A")`, `Text("#DIV/0!")`, `Number(0.0)`; popnei_web's reader makes both errors missing, not xlsx_rs |
| `0.1 + 0.2` | `Number(0.30000000000000004)` |
| a text `"  sp "` and a text with a line break | both as they are |
| a text of several fonts, `write_rich_string` | the parts joined |
| a population merged over rows 2 to 4, and a name merged over two columns of the header | the population in the three rows; the name in both columns |
| a hidden first sheet, and the table on the second | the second, by its name |
| a very hidden first sheet, which only a macro can show, and the table on the second | the second, by its name |
| a table with a hidden row and a hidden column | both read |
| "id" at A1, and a blank cell with a format at E10 | a rectangle of one cell: a format is not a value |
| values at B1, C1 and A3 | the rectangle from row 1 and column A, 3 × 3 |
| a first sheet with no value, and a table on the second | `EmptySheet`, with the name of the first |
| a value at A1 and one at XFD200, with `max_cells` 2,000,000 | `SheetTooLarge`, from row 1 and column 1, 200 rows and 16,384 columns |
| a value at XFD1, and one in column A of each row down to row 200 | `SheetTooLarge` at row 123, 16,384 × 123 being the first rectangle above 2,000,000: 123 rows and 16,384 columns, the rows after it not read |
| a formula saved with the value `#GETTING_DATA` | `CellError`, `#GETTING_DATA` |
| the bytes of `id,pop\n` | `NotXlsx`; and the empty bytes |
| the first 500 bytes of an xlsx | `Unreadable`, with the zip crate's message (since xlsx_rs opens the zip itself, "What xlsx_rs reads before calamine") |
| the 22 bytes of an empty zip, `PK` and the bytes 5 and 6 | `NotXlsx` |
| "id" at C2 and a value at XFD200 | `SheetTooLarge` from row 2 and column 3, whose first row and column differ |
| a cell written 3 times at A1, with `max_cells` 2 | `Unreadable`, "cells written more than once" |
| a text of 100,000 characters merged over enough cells to pass `MAX_TEXT_BYTES` | `Unreadable`, "too much text" |
| a range written `B4:B2`, and two ranges that overlap | the range filled as `B2:B4`; `Unreadable`, "overlapping merged ranges" |
| a merged range whose other cells hold values in the file | every cell the first cell's value |
| the eight bytes of a compound file of Office followed by zeros | `OldExcel` |
| a compound file with `EncryptedPackage` in UTF-16 among its bytes, cut short | `Encrypted`, and no panic |

A text with no character, the first row of "Each cell", a whole number,
a date and a duration written in ISO 8601, and `GettingData`, are tested
at the function that makes the cell of one value, since calamine's
reader of an xlsx gives none of the last four; the text with no
character, since rust_xlsxwriter writes
an empty text as no cell and a formula saved with `""` as the number 0.

rust_xlsxwriter writes neither the date system of 1904, nor a password,
nor an `.xls`, nor the errors of the newest Excel, nor a date as ISO
text; those come from the owner's files.

**Made by the owner**, since popnei_web's `docs/technology.md` asks, in its open point
1, whether calamine reads right the files that users make. They are kept
in `tests/data/`, and their tests assert the cells the owner
says the file shows in Excel, as literals. Each is small, a header and
five rows, typed by hand as a user would:

1. `excel_es.xlsx`, Excel in Spanish: `Individuo`, `Población`,
   `Altura`, `Fecha`, `Hora`, `Afectado`, `Código`; accented names;
   heights typed with a decimal comma, `1,75`; a date typed `13/05/2024`;
   a time `14:30`; `VERDADERO` and `FALSO`; an identifier typed `'001`;
   a code `7` with the format `000`; a cell of `=NOD()`, the Spanish
   `NA()`, and one of `=1/0`; a population merged over two rows; a blank
   row in the middle.
2. `excel_en.xlsx`, the same in Excel in English, with `=NA()`. The file
   committed is a copy of `excel_es.xlsx`, decided on 28 September 2026:
   the owner's English file had been made by pasting the values of the
   Spanish one, without its formulas and its merged range, and an xlsx
   stores its formulas and its errors in English whatever the language
   of Excel, as `excel_es.xlsx` shows, `1/0` and `#DIV/0!`.
3. `libreoffice.xlsx`, the same in LibreOffice Calc, saved as "Excel
   2007-365 (.xlsx)". Not made yet: LibreOffice is not on the owner's
   Mac; its test waits for it.
4. `excel_1904.xlsx`, the date of the first file in a workbook set to
   the date system of 1904 (in Excel for Windows, File › Options ›
   Advanced; in Excel for Mac, Preferences › Calculation): the same text,
   `2024-05-13`. The owner's file gave `2020-05-12` through calamine,
   whose reading of the date system is wrong for it ("What xlsx_rs reads
   before calamine", point 2); its test waits for xlsx_rs to read it.
5. `encrypted.xlsx`, any table saved with a password to open it:
   `Encrypted`.
6. `excel97.xls`, any table saved as "Excel 97-2003 Workbook":
   `OldExcel`.
7. `spill.xlsx`, from Excel 365: a cell `=SEQUENCE(3)` with a value
   under it, which gives `#SPILL!`: `CellError`, `#SPILL!`, or the text
   `#VALUE!` if Excel saves it so (above, "The refusals", point 5). What
   it gives settles the words of `cellError`, and the spec is corrected
   to it.
8. `google_sheets.xlsx`, the first file downloaded from Google Sheets as
   .xlsx, if the owner uses it. The owner's was made from the pasted
   English file, so it holds no formula and no merged range, `001` as the
   number 1 and the Spanish text `#¡DIV/0!`; it checks Google's writer on
   values alone, every one given back as it was. A copy made from
   `excel_es.xlsx` would check it on the rest.

Until they arrive, their tests are written and marked `#[ignore]` with
the name of the file they wait for, and the report of the plan says
which ran. What each gives that this spec does not expect, a date read
as a number, a cell missing, is a finding for this spec and not a test
to be bent.

**No file makes it panic**, added on 28 September 2026 by the owner's
decision, from the rules of pop_var_caller's review. One test takes a file
written by the tests with a text, a number, a boolean, a date, a formula,
a merged range and a hidden first sheet, and a compound file with
`EncryptedPackage` in UTF-16 among its bytes, and reads each cut short at every
length from 0 bytes to its whole size, and with each of its bytes in turn
replaced by its complement, the byte with every bit flipped. A zip keeps
its directory at its end, so a copy cut short stops in the reader of the
zip and a byte flipped in the compressed data mostly stops at its
checksum, and neither reaches calamine's reading of the XML; so the test
also builds the file from its parts, stored in the zip uncompressed,
and cuts short and flips each part xlsx_rs reads, the workbook, its
relationships, the table of texts, the sheet and the styles, before it
zips them again (added after the review of work package 4, 28 September
2026). For every
copy, `read_first_sheet` returns, a sheet, a refusal or an error, and
does not panic, which the test checks with `std::panic::catch_unwind`;
it asserts nothing of which. A panic is a trap in the wasm that ends
popnei_web's light worker (above, "The refusals"). One found in calamine
is a finding for this spec and an issue for calamine, and the test keeps
the copy that found it as a case of its own.

### The package, built

A test under node of the package as it is released, which gives its
`init` the bytes of the `.wasm`, as `init({ module_or_path: bytes })`,
since wasm-bindgen 0.2 warns when the bytes are given bare, and since node's `fetch` does not read the
file beside its JavaScript, as a browser does, and reads
`excel_en.xlsx` and `encrypted.xlsx`: the cells the Rust tests give, and
the refusal `encrypted`. It is what checks a release before it is
tagged, as popnei's `npm test` checks its package.

Until the owner's two files exist, as the owner decided on 28 September
2026, the test reads in their place `tests/data/written.xlsx`, a file of
a table of a few rows with a text, a number, a boolean, a date and a
merged range, which a test of xlsx_rs marked `#[ignore]` writes with
rust_xlsxwriter when it is run by hand and which is committed; and the
bytes of `id,pop\n`, a CSV: every cell of the first as the Rust tests
give it, its type in JavaScript among it, a blank cell `null`, and the
refusal `notXlsx` for the second. Two more files written the same way,
a first sheet with no value and a table that starts at C2, give the
refusal `emptySheet` with its sheet's name and a rectangle whose first
row and first column differ; and two more, a cell saved with
`#GETTING_DATA` and a table from C2 read with a small `maxCells`, give
`cellError` with its text in `detail` and `sheetTooLarge` with its
rectangle, so that the codes and the fields popnei_web reads are each
checked in the package. They stay when the owner's
files arrive, and a release made before that says in its notes that
`excel_en.xlsx` and `encrypted.xlsx` were not read.

The same test compares the declarations wasm-bindgen generated,
`wasm/xlsx_rs.d.ts`, with `js/xlsx_rs/test/xlsx_rs.d.ts`, kept in git:
the whole file wasm-bindgen writes: the lines of "The Rust interface"
above, the doc comments of the binding crate, which it copies as
comments of JavaScript, and the types and the function `initSync` it
always adds, which the spec does not show; but not the interface
`InitOutput`, the functions wasm-bindgen exports for its own
JavaScript, which change with its version and are not read by
popnei_web, and which the test leaves out of both files before it
compares them. A difference fails the test, so that a
change of the contract with popnei_web is made in that file, and seen,
before it is released (added on 28 September 2026 by the owner's
decision).

### In popnei_web

`readXlsxCells` under Vitest, with an object of the test in the place of
the package, and the flow of the Individuals step in Chromium, Firefox
and WebKit, with `excel_en.xlsx`, `encrypted.xlsx` and a sheet of 10,000
rows and 20 columns, `individuals_10000.xlsx`, which a test of xlsx_rs
marked `#[ignore]` writes when it is run by hand, copied into
popnei_web's `e2e/fixtures/`: popnei_web's `docs/specs/worker/individuals.md`, "How it is
verified". A finding there about the cells is a finding for
this spec, fixed here and released.

## What this spec asks of other documents

Of popnei_web, all made on 27 and 28 September 2026:

- popnei_web's `docs/specs/worker/individuals.md`: the refusals `notXlsx`,
  `oldExcel`, `encrypted`, `emptySheet`, `cellError`, `sheetTooLarge` and
  `xlsxReaderNotLoaded` and their words, `MAX_SHEET_CELLS`, and, since 28
  September 2026, the light worker's part of this spec, above.
- popnei_web's `docs/specs/worker/messages.md`: the request of an xlsx, and the load
  of the files wasm that fails.
- popnei_web's `docs/technology.md`, section 2 and open point 1: the measure of
  calamine alone; that calamine 0.36.1 refuses a whole sheet at an error
  it does not know, and that whether `#SPILL!` reaches it as such waits
  for the owner's `spill.xlsx`; and the answer to open point 1 when the
  owner's files have been read.
- popnei_web's `docs/functionality.md`, section 4: "the first sheet" is the first that
  is not hidden, and merged cells take the value Excel shows over them;
  and, since 28 September 2026, the errors of an xlsx, `#N/A` and the
  six others calamine knows, among the missing values.

Of xlsx_rs, made with its repository on 28 September 2026: a
`CLAUDE.md`, the skills of popnei adapted to it, among them the coding
skill with these lints and the thin exported functions, and `docs/` with
its architecture and this spec.

## Open points

1. **An error cell of Excel, decided by the owner on 28 September 2026:
   every error calamine knows is missing.** A cell whose formula failed
   holds an error, and xlsx_rs gives its text, `#N/A`, `#DIV/0!`, as
   Excel writes it in English, whatever the language of Excel. The owner
   decided that popnei_web reads `#N/A` as a missing value, as Excel
   means it, "not available", and as a lookup, `VLOOKUP`, gives it for
   an individual it did not find: a column of heights with one `#N/A`
   stays continuous, and that individual has no value in it. The option
   not taken, which had been recommended, was to read it as the text
   `#N/A`, as the reader of CSV reads it in the CSV Excel saves from the
   same sheet; so an xlsx and that CSV give two tables, the first with
   the cell missing and the second with the text, which makes the
   column categorical there and shows the value to the user.

   It is popnei_web's reader, and not xlsx_rs, that makes an error
   missing, with its other missing values, `NA` and `-`
   (popnei_web's `docs/specs/worker/individuals.md`, "The xlsx"): xlsx_rs gives what
   the file holds, the text of the error, and what counts as missing is
   the rule of the application, in one place. Since the cell reaches the
   reader as text, the rule is a text equal to one of the seven errors,
   and a cell where the user typed `#N/A` as text is missing too. The option not taken, an
   empty cell given by xlsx_rs, would put that rule in the library, and
   a second user of xlsx_rs could not tell an error from a blank.

   The other six errors, `#DIV/0!`, `#NAME?`, `#NULL!`, `#NUM!`,
   `#REF!` and `#VALUE!`, are missing as well, decided by the owner
   later on 28 September 2026 (popnei_web's `docs/specs/stage-4-open-points.md`,
   point 11). They say that a formula went wrong, a division by 0, a
   name or a reference that does not exist, an argument of the wrong
   kind, and read as missing such a cell leaves its individual with no
   value in that column, as `#N/A` does: a column of heights with one
   `#DIV/0!` stays continuous. The option not taken, which the writers
   had chosen: the six as their text, so that a broken formula made its
   column categorical and showed among its values, for the user to mend
   in the sheet. An error calamine does not know, `#SPILL!` among them,
   still refuses the sheet (above, "The refusals", point 5).

2. **A date whose time the format hides, decided by the owner on 27
   September 2026.** calamine does not give the
   format of a cell, only that it is one of a date or a time, so the
   crate cannot tell `13/05/2024` from `13/05/2024 14:03`. The rule
   decided, as recommended: a number whose time, rounded to the millisecond, is 0
   gives the date alone, `2024-05-13`, and any other gives the date and
   the time, `2024-05-13 14:03:00`. A date typed by hand has no time, so
   it gives what the user sees. A date that a formula such as `=NOW()`
   made, or that another program wrote with its time, and that the
   format shows as a date alone, gives the date and the time: a column
   of such dates shows times the user did not see in Excel, and, being
   text, may have a value for each individual where Excel shows a few
   dates. The user sees it in the table; nothing is left out. The option
   not taken: always the date and the time, `2024-05-13 00:00:00` for a
   date typed by hand, which is the same for every date and never
   depends on the number, but adds a time of midnight to every date,
   the common case, to spare the rare one. A third, reading the format
   from the file itself, is not in calamine 0.36.1, and would be a
   reader of the styles of the workbook written for xlsx_rs, and was
   not taken either.

## Not in this spec

- What the cells become in the table, the header, the first column, the
  spaces, the missing values, the types, and the words of the refusals;
  and how popnei_web's light worker loads the package and what the user
  sees while it downloads: popnei_web's `docs/specs/worker/individuals.md`.
- Which files the Individuals step loads as an xlsx, by their name, and
  what it shows while one is read: popnei_web's `docs/specs/steps/individuals.md`.
- Writing an xlsx and zipping the report, and what they add to the
  download: stage 6 of popnei_web, and a spec of xlsx_rs then
  (popnei_web's `docs/architecture.md`, section 13, point 14).
- Whether pandas, reading the xlsx the report writes, gives a number as
  the same text as the application, `1` and not `1.0`: popnei_web's stage 6, with the
  report (popnei_web's `docs/specs/analyses/diversity.md`, "Not in this spec").
- A workflow that builds and releases the package, shared with popnei
  later, as the owner decided on 28 September 2026: section 13, point
  13, of popnei_web's `docs/architecture.md`.
- `.xlsm`, a workbook with macros, which calamine reads as an xlsx, and
  `.ods`: whether the step loads them is the step's.
