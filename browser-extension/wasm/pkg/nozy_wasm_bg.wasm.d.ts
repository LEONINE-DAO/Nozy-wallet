/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const build_orchard_v5_tx_from_note: (a: number, b: number, c: number, d: number, e: bigint, f: bigint, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number, number];
export const create_wallet: (a: number, b: number) => [number, number, number];
export const decrypt_from_storage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
export const encrypt_for_storage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
export const generate_address: (a: number, b: number, c: number, d: number) => [number, number, number, number];
export const get_nu5_activation_height: () => number;
export const get_zcash_chain_id: () => [number, number];
export const prove_orchard_transaction_dummy: (a: number, b: number, c: bigint, d: number, e: number) => [number, number, number];
export const prove_orchard_transaction_spend_from_note: (a: number, b: number, c: number, d: number, e: bigint, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number];
export const restore_wallet: (a: number, b: number, c: number, d: number) => [number, number, number];
export const scan_orchard_actions: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number];
export const sign_message: (a: number, b: number, c: number, d: number) => [number, number, number, number];
export const unlock_wallet: (a: number, b: number, c: number, d: number) => [number, number, number];
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_exn_store: (a: number) => void;
export const __externref_table_alloc: () => number;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_start: () => void;
