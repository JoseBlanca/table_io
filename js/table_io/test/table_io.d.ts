/* tslint:disable */
/* eslint-disable */

/**
 * What `convertColumn` gives: the column converted, as its five arrays,
 * or, with `numFailed` 1 or more, how many values do not convert and the
 * first of them. It stays in the memory of the wasm, each read of a field
 * copies it out, and `free()` releases it, once.
 */
export class Conversion {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The values, 1 for true and 0 for false or a missing one, when
     * converted to "boolean"; empty otherwise.
     */
    readonly booleans: Uint8Array;
    /**
     * The row of the first value that does not convert, among the rows
     * of the table, from 1; 0 when the column converted.
     */
    readonly firstRow: number;
    /**
     * The text of the first value that does not convert; "" when the
     * column converted.
     */
    readonly firstText: string;
    /**
     * The values, 0 for a missing one, when converted to "float"; empty
     * otherwise.
     */
    readonly floats: Float64Array;
    /**
     * The values, 0 for a missing one, when converted to "integer";
     * empty otherwise.
     */
    readonly integers: BigInt64Array;
    /**
     * For each row, 1 when its value is missing and 0 otherwise; empty
     * when a value does not convert.
     */
    readonly missing: Uint8Array;
    /**
     * How many values do not convert; 0 when the column converted.
     */
    readonly numFailed: number;
    /**
     * The values, "" for a missing one, when converted to "text"; empty
     * otherwise.
     */
    readonly texts: string[];
}

/**
 * What `importTable` gives: a table, or the kind of a refusal with the
 * fields its words need. It stays in the memory of the wasm, each read of
 * a field or of a column copies it out, and `free()` releases it, once.
 */
