import pc from "picocolors";

export async function getKeyPress(): Promise<string> {
  Deno.stdin.setRaw(true);
  const buffer = new Uint8Array(1);
  await Deno.stdin.read(buffer);
  Deno.stdin.setRaw(false);
  return new TextDecoder().decode(buffer).toLowerCase();
}

export async function getUserAction(): Promise<string> {
  console.log(pc.blue("What do you want to do with this command?"));
  console.log(pc.blue("[c] Copy  [e] Execute  [a] Abort"));
  console.log(pc.blue("Press key: "), { newline: false });
  return await getKeyPress();
}

export async function copyToClipboard(text: string): Promise<void> {
  try {
    const { default: clipboardy } = await import("clipboardy");
    await clipboardy.write(text);
  } catch (error) {
    console.log(pc.yellow("Clipboard not available, please copy manually:"));
    console.log(text);
    throw error;
  }
}

export async function executeCommand(command: string): Promise<void> {
  const process = new Deno.Command("sh", {
    args: ["-c", command],
    stdout: "inherit",
    stderr: "inherit",
  });

  const { success } = await process.output();
  if (!success) {
    throw new Error("Command execution failed");
  }
}
