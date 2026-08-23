import { invoke } from "@tauri-apps/api/core";

const STARTUP_RETRY_MILLISECONDS = 100;
const STARTUP_RETRY_ATTEMPTS = 30;

const wait = (milliseconds: number): Promise<void> =>
  new Promise((resolve) => window.setTimeout(resolve, milliseconds));

function isUnmanagedStateError(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return message.includes("state not managed") || message.includes("manage() before using this command");
}

export async function invokeWhenReady<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  for (let attempt = 0; attempt < STARTUP_RETRY_ATTEMPTS; attempt += 1) {
    try {
      return await invoke<T>(command, args);
    } catch (error) {
      if (!isUnmanagedStateError(error) || attempt === STARTUP_RETRY_ATTEMPTS - 1) throw error;
      await wait(STARTUP_RETRY_MILLISECONDS);
    }
  }
  throw new Error(`Tauri state did not become ready for ${command}`);
}