export class TableRead {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * The values of the column `index` when it is a boolean column, one
     * for each row, 1 for true and 0 for false or a missing one; empty for
     * a column of another type. Each call makes a new array.
     *
     * # Errors
     *
     * Throws an `Error` for an index that is not below `numColumns`.
     */
    columnBooleans(index: number): Uint8Array;
    /**
     * The values of the column `index` when it is a float column, one for
     * each row, 0 for a missing one; empty for a column of another type.
     * Each call makes a new array.
     *
     * # Errors
     *
     * Throws an `Error` for an index that is not below `numColumns`.
     */
    columnFloats(index: number): Float64Array;
    /**
     * The values of the column `index` when it is an integer column, one
     * for each row, 0 for a missing one; empty for a column of another
     * type. Each call makes a new array.
     *
     * # Errors
     *
     * Throws an `Error` for an index that is not below `numColumns`.
     */
    columnIntegers(index: number): BigInt64Array;
    /**
     * For each row of the column `index`, 1 when its value is missing and
     * 0 otherwise. Each call makes a new array.
     *
     * # Errors
     *
     * Throws an `Error` for an index that is not below `numColumns`.
     */
    columnMissing(index: number): Uint8Array;
    /**
     * The name in the header of the column `index`, from 0.
     *
     * # Errors
     *
     * Throws an `Error` for an index that is not below `numColumns`.
     */
    columnName(index: number): string;
    /**
     * The column in the file of the column `index`, from 1: its column of
     * the sheet in an xlsx, column A being 1.
     *
     * # Errors
     *
     * Throws an `Error` for an index that is not below `numColumns`.
     */
    columnNumber(index: number): number;
    /**
     * The values of the column `index` when it is a text column, one for
     * each row, "" for a missing one; empty for a column of another type.
     * Each call makes a new array.
     *
     * # Errors
     *
     * Throws an `Error` for an index that is not below `numColumns`.
     */
    columnTexts(index: number): string[];
    /**
     * The type of the column `index`: "integer", "float", "boolean" or
     * "text".
     *
     * # Errors
     *
     * Throws an `Error` for an index that is not below `numColumns`.
     */
    columnType(index: number): string;
    /**
     * The column, from 1, its place in the row of a text file or its
     * column of the sheet of an xlsx, column A being 1, for
     * "headerError", "unnamedColumn", the first of "duplicateColumn", and
     * the first column of the rectangle of "sheetTooLarge"; 0 otherwise.
     */
    readonly column: number;
    /**
     * The decimal mark the numbers in text were read with, "point" or
     * "comma", "point" for an xlsx; "" for a refusal.
     */
    readonly decimal: string;
    /**
     * The encoding a text file was read with, "utf-8", "utf-16" or
     * "windows-1252"; "" for an xlsx and for a refusal.
     */
    readonly encoding: string;
    /**
     * The number of cells of the header, for "raggedRow"; 0 otherwise.
     */
    readonly expected: number;
    /**
     * The format found from the first bytes, "text" or "xlsx", with a
     * table and with every refusal but "unreadable", for which it is "".
     */
    readonly format: string;
    /**
     * The number of cells of the row, for "raggedRow"; 0 otherwise.
     */
    readonly found: number;
    /**
     * The line of a text file, from 1, for "unclosedQuote", "raggedRow",
     * "emptyIndividual" and the first of "duplicateIndividual"; 0
     * otherwise.
     */
    readonly line: number;
    /**
     * The header of the column of the names, the first column of the
     * file, which may be ""; "" for a refusal.
     */
    readonly namesHeader: string;
    /**
     * The column of the names in the file, from 1, its column of the
     * sheet in an xlsx; 0 for a refusal.
     */
    readonly namesNumber: number;
    /**
     * The names of the individuals, one for each row, none empty, no two
     * the same; empty for a refusal. Each read makes a new array.
     */
    readonly names: string[];
    /**
     * The number of columns other than the names, which the methods of a
     * column take by their index, from 0; 0 for a refusal.
     */
    readonly numColumns: number;
    /**
     * "" for a table; otherwise the kind of the refusal: "unreadable",
     * "tooLarge", "formatNotBuilt", "oldExcel", "encrypted",
     * "notWorkbook", "emptySheet", "cellError", "sheetTooLarge",
     * "cutShort", "notText", "variantsFile", "unclosedQuote",
     * "headerError", "empty", "unnamedColumn", "raggedRow",
     * "duplicateColumn", "emptyIndividual" or "duplicateIndividual".
     */
    readonly refusal: string;
    /**
     * The row of the sheet of an xlsx, from 1, for "headerError",
     * "emptyIndividual", the first of "duplicateIndividual", and the first
     * row of the rectangle of "sheetTooLarge"; 0 otherwise.
     */
    readonly row: number;
    /**
     * The column that repeats the name, for "duplicateColumn"; 0
     * otherwise.
     */
    readonly secondColumn: number;
    /**
     * The line of a text file that repeats the name, for
     * "duplicateIndividual"; 0 otherwise.
     */
    readonly secondLine: number;
    /**
     * The row of the sheet that repeats the name, for
     * "duplicateIndividual" of an xlsx; 0 otherwise.
     */
    readonly secondRow: number;
    /**
     * The separator, "tab", "semicolon" or "comma": the one a text file
     * was read with, or split with for "unclosedQuote" and "raggedRow";
     * "" for an xlsx and for the other refusals.
     */
    readonly separator: string;
    /**
     * The number of columns the rectangle had reached, for
     * "sheetTooLarge"; 0 otherwise.
     */
    readonly sheetColumns: number;
    /**
     * The number of rows the rectangle had reached, for "sheetTooLarge";
     * 0 otherwise.
     */
    readonly sheetRows: number;
    /**
     * The name of the sheet of an xlsx, as its tab shows it, with a table
     * and for "emptySheet" and "sheetTooLarge"; "" otherwise.
     */
    readonly sheet: string;
    /**
     * For "tooLarge", the size of the file in bytes; 0 otherwise.
     */
    readonly size: number;
    /**
     * For "unreadable", the message of the zip crate, of calamine or of
     * table_io; for "cellError" and "headerError", the error; for
     * "duplicateColumn" and "duplicateIndividual", the name; "" otherwise.
     */
    readonly text: string;
    /**
     * The line of a text file, from 1, of the first character that could
     * not be decoded and stands as U+FFFD; undefined when every character
     * was decoded, for an xlsx and for a refusal.
     */
    readonly undecodedLine: number | undefined;
}

/**
 * Converts a column to the type `to`, "integer", "float", "boolean" or
 * "text", its texts read and its floats written with `decimal`, "point"
 * or "comma". The column is given as `TableRead` gives it: its type,
 * `column_type`, `missing`, with 1 for a missing value and 0 for a value,
 * and the array of its type, of as many entries; the arrays of the other
 * types are not read.
 *
 * # Errors
 *
 * Throws an `Error` for a type or a decimal mark that is not one of its
 * strings, a `missing` or a `booleans` with an entry other than 0 and 1,
 * and an array of the column's type of another length than `missing`.
 */
export function convertColumn(column_type: string, missing: Uint8Array, integers: BigInt64Array, floats: Float64Array, booleans: Uint8Array, texts: string[], to: string, decimal: string): Conversion;

/**
 * `number` as JavaScript's `String` writes it, with the decimal mark
 * `decimal`, "point" or "comma".
 *
 * # Errors
 *
 * Throws an `Error` for a decimal mark that is not one of its strings.
 */
export function floatText(number: number, decimal: string): string;

