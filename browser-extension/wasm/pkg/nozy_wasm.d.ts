/* tslint:disable */
/* eslint-disable */

export function create_wallet(password: string): any;

export function decrypt_from_storage(encrypted: Uint8Array, password: string): Uint8Array;

export function encrypt_for_storage(data: Uint8Array, password: string): Uint8Array;

export function generate_address(mnemonic_str: string, account: number, index: number): string;

export function get_nu5_activation_height(): number;

export function get_zcash_chain_id(): string;

export function restore_wallet(mnemonic_str: string, password: string): any;

export function sign_message(mnemonic_str: string, message: string): string;

export function unlock_wallet(encrypted_seed: Uint8Array, password: string): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly create_wallet: (a: number, b: number) => [number, number, number];
    readonly decrypt_from_storage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly encrypt_for_storage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly generate_address: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly get_nu5_activation_height: () => number;
    readonly get_zcash_chain_id: () => [number, number];
    readonly restore_wallet: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly sign_message: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly unlock_wallet: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
