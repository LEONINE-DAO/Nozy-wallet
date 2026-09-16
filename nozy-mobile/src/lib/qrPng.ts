import QRCode from "qrcode";

function crc32(buf: Uint8Array): number {
  let c = ~0;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
  }
  return ~c >>> 0;
}

function adler32(buf: Uint8Array): number {
  let a = 1;
  let b = 0;
  for (let i = 0; i < buf.length; i++) {
    a = (a + buf[i]) % 65521;
    b = (b + a) % 65521;
  }
  return ((b << 16) | a) >>> 0;
}

function u32(n: number): Uint8Array {
  return Uint8Array.of((n >>> 24) & 255, (n >>> 16) & 255, (n >>> 8) & 255, n & 255);
}

function concat(parts: Uint8Array[]): Uint8Array {
  const total = parts.reduce((n, p) => n + p.length, 0);
  const out = new Uint8Array(total);
  let off = 0;
  for (const p of parts) {
    out.set(p, off);
    off += p.length;
  }
  return out;
}

function chunk(type: string, data: Uint8Array): Uint8Array {
  const t = Uint8Array.from(type, (ch) => ch.charCodeAt(0));
  const body = concat([t, data]);
  return concat([u32(data.length), body, u32(crc32(body))]);
}

function zlibStore(raw: Uint8Array): Uint8Array {
  const blocks: Uint8Array[] = [Uint8Array.of(0x78, 0x01)];
  let off = 0;
  while (off < raw.length) {
    const n = Math.min(65535, raw.length - off);
    const last = off + n >= raw.length ? 1 : 0;
    const ninv = (~n) & 0xffff;
    const header = Uint8Array.of(
      last,
      n & 255,
      (n >> 8) & 255,
      ninv & 255,
      (ninv >> 8) & 255,
    );
    blocks.push(header, raw.subarray(off, off + n));
    off += n;
  }
  blocks.push(u32(adler32(raw)));
  return concat(blocks);
}

function bytesToBase64(bytes: Uint8Array): string {
  const chars =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  let out = "";
  for (let i = 0; i < bytes.length; i += 3) {
    const a = bytes[i];
    const b = i + 1 < bytes.length ? bytes[i + 1] : 0;
    const c = i + 2 < bytes.length ? bytes[i + 2] : 0;
    const triple = (a << 16) | (b << 8) | c;
    out += chars[(triple >> 18) & 63];
    out += chars[(triple >> 12) & 63];
    out += i + 1 < bytes.length ? chars[(triple >> 6) & 63] : "=";
    out += i + 2 < bytes.length ? chars[triple & 63] : "=";
  }
  return out;
}

/** Pure-JS QR PNG for React Native (no canvas / no native SVG). */
export function addressQrDataUrl(text: string): string {
  const qr = QRCode.create(text, { errorCorrectionLevel: "M" });
  const modules = qr.modules;
  if (!modules) throw new Error("QR encode failed");
  const n = modules.size;
  const margin = 2;
  const scale = 6;
  const dim = (n + margin * 2) * scale;
  const raw = new Uint8Array(dim * (dim + 1));
  for (let y = 0; y < dim; y++) {
    const row = y * (dim + 1);
    raw[row] = 0;
    const my = Math.floor(y / scale) - margin;
    for (let x = 0; x < dim; x++) {
      const mx = Math.floor(x / scale) - margin;
      const dark =
        mx >= 0 && my >= 0 && mx < n && my < n && modules.get(mx, my);
      raw[row + 1 + x] = dark ? 0x11 : 0xff;
    }
  }
  const ihdr = concat([
    u32(dim),
    u32(dim),
    Uint8Array.of(8, 0, 0, 0, 0),
  ]);
  const png = concat([
    Uint8Array.of(137, 80, 78, 71, 13, 10, 26, 10),
    chunk("IHDR", ihdr),
    chunk("IDAT", zlibStore(raw)),
    chunk("IEND", new Uint8Array()),
  ]);
  return `data:image/png;base64,${bytesToBase64(png)}`;
}
