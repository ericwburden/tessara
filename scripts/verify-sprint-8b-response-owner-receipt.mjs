import { createHash, createPrivateKey, createPublicKey, sign, verify } from "node:crypto";

function decodeBase64Url(value, label) {
  if (typeof value !== "string" || !/^[A-Za-z0-9_-]+$/.test(value)) {
    throw new Error(`${label} is not unpadded base64url.`);
  }
  return Buffer.from(value, "base64url");
}

function canonicalize(value) {
  if (value === null || typeof value === "boolean" || typeof value === "string") {
    return JSON.stringify(value);
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) throw new Error("JCS input contains a non-finite number.");
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(canonicalize).join(",")}]`;
  if (typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonicalize(value[key])}`
    ).join(",")}}`;
  }
  throw new Error(`JCS input contains unsupported type '${typeof value}'.`);
}

function parseArguments(argv) {
  const result = {};
  for (let index = 0; index < argv.length; index += 2) {
    const name = argv[index];
    const value = argv[index + 1];
    if (!name?.startsWith("--") || value === undefined) {
      throw new Error("Expected paired --name value arguments.");
    }
    result[name.slice(2)] = value;
  }
  return result;
}

async function readStdin() {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  return Buffer.concat(chunks).toString("utf8");
}

function verifyEnvelope(envelope, encodedPublicKey) {
  const expectedKeys = ["issuer", "key_id", "payload", "purpose", "schema_version", "signature"];
  if (Object.keys(envelope).sort().join("\n") !== expectedKeys.join("\n") ||
      envelope.schema_version !== 1 || envelope.issuer !== "tessara.core" ||
      envelope.key_id !== "core-development-v1" ||
      envelope.purpose !== "response_owner_action_receipt") {
    throw new Error("Response owner receipt envelope identity or property set is invalid.");
  }
  const publicKey = decodeBase64Url(encodedPublicKey, "public key");
  const signature = decodeBase64Url(envelope.signature, "signature");
  if (publicKey.length !== 32 || signature.length !== 64) {
    throw new Error("Response owner receipt key or signature has an invalid length.");
  }
  const signingInput = {
    schema_version: envelope.schema_version,
    issuer: envelope.issuer,
    key_id: envelope.key_id,
    purpose: envelope.purpose,
    payload: envelope.payload,
  };
  const canonical = Buffer.from(canonicalize(signingInput), "utf8");
  const spki = Buffer.concat([
    Buffer.from("302a300506032b6570032100", "hex"),
    publicKey,
  ]);
  if (!verify(
    null,
    canonical,
    createPublicKey({ key: spki, format: "der", type: "spki" }),
    signature,
  )) {
    throw new Error("Response owner receipt signature is invalid.");
  }
  return {
    schema_version: 1,
    proof: "response-owner-receipt-ed25519-verification",
    state: "passed",
    signing_input_sha256: createHash("sha256").update(canonical).digest("hex"),
  };
}

if (process.argv.slice(2).length === 1 && process.argv[2] === "--self-test") {
  const seed = Buffer.alloc(32, 12);
  const privateKey = createPrivateKey({
    key: Buffer.concat([Buffer.from("302e020100300506032b657004220420", "hex"), seed]),
    format: "der",
    type: "pkcs8",
  });
  const payload = { schema_version: 1, logical_key: "response.self-test", action: "create" };
  const signingInput = {
    schema_version: 1,
    issuer: "tessara.core",
    key_id: "core-development-v1",
    purpose: "response_owner_action_receipt",
    payload,
  };
  const envelope = {
    ...signingInput,
    signature: sign(null, Buffer.from(canonicalize(signingInput), "utf8"), privateKey)
      .toString("base64url"),
  };
  const publicKey = "C1E62bSSQBXKCQLtB5BE06xdvsIwbwaUjBDajrbjny0";
  verifyEnvelope(envelope, publicKey);
  let tamperRejected = false;
  try {
    verifyEnvelope({ ...envelope, payload: { ...payload, action: "delete" } }, publicKey);
  } catch {
    tamperRejected = true;
  }
  if (!tamperRejected) throw new Error("Verifier self-test accepted a tampered receipt.");
  process.stdout.write(`${JSON.stringify({
    schema_version: 1,
    proof: "response-owner-receipt-verifier-self-test",
    state: "passed",
    tamper_rejected: true,
  })}\n`);
} else {
  const args = parseArguments(process.argv.slice(2));
  if (!args["public-key"]) throw new Error("--public-key is required.");
  const envelope = JSON.parse(await readStdin());
  process.stdout.write(`${JSON.stringify(verifyEnvelope(envelope, args["public-key"]))}\n`);
}
