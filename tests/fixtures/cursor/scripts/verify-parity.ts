import { create, toBinary, fromBinary } from "@bufbuild/protobuf";
import {
  AgentClientMessageSchema,
  AgentRunRequestSchema,
  ConversationActionSchema,
  UserMessageActionSchema,
  UserMessageSchema,
  ModelDetailsSchema,
  RequestContextSchema,
  RequestContextEnvSchema,
} from "../../../../../unified-model-proxy/src/lib/cursor/proto/agent_pb.ts";
import * as fs from "fs";
import * as path from "path";

const rustBinaryPath = path.join(__dirname, "../run/basic_system_user.bin");

console.log("Loading Rust golden binary from:", rustBinaryPath);
if (!fs.existsSync(rustBinaryPath)) {
  console.error("Rust binary fixture not found. Make sure to run UMP_REGENERATE_CURSOR_FIXTURES=1 cargo test first.");
  process.exit(1);
}

const rustBytes = fs.readFileSync(rustBinaryPath);

// Construct the same payload as the Rust unit test
const prompt = "System: You are a helpful assistant.\nUser: Explain quantum entanglement in one paragraph.\n";

const userMessage = create(UserMessageSchema, {
  text: prompt,
  messageId: "msg-fixture-0001",
  mode: 1,
});

const requestContextEnv = create(RequestContextEnvSchema, {
  osVersion: "darwin-24.6.0",
  workspacePaths: ["/tmp/cursor-fixture-workspace"],
  shell: "/bin/zsh",
  projectFolder: "/tmp/cursor-fixture-workspace",
});

const requestContext = create(RequestContextSchema, {
  env: requestContextEnv,
});

const userMessageAction = create(UserMessageActionSchema, {
  userMessage,
  requestContext,
});

const conversationAction = create(ConversationActionSchema, {
  action: {
    case: "userMessageAction",
    value: userMessageAction,
  },
});

const modelDetails = create(ModelDetailsSchema, {
  modelId: "composer-2-fast",
});

const agentRunRequest = create(AgentRunRequestSchema, {
  conversationState: {},
  action: conversationAction,
  modelDetails,
  conversationId: "conv-fixture-0001",
});

const agentClientMessage = create(AgentClientMessageSchema, {
  message: {
    case: "runRequest",
    value: agentRunRequest,
  },
});

const tsBytes = toBinary(AgentClientMessageSchema, agentClientMessage);

console.log(`Rust payload length: ${rustBytes.length} bytes`);
console.log(`TS payload length:   ${tsBytes.length} bytes`);

if (rustBytes.length !== tsBytes.length) {
  console.error("Length mismatch!");
  printDiff(rustBytes, tsBytes);
  process.exit(1);
}

let mismatch = false;
for (let i = 0; i < rustBytes.length; i++) {
  if (rustBytes[i] !== tsBytes[i]) {
    mismatch = true;
    break;
  }
}

if (mismatch) {
  console.error("Byte-by-byte mismatch!");
  printDiff(rustBytes, tsBytes);
  process.exit(1);
}

console.log("Success! TypeScript and Rust generated identical wire format binary payloads.");

// Let's also verify that the Rust decoder can decode what TS generated, and TS can decode what Rust generated.
// We do this by parsing the TS message using fromBinary.
try {
  const parsedFromRust = fromBinary(AgentClientMessageSchema, rustBytes);
  if (parsedFromRust.message.case !== "runRequest") {
    throw new Error("Parsed message case is not runRequest");
  }
  const runReq = parsedFromRust.message.value;
  if (runReq.conversationId !== "conv-fixture-0001") {
    throw new Error(`Unexpected conversationId: ${runReq.conversationId}`);
  }
  console.log("Success! TS successfully decoded the Rust-generated binary payload.");
} catch (err: any) {
  console.error("Failed to decode Rust binary using TS parser:", err.message);
  process.exit(1);
}

function printDiff(rust: Buffer, ts: Uint8Array) {
  const len = Math.max(rust.length, ts.length);
  console.log("Index | Rust (hex) | TS (hex) | Match?");
  console.log("------|------------|----------|-------");
  for (let i = 0; i < len; i++) {
    const r = rust[i] !== undefined ? rust[i].toString(16).padStart(2, "0") : "--";
    const t = ts[i] !== undefined ? ts[i].toString(16).padStart(2, "0") : "--";
    const match = r === t ? "OK" : "XX";
    if (match === "XX") {
      console.log(`${i.toString().padStart(5, " ")} | ${r.padStart(10, " ")} | ${t.padStart(8, " ")} | ${match}`);
    }
  }
}