/**
 * Reads the table of the file `bytes`, a CSV, a TSV or an xlsx, found from
 * its first bytes. `max_bytes` is the largest file accepted, in bytes, and
 * `max_cells` the largest rectangle of the values of an xlsx, in cells.
 * `encoding` is "" to find it, "utf-8" or "windows-1252"; `separator` ""
 * to find it, "tab", "semicolon" or "comma"; `decimal` "" to find it,
 * "point" or "comma"; the three are for a text file and ignored for an
 * xlsx. A file refused, or one that cannot be read, is a value: its kind
 * is in `refusal`.
 *
 * # Errors
 *
 * Throws an `Error` for a `max_bytes` that is not a whole number from 0 to
 * 2^53, a `max_cells` that is not one from 0 to 4,294,967,295, and an
 * option that is not one of its strings.
 */
export function importTable(bytes: Uint8Array, max_bytes: number, max_cells: number, encoding: string, separator: string, decimal: string): TableRead;

/**
 * Whether `text` is a missing value: "", "NA" or "-", exactly.
 */
export function isMissing(text: string): boolean;

/**
 * The boolean `text` holds, TRUE or FALSE in any case, or undefined.
 */
export function parseBoolean(text: string): boolean | undefined;

/**
 * The finite number `text` holds with the decimal mark `decimal`, "point"
 * or "comma", or undefined.
 *
 * # Errors
 *
 * Throws an `Error` for a decimal mark that is not one of its strings.
 */
export function parseFloat(text: string, decimal: string): number | undefined;

/**
 * The whole number `text` holds, from −2^63 to 2^63 − 1, or undefined.
 */
export function parseInteger(text: string): bigint | undefined;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_conversion_free: (a: number, b: number) => void;
    readonly __wbg_get_conversion_booleans: (a: number) => [number, number];
    readonly __wbg_get_conversion_firstRow: (a: number) => number;
    readonly __wbg_get_conversion_firstText: (a: number) => [number, number];
    readonly __wbg_get_conversion_floats: (a: number) => [number, number];
    readonly __wbg_get_conversion_integers: (a: number) => [number, number];
    readonly __wbg_get_conversion_missing: (a: number) => [number, number];
    readonly __wbg_get_conversion_numFailed: (a: number) => number;
    readonly __wbg_get_conversion_texts: (a: number) => [number, number];
    readonly __wbg_get_tableread_column: (a: number) => number;
    readonly __wbg_get_tableread_decimal: (a: number) => [number, number];
    readonly __wbg_get_tableread_encoding: (a: number) => [number, number];
    readonly __wbg_get_tableread_expected: (a: number) => number;
    readonly __wbg_get_tableread_format: (a: number) => [number, number];
    readonly __wbg_get_tableread_found: (a: number) => number;
    readonly __wbg_get_tableread_line: (a: number) => number;
    readonly __wbg_get_tableread_names: (a: number) => [number, number];
    readonly __wbg_get_tableread_namesHeader: (a: number) => [number, number];
    readonly __wbg_get_tableread_namesNumber: (a: number) => number;
    readonly __wbg_get_tableread_numColumns: (a: number) => number;
    readonly __wbg_get_tableread_refusal: (a: number) => [number, number];
    readonly __wbg_get_tableread_row: (a: number) => number;
    readonly __wbg_get_tableread_secondColumn: (a: number) => number;
    readonly __wbg_get_tableread_secondLine: (a: number) => number;
    readonly __wbg_get_tableread_secondRow: (a: number) => number;
    readonly __wbg_get_tableread_separator: (a: number) => [number, number];
    readonly __wbg_get_tableread_sheet: (a: number) => [number, number];
    readonly __wbg_get_tableread_sheetColumns: (a: number) => number;
    readonly __wbg_get_tableread_sheetRows: (a: number) => number;
    readonly __wbg_get_tableread_size: (a: number) => number;
    readonly __wbg_get_tableread_text: (a: number) => [number, number];
    readonly __wbg_get_tableread_undecodedLine: (a: number) => number;
    readonly __wbg_tableread_free: (a: number, b: number) => void;
    readonly convertColumn: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: number, p: number) => [number, number, number];
    readonly floatText: (a: number, b: number, c: number) => [number, number, number, number];
    readonly importTable: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number];
    readonly isMissing: (a: number, b: number) => number;
    readonly parseBoolean: (a: number, b: number) => number;
    readonly parseFloat: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly parseInteger: (a: number, b: number) => [number, bigint];
    readonly tableread_columnBooleans: (a: number, b: number) => [number, number, number, number];
    readonly tableread_columnFloats: (a: number, b: number) => [number, number, number, number];
    readonly tableread_columnIntegers: (a: number, b: number) => [number, number, number, number];
    readonly tableread_columnMissing: (a: number, b: number) => [number, number, number, number];
    readonly tableread_columnName: (a: number, b: number) => [number, number, number, number];
    readonly tableread_columnNumber: (a: number, b: number) => [number, number, number];
    readonly tableread_columnTexts: (a: number, b: number) => [number, number, number, number];
    readonly tableread_columnType: (a: number, b: number) => [number, number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_alloc: () => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
