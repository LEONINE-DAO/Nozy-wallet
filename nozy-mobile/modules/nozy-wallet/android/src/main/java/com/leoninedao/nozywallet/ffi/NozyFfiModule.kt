package com.leoninedao.nozywallet.ffi

import android.os.StatFs
import expo.modules.kotlin.exception.CodedException
import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import java.io.File

class NozyModuleException(message: String) : CodedException(message)

private const val NU5_ORCHARD_MAINNET = 1_687_104L
private const val MIN_FREE_BYTES = 256L * 1024L * 1024L

class NozyFfiModule : Module() {
  override fun definition() = ModuleDefinition {
    Name("NozyFfi")

    Function("nativeLibReady") {
      true
    }

    Function("walletPaths") {
      val ctx =
        appContext.reactContext
          ?: throw NozyModuleException("Android context unavailable")
      val dir = File(ctx.filesDir, "nozy")
      dir.mkdirs()
      mapOf(
        "walletDataDir" to dir.absolutePath,
        "compactDbPath" to File(dir, "lwd_compact.sqlite").absolutePath,
      )
    }

    Function("lockWallet") { }

    AsyncFunction("generateMnemonic") {
      wrap { uniffi.nozy_ffi.generateMnemonic() }
    }

    AsyncFunction("validateMnemonic") { mnemonic: String ->
      wrap { uniffi.nozy_ffi.validateMnemonic(mnemonic) }
    }

    AsyncFunction("orchardUnifiedAddress") { mnemonic: String, account: Int ->
      wrap {
        val ua = uniffi.nozy_ffi.orchardUnifiedAddress(mnemonic, account.toUInt())
        mapOf("address" to ua.address, "network" to ua.network)
      }
    }

    AsyncFunction("lwdGetInfo") { lightwalletdUrl: String ->
      wrap {
        val info = uniffi.nozy_ffi.lwdGetInfo(lightwalletdUrl)
        mapOf(
          "version" to info.version,
          "chain_name" to info.chainName,
          "block_height" to info.blockHeight.toLong(),
          "estimated_height" to info.estimatedHeight.toLong(),
        )
      }
    }

    AsyncFunction("lwdSyncCompactToTip") { lightwalletdUrl: String, compactDbPath: String, startFloor: Double? ->
      wrap {
        val rawFloor = startFloor?.toLong() ?: 0L
        val floor = if (rawFloor >= NU5_ORCHARD_MAINNET) rawFloor else NU5_ORCHARD_MAINNET
        val db = File(compactDbPath)
        val parent = db.parentFile ?: db
        parent.mkdirs()
        val avail = try {
          StatFs(parent.absolutePath).availableBytes
        } catch (_: Exception) {
          -1L
        }
        if (avail in 0 until MIN_FREE_BYTES && db.exists()) {
          db.delete()
          File("$compactDbPath-wal").delete()
          File("$compactDbPath-shm").delete()
        }
        uniffi.nozy_ffi.lwdSyncCompactToTip(lightwalletdUrl, compactDbPath, floor.toULong()).toLong()
      }
    }

    AsyncFunction("lwdGetLatestTreeState") { lightwalletdUrl: String ->
      wrap {
        val ts = uniffi.nozy_ffi.lwdGetLatestTreeState(lightwalletdUrl)
        mapOf(
          "network" to ts.network,
          "height" to ts.height.toLong(),
          "hash" to ts.hash,
          "time" to ts.time.toInt(),
          "sapling_tree" to ts.saplingTree,
          "orchard_tree" to ts.orchardTree,
        )
      }
    }

    AsyncFunction("lwdSendTransaction") { lightwalletdUrl: String, rawTxHex: String ->
      wrap { uniffi.nozy_ffi.lwdSendTransaction(lightwalletdUrl, rawTxHex) }
    }

    AsyncFunction("saplingStatus") { walletDataDir: String ->
      wrap {
        val s = uniffi.nozy_ffi.saplingStatus(walletDataDir)
        mapOf(
          "unspent_notes" to s.unspentNotes.toLong(),
          "with_rseed" to s.withRseed.toLong(),
          "ready_to_shield" to s.readyToShield.toLong(),
          "unspent_zatoshis" to s.unspentZatoshis.toLong(),
          "unspent_zec" to s.unspentZec,
          "fee_zatoshis" to s.feeZatoshis.toLong(),
          "fee_zec" to s.feeZec,
          "has_legacy_balance" to s.hasLegacyBalance,
          "message" to s.message,
        )
      }
    }

    AsyncFunction("saplingScan") { mnemonic: String, walletDataDir: String, compactDbPath: String, startFloor: Double?, full: Boolean ->
      wrap {
        val floor = startFloor?.toLong()?.takeIf { it >= 0 }?.toULong()
        val s = uniffi.nozy_ffi.saplingScan(mnemonic, walletDataDir, compactDbPath, floor, full)
        mapOf(
          "blocks_scanned" to s.blocksScanned.toLong(),
          "outputs_seen" to s.outputsSeen.toLong(),
          "notes_discovered" to s.notesDiscovered.toLong(),
          "notes_marked_spent" to s.notesMarkedSpent.toLong(),
          "range_start" to s.rangeStart.toLong(),
          "range_end" to s.rangeEnd.toLong(),
          "unspent_zatoshis" to s.unspentZatoshis.toLong(),
          "unspent_notes" to s.unspentNotes.toLong(),
          "message" to s.message,
        )
      }
    }

    AsyncFunction("orchardScan") { mnemonic: String, walletDataDir: String, compactDbPath: String, startFloor: Double?, maxAccount: Int, full: Boolean ->
      wrap {
        val floor = startFloor?.toLong()?.takeIf { it >= 0 }?.toULong()
        val s = uniffi.nozy_ffi.orchardScan(
          mnemonic,
          walletDataDir,
          compactDbPath,
          floor,
          maxAccount.toUInt(),
          full,
        )
        mapOf(
          "blocks_scanned" to s.blocksScanned.toLong(),
          "actions_seen" to s.actionsSeen.toLong(),
          "orchard_actions_seen" to s.orchardActionsSeen.toLong(),
          "ironwood_actions_seen" to s.ironwoodActionsSeen.toLong(),
          "notes_discovered" to s.notesDiscovered.toLong(),
          "orchard_notes_discovered" to s.orchardNotesDiscovered.toLong(),
          "ironwood_notes_discovered" to s.ironwoodNotesDiscovered.toLong(),
          "notes_marked_spent" to s.notesMarkedSpent.toLong(),
          "range_start" to s.rangeStart.toLong(),
          "range_end" to s.rangeEnd.toLong(),
          "unspent_zatoshis" to s.unspentZatoshis.toLong(),
          "unspent_zec" to s.unspentZec,
          "unspent_notes" to s.unspentNotes.toLong(),
          "message" to s.message,
        )
      }
    }

    AsyncFunction("orchardStatus") { walletDataDir: String ->
      wrap {
        val s = uniffi.nozy_ffi.orchardStatus(walletDataDir)
        mapOf(
          "unspent_zatoshis" to s.unspentZatoshis.toLong(),
          "unspent_zec" to s.unspentZec,
          "unspent_notes" to s.unspentNotes.toLong(),
          "notes" to s.notes.map { n ->
            mapOf(
              "txid" to n.txid,
              "block_height" to n.blockHeight.toInt(),
              "value_zec" to n.valueZec,
              "spent" to n.spent,
              "pool" to n.pool,
            )
          },
          "message" to s.message,
        )
      }
    }

    AsyncFunction("saplingShield") { mnemonic: String, walletDataDir: String, compactDbPath: String, zebraUrl: String, lightwalletdUrl: String, dryRun: Boolean, noBroadcast: Boolean ->
      wrap {
        val s = uniffi.nozy_ffi.saplingShield(
          mnemonic,
          walletDataDir,
          compactDbPath,
          zebraUrl,
          lightwalletdUrl,
          dryRun,
          noBroadcast,
        )
        mapOf(
          "dry_run" to s.dryRun,
          "broadcast" to s.broadcast,
          "txid" to s.txid,
          "shielded_value_zatoshis" to s.shieldedValueZatoshis?.toLong(),
          "fee_zatoshis" to s.feeZatoshis.toLong(),
          "expiry_height" to s.expiryHeight?.toInt(),
          "candidate_notes" to s.candidateNotes.toLong(),
          "candidate_zatoshis" to s.candidateZatoshis.toLong(),
          "message" to s.message,
        )
      }
    }

    AsyncFunction("voteCalendarInfo") {
      wrap {
        val c = uniffi.nozy_ffi.voteCalendarInfo()
        mapOf(
          "snapshot_utc" to c.snapshotUtc,
          "vote_start_utc" to c.voteStartUtc,
          "vote_end_utc" to c.voteEndUtc,
          "forum_url" to c.forumUrl,
          "tally_url" to c.tallyUrl,
          "message" to c.message,
        )
      }
    }

    AsyncFunction("voteExportNotes") { mnemonic: String, walletDataDir: String, network: String ->
      wrap {
        val v = uniffi.nozy_ffi.voteExportNotes(mnemonic, walletDataDir, network)
        mapOf(
          "format" to v.format,
          "network" to v.network,
          "note_count" to v.noteCount.toLong(),
          "total_value_zat" to v.totalValueZat.toLong(),
          "seed_fingerprint_hex" to v.seedFingerprintHex,
          "notes_json" to v.notesJson,
          "message" to v.message,
        )
      }
    }

    AsyncFunction("voteSignDelegation") { mnemonic: String, requestJson: String ->
      wrap {
        val v = uniffi.nozy_ffi.voteSignDelegation(mnemonic, requestJson)
        mapOf(
          "format" to v.format,
          "round_id" to v.roundId,
          "bundle_index" to v.bundleIndex.toInt(),
          "sighash_hex" to v.sighashHex,
          "spend_auth_sig_hex" to v.spendAuthSigHex,
          "sig_json" to v.sigJson,
          "message" to v.message,
        )
      }
    }
  }

  private inline fun <T> wrap(block: () -> T): T {
    try {
      return block()
    } catch (e: Exception) {
      val combined = listOfNotNull(e.message, e.cause?.message).joinToString(" | ")
      if (combined.contains("ENOSPC") || combined.contains("No space", ignoreCase = true)) {
        throw NozyModuleException(
          "Phone storage is full. Compact cache was filling from genesis; free space and sync from wallet birthday.",
        )
      }
      throw NozyModuleException(e.message ?: "nozy-ffi error")
    }
  }
}
