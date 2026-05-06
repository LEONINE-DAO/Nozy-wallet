/* tslint:disable */
/* eslint-disable */

export function advance_orchard_witness_hex(witness_hex: string, block_json: string): string;

export function build_orchard_v5_tx_from_note(mnemonic_str: string, recipient_address: string, amount_zatoshis: bigint, fee_zatoshis: bigint, memo: string, spend_note_json: string, witness_json: string): any;

export function create_wallet(password: string): any;

export function decrypt_from_storage(encrypted: Uint8Array, password: string): Uint8Array;

export function encrypt_for_storage(data: Uint8Array, password: string): Uint8Array;

export function generate_address(mnemonic_str: string, account: number, index: number): string;

export function get_nu5_activation_height(): number;

export function get_zcash_chain_id(): string;

export function orchard_scan_tracker_apply_block(tracker_state_json: string, mnemonic_str: string, wallet_address: string, block_height: number, block_json: string): any;

/**
 * Pass empty string to start from an empty Orchard tree (only valid when scanning from chain genesis / NU5).
 */
export function orchard_scan_tracker_new(final_state_hex: string): string;

export function orchard_witness_matches_anchor_hex(witness_hex: string, anchor_hex: string): boolean;

export function prove_orchard_transaction_dummy(recipient_address: string, amount_zatoshis: bigint, memo: string): any;

export function prove_orchard_transaction_spend_from_note(mnemonic_str: string, recipient_address: string, amount_zatoshis: bigint, memo: string, spend_note_json: string, witness_json: string): any;

export function restore_wallet(mnemonic_str: string, password: string): any;

export function scan_orchard_actions(mnemonic_str: string, address: string, actions_json: string, block_height: number, txid: string): any;

export function sign_message(mnemonic_str: string, message: string): string;

export function unlock_wallet(encrypted_seed: Uint8Array, password: string): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly advance_orchard_witness_hex: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly build_orchard_v5_tx_from_note: (a: number, b: number, c: number, d: number, e: bigint, f: bigint, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number, number];
    readonly create_wallet: (a: number, b: number) => [number, number, number];
    readonly decrypt_from_storage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly encrypt_for_storage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly generate_address: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly get_nu5_activation_height: () => number;
    readonly get_zcash_chain_id: () => [number, number];
    readonly orchard_witness_matches_anchor_hex: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly prove_orchard_transaction_dummy: (a: number, b: number, c: bigint, d: number, e: number) => [number, number, number];
    readonly prove_orchard_transaction_spend_from_note: (a: number, b: number, c: number, d: number, e: bigint, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number];
    readonly restore_wallet: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly scan_orchard_actions: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number];
    readonly sign_message: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly unlock_wallet: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly orchard_scan_tracker_apply_block: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number];
    readonly orchard_scan_tracker_new: (a: number, b: number) => [number, number, number, number];
    readonly rustsecp256k1_v0_10_0_context_create: (a: number) => number;
    readonly rustsecp256k1_v0_10_0_context_destroy: (a: number) => void;
    readonly rustsecp256k1_v0_10_0_default_error_callback_fn: (a: number, b: number) => void;
    readonly rustsecp256k1_v0_10_0_default_illegal_callback_fn: (a: number, b: number) => void;
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
