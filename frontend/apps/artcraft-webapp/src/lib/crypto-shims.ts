// NB: `crypto.randomUUID` is only exposed in secure contexts (HTTPS or
// localhost). ArtCraft self-host/dev instances are commonly served over
// plain-HTTP LAN hosts (e.g. http://k7:4201), where that method is missing and
// every batch/upload call crashes with
// "TypeError: crypto.randomUUID is not a function".
//
// This imported-for-side-effects shim fills it in with a UUID v4 synthesised
// from `crypto.getRandomValues`, which IS available in insecure contexts.
// Import it before any business logic (see main.tsx).

type RandomUuid = () => `${string}-${string}-${string}-${string}-${string}`;

function uuidV4FromRandomValues(): `${string}-${string}-${string}-${string}-${string}` {
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
  bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant 10xxxxxx
  const hex = (byte: number) => byte.toString(16).padStart(2, "0");
  return [
    hex(bytes[0]) + hex(bytes[1]) + hex(bytes[2]) + hex(bytes[3]),
    hex(bytes[4]) + hex(bytes[5]),
    hex(bytes[6]) + hex(bytes[7]),
    hex(bytes[8]) + hex(bytes[9]),
    hex(bytes[10]) + hex(bytes[11]) + hex(bytes[12]) + hex(bytes[13])
      + hex(bytes[14]) + hex(bytes[15]),
  ].join("-") as `${string}-${string}-${string}-${string}-${string}`;
}

if (typeof crypto !== "undefined") {
  const shim = crypto as Crypto & { randomUUID?: RandomUuid };
  if (typeof shim.randomUUID !== "function") {
    shim.randomUUID = uuidV4FromRandomValues;
  }
}
