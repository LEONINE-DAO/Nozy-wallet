/* tslint:disable */
/* eslint-disable */

export function advance_orchard_witness_hex(witness_hex: string, block_json: string): string;

export function build_orchard_v5_tx_from_note(mnemonic_str: string, recipient_address: string, amount_zatoshis: bigint, fee_zatoshis: bigint, memo: string, spend_note_json: string, witness_json: string): any;

/**
 * Nullifier the spend builder puts on chain. Scan used to store the action nullifier,
 * which is the note that funded this output, not this note.
 */
export function canonical_nullifier_hex(mnemonic_str: string, note_json: string): string;

export function create_wallet(password: string): any;

export function decrypt_from_storage(encrypted: Uint8Array, password: string): Uint8Array;

/**
 * Decrypt ZIP-302 memos from a `getrawtransaction` hex blob.
 *
 * Returns `[{ cmx, rho, value, memo }]` where `memo` is display text.
 * An empty `memo` means the output decrypted and the memo has no text.
 */
export function decrypt_memos_from_raw_tx(mnemonic: string, raw_tx_hex: string): any;

export function encrypt_for_storage(data: Uint8Array, password: string): Uint8Array;

/**
 * ZIP-317 fee in zatoshis using the pre-ZIP-2009 mainnet marginal fee.
 *
 * `priority` selects the 10× inclusion fee. Callers that know the chain tip should use
 * [`estimate_orchard_send_fee_zats_at`] or [`estimate_orchard_send_fee_zats_quoted`].
 */
export function estimate_orchard_send_fee_zats(memo: string, priority: boolean): bigint;

/**
 * ZIP-317 fee for the block the transaction is mined in.
 *
 * `mined_block_height` is chain tip + 1. Without a node quote, mainnet stays at 5,000
 * zats/action until height 3,590,000, then ZIP 2009's 1,000. Test networks use 1,000
 * at every height. `priority` selects the 10× inclusion fee.
 */
export function estimate_orchard_send_fee_zats_at(memo: string, priority: boolean, mainnet: boolean, mined_block_height: number): bigint;

/**
 * Same as [`estimate_orchard_send_fee_zats_at`], plus the `getstandardfee` marginal fee.
 *
 * `marginal_fee` is zatoshis per logical action. `0` keeps the local height schedule.
 */
export function estimate_orchard_send_fee_zats_quoted(memo: string, priority: boolean, mainnet: boolean, mined_block_height: number, marginal_fee: bigint): bigint;

/**
 * Build `nozy-vote-notes-v1` JSON from scanned Ironwood notes (cached witnesses).
 */
export function export_ironwood_vote_notes_json(mnemonic_str: string, notes_json: string, network: string): string;

export function generate_address(mnemonic_str: string, account: number, index: number): string;

export function get_nu5_activation_height(): number;

export function get_zcash_chain_id(): string;

export function nozy_version_display(): string;

/**
 * Root of a Zebra `finalState` commitment tree. This is the anchor the scan was built from.
 */
export function orchard_anchor_hex_from_final_state(final_state_hex: string): string;

export function orchard_scan_tracker_apply_block(tracker_state_json: string, mnemonic_str: string, wallet_address: string, block_height: number, block_json: string, account_indices_json: string): any;

export function orchard_scan_tracker_apply_blocks(tracker_state_json: string, mnemonic_str: string, wallet_address: string, blocks_json: string, account_indices_json: string): any;

export function orchard_scan_tracker_new(final_state_hex: string): string;

export function orchard_witness_matches_anchor_hex(witness_hex: string, anchor_hex: string): boolean;

export function pilot_expiry_delta_blocks(): number;

export function prove_orchard_transaction_dummy(recipient_address: string, amount_zatoshis: bigint, memo: string): any;

export function prove_orchard_transaction_spend_from_note(mnemonic_str: string, recipient_address: string, amount_zatoshis: bigint, memo: string, spend_note_json: string, witness_json: string): any;

export function restore_wallet(mnemonic_str: string, password: string): any;

export function scan_orchard_actions(mnemonic_str: string, address: string, actions_json: string, block_height: number, txid: string): any;

/**
 * Initialize dual-pool trackers from Zebrad `z_gettreestate` final states.
 */
export function shielded_scan_tracker_new(orchard_final_state_hex: string, ironwood_final_state_hex: string): string;

export function sign_message(mnemonic_str: string, message: string): string;

export function sign_vote_delegation(mnemonic_str: string, request_json: string): string;

export function unlock_wallet(encrypted_seed: Uint8Array, password: string): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly canonical_nullifier_hex: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly advance_orchard_witness_hex: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly orchard_anchor_hex_from_final_state: (a: number, b: number) => [number, number, number, number];
    readonly orchard_witness_matches_anchor_hex: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly create_wallet: (a: number, b: number) => [number, number, number];
    readonly restore_wallet: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly unlock_wallet: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly generate_address: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly get_zcash_chain_id: () => [number, number];
    readonly get_nu5_activation_height: () => number;
    readonly sign_message: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly scan_orchard_actions: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number, number];
    readonly prove_orchard_transaction_dummy: (a: number, b: number, c: bigint, d: number, e: number) => [number, number, number];
    readonly prove_orchard_transaction_spend_from_note: (a: number, b: number, c: number, d: number, e: bigint, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number];
    readonly build_orchard_v5_tx_from_note: (a: number, b: number, c: number, d: number, e: bigint, f: bigint, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number, number];
    readonly estimate_orchard_send_fee_zats: (a: number, b: number, c: number) => bigint;
    readonly estimate_orchard_send_fee_zats_at: (a: number, b: number, c: number, d: number, e: number) => bigint;
    readonly estimate_orchard_send_fee_zats_quoted: (a: number, b: number, c: number, d: number, e: number, f: bigint) => bigint;
    readonly pilot_expiry_delta_blocks: () => number;
    readonly nozy_version_display: () => [number, number];
    readonly encrypt_for_storage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly decrypt_from_storage: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly orchard_scan_tracker_apply_blocks: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number];
    readonly orchard_scan_tracker_new: (a: number, b: number) => [number, number, number, number];
    readonly shielded_scan_tracker_new: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly orchard_scan_tracker_apply_block: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number];
    readonly decrypt_memos_from_raw_tx: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly sign_vote_delegation: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly export_ironwood_vote_notes_json: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
    readonly rustsecp256k1_v0_10_0_context_create: (a: number) => number;
    readonly rustsecp256k1_v0_10_0_context_destroy: (a: number) => void;
    readonly rustsecp256k1_v0_10_0_default_illegal_callback_fn: (a: number, b: number) => void;
    readonly rustsecp256k1_v0_10_0_default_error_callback_fn: (a: number, b: number) => void;
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
