/* tslint:disable */
/* eslint-disable */

/**
 * What `readXlsx` gives: the sheet read, or the code of a refusal with the
 * fields its words need. It stays in the memory of the wasm, each field
 * read is a copy, and `free()` releases it.
 */
export class XlsxRead {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Row after row: null, a string, a number or a boolean; empty for a
     * refusal. Each read of it makes a new array.
     */
    readonly cells: any[];
    /**
     * The text of the error, for "cellError"; "" otherwise.
     */
    readonly detail: string;
    /**
     * The first column of the rectangle, column A being 1; 0 when there
     * is no rectangle.
     */
    readonly firstColumn: number;
    /**
     * The first row of the rectangle of the sheet read, or of
     * "sheetTooLarge", as Excel numbers the rows, from 1; 0 otherwise.
     */
    readonly firstRow: number;
    /**
     * The number of columns of the rectangle; 0 when there is none.
     */
    readonly numColumns: number;
    /**
     * The number of rows of the rectangle; 0 when there is none.
     */
    readonly numRows: number;
    /**
     * "" for a sheet read; otherwise "notXlsx", "oldExcel",
     * "encrypted", "emptySheet", "cellError" or "sheetTooLarge".
     */
    readonly refusal: string;
    /**
     * The name of the sheet, for a sheet read, "emptySheet" and
     * "sheetTooLarge"; "" otherwise.
     */
    readonly sheet: string;
}

/**
 * Reads the first worksheet that is not hidden of the xlsx `bytes`, or
 * refuses it; one reason is a rectangle of the values larger than
 * `max_cells` cells.
 *
 * # Errors
 *
 * Throws an `Error` for a file that cannot be read, with the message of
 * the zip crate, of calamine or of xlsx_rs, whichever failed.
 */
export function readXlsx(bytes: Uint8Array, max_cells: number): XlsxRead;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_xlsxread_cells: (a: number) => [number, number];
    readonly __wbg_get_xlsxread_detail: (a: number) => [number, number];
    readonly __wbg_get_xlsxread_firstColumn: (a: number) => number;
    readonly __wbg_get_xlsxread_firstRow: (a: number) => number;
    readonly __wbg_get_xlsxread_numColumns: (a: number) => number;
    readonly __wbg_get_xlsxread_numRows: (a: number) => number;
    readonly __wbg_get_xlsxread_refusal: (a: number) => [number, number];
    readonly __wbg_get_xlsxread_sheet: (a: number) => [number, number];
    readonly __wbg_xlsxread_free: (a: number, b: number) => void;
    readonly readXlsx: (a: number, b: number, c: number) => [number, number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_drop_slice: (a: number, b: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
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
