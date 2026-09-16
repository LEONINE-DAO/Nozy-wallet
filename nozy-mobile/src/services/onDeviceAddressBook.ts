import AsyncStorage from "@react-native-async-storage/async-storage";
import type { AddressBookEntry } from "../types";

const STORAGE_KEY = "nozy.ondevice.addressbook";

function isShieldedAddress(address: string): boolean {
  const a = address.trim().toLowerCase();
  return (
    a.startsWith("u1") ||
    a.startsWith("utest1") ||
    a.startsWith("zs1") ||
    a.startsWith("ztestsapling")
  );
}

async function readAll(): Promise<AddressBookEntry[]> {
  const raw = await AsyncStorage.getItem(STORAGE_KEY);
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw) as AddressBookEntry[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

async function writeAll(entries: AddressBookEntry[]): Promise<void> {
  await AsyncStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
}

export async function listOnDeviceAddressBook(): Promise<AddressBookEntry[]> {
  const entries = await readAll();
  return [...entries].sort((a, b) => a.name.localeCompare(b.name));
}

export async function addOnDeviceAddressBookEntry(
  name: string,
  address: string,
  notes?: string,
): Promise<AddressBookEntry> {
  const trimmedName = name.trim();
  const trimmedAddress = address.trim();
  if (!trimmedName || !trimmedAddress) {
    throw new Error("Name and address are required");
  }
  if (!isShieldedAddress(trimmedAddress)) {
    throw new Error("Use a shielded address (u1 or zs1)");
  }
  const entries = await readAll();
  if (entries.some((e) => e.name.toLowerCase() === trimmedName.toLowerCase())) {
    throw new Error("A contact with that name already exists");
  }
  const now = new Date().toISOString();
  const entry: AddressBookEntry = {
    name: trimmedName,
    address: trimmedAddress,
    created_at: now,
    last_used: null,
    usage_count: 0,
    notes: notes?.trim() || null,
  };
  entries.push(entry);
  await writeAll(entries);
  return entry;
}

export async function removeOnDeviceAddressBookEntry(name: string): Promise<void> {
  const entries = await readAll();
  await writeAll(entries.filter((e) => e.name !== name));
}
