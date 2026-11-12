/**
 * Catch-up sync has to keep the unlocked session. Auto-lock wipes the seed and
 * stops the scan, so an idle popout or full-page dashboard would abandon a
 * sync before it finishes. Tip-follow ("watching") still auto-locks.
 *
 * @param {string | null | undefined} status
 * @returns {boolean}
 */
export function scanStatusDefersAutoLock(status) {
  return status === "scanning";
}
