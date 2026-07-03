import { useEffect, useState } from "react";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { walletApi } from "../lib/api";
import type { AddressBookEntry } from "../lib/types";
import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../utils/errors";
import { Copy, User } from "@solar-icons/react";

export function ContactsPage() {
  const [entries, setEntries] = useState<AddressBookEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [showAddForm, setShowAddForm] = useState(false);
  const [addName, setAddName] = useState("");
  const [addAddress, setAddAddress] = useState("");
  const [addNotes, setAddNotes] = useState("");
  const [addSaving, setAddSaving] = useState(false);
  const [removeName, setRemoveName] = useState<string | null>(null);

  const loadList = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await walletApi.listAddressBook();
      setEntries(Array.isArray(res?.data) ? res.data : []);
    } catch (e) {
      setError(formatErrorForDisplay(e, "Failed to load contacts"));
      setEntries([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadList();
  }, []);

  const filtered = searchQuery.trim()
    ? entries.filter(
        (e) =>
          e.name.toLowerCase().includes(searchQuery.trim().toLowerCase()) ||
          e.address.toLowerCase().includes(searchQuery.trim().toLowerCase()) ||
          (e.notes ?? "").toLowerCase().includes(searchQuery.trim().toLowerCase())
      )
    : entries;

  const resetAddForm = () => {
    setAddName("");
    setAddAddress("");
    setAddNotes("");
  };

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    const name = addName.trim();
    const address = addAddress.trim();
    if (!name || !address) {
      toast.error("Name and address are required");
      return;
    }
    if (!address.startsWith("u1") && !address.startsWith("zs1")) {
      toast.error("Address must be a shielded address (u1 or zs1)");
      return;
    }
    setAddSaving(true);
    try {
      await walletApi.addAddressBookEntry({
        name,
        address,
        notes: addNotes.trim() || undefined,
      });
      toast.success("Contact added");
      resetAddForm();
      setShowAddForm(false);
      await loadList();
    } catch (err) {
      toast.error(formatErrorForDisplay(err, "Failed to add contact"));
    } finally {
      setAddSaving(false);
    }
  };

  const handleRemove = async (name: string) => {
    if (!confirm(`Remove "${name}" from contacts?`)) return;
    setRemoveName(name);
    try {
      await walletApi.removeAddressBookEntry(name);
      toast.success("Contact removed");
      loadList();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to remove contact"));
    } finally {
      setRemoveName(null);
    }
  };

  const handleCopy = (address: string) => {
    navigator.clipboard.writeText(address);
    toast.success("Address copied");
  };

  return (
    <div className="space-y-6 animate-fade-in max-w-4xl mx-auto text-left">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <h2 className="text-3xl font-bold text-gray-900">Contacts</h2>
        <Button
          type="button"
          onClick={() => {
            if (showAddForm) {
              resetAddForm();
              setShowAddForm(false);
            } else {
              setShowAddForm(true);
            }
          }}
          className="gap-2"
        >
          {showAddForm ? "Cancel" : "Add contact"}
        </Button>
      </div>

      {showAddForm && (
        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Add contact</h3>
          <form className="space-y-4" onSubmit={handleAdd}>
            <Input
              label="Name"
              placeholder="e.g. Exchange"
              value={addName}
              onChange={(e) => setAddName(e.target.value)}
              autoComplete="off"
              required
            />
            <Input
              label="Address (u1 or zs1)"
              placeholder="Shielded address"
              value={addAddress}
              onChange={(e) => setAddAddress(e.target.value)}
              autoComplete="off"
              spellCheck={false}
              required
            />
            <Input
              label="Notes (optional)"
              placeholder="e.g. Withdrawal"
              value={addNotes}
              onChange={(e) => setAddNotes(e.target.value)}
              autoComplete="off"
            />
            <p className="text-xs text-gray-500">
              Paste a shielded Zcash address starting with <span className="font-mono">u1</span> or{" "}
              <span className="font-mono">zs1</span>.
            </p>
            <div className="flex gap-2 justify-end pt-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  resetAddForm();
                  setShowAddForm(false);
                }}
                disabled={addSaving}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={addSaving}>
                {addSaving ? "Saving…" : "Save contact"}
              </Button>
            </div>
          </form>
        </div>
      )}

      {!loading && !error && entries.length > 0 && (
        <div className="max-w-md">
          <Input
            type="search"
            placeholder="Search by name, address, or notes"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            aria-label="Search contacts"
            className="bg-white/60 border-white/50"
          />
        </div>
      )}

      <div className="bg-white/60 backdrop-blur-md rounded-2xl border border-white/50 shadow-sm overflow-hidden">
        {loading ? (
          <div className="p-12 flex items-center justify-center gap-2 text-gray-600">
            <div className="w-6 h-6 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
            <span>Loading contacts…</span>
          </div>
        ) : error ? (
          <div className="p-12 text-center">
            <p className="text-red-600 mb-2">{error}</p>
            <p className="text-sm text-gray-500">
              Restart the desktop app after updating so address book commands are available.
            </p>
            <Button variant="outline" onClick={loadList} className="mt-4">
              Retry
            </Button>
          </div>
        ) : entries.length === 0 ? (
          <div className="p-12 text-center text-gray-500">
            <p>No contacts yet</p>
            <p className="text-sm mt-1">Add addresses to quickly reuse when sending.</p>
            {!showAddForm && (
              <Button type="button" onClick={() => setShowAddForm(true)} className="mt-4">
                Add contact
              </Button>
            )}
          </div>
        ) : filtered.length === 0 ? (
          <div className="p-12 text-center text-gray-500">
            <p>No contacts match your search</p>
          </div>
        ) : (
          <ul className="divide-y divide-gray-100/50">
            {filtered.map((entry) => (
              <li
                key={entry.name}
                className="p-4 hover:bg-white/40 transition-colors flex items-center justify-between gap-4 group"
              >
                <div className="flex items-center gap-4 min-w-0 flex-1">
                  <div className="w-10 h-10 rounded-full bg-primary/10 text-primary flex items-center justify-center shrink-0">
                    <User size={20} />
                  </div>
                  <div className="min-w-0">
                    <p className="font-semibold text-gray-900 truncate">{entry.name}</p>
                    <p
                      className="text-sm font-mono text-gray-500 truncate cursor-pointer hover:text-gray-700"
                      title={entry.address}
                      onClick={() => handleCopy(entry.address)}
                    >
                      {entry.address}
                    </p>
                    {entry.notes && (
                      <p className="text-xs text-gray-500 mt-0.5 truncate">{entry.notes}</p>
                    )}
                  </div>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  <button
                    type="button"
                    onClick={() => handleCopy(entry.address)}
                    className="p-2 rounded-lg text-gray-500 hover:bg-gray-100 hover:text-gray-700 transition-colors"
                    title="Copy address"
                  >
                    <Copy size={18} />
                  </button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-red-600 hover:bg-red-50 hover:text-red-700"
                    onClick={() => handleRemove(entry.name)}
                    disabled={removeName === entry.name}
                  >
                    {removeName === entry.name ? "Removing…" : "Remove"}
                  </Button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
