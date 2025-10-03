import { anthropic } from "@ai-sdk/anthropic";
import { openai } from "@ai-sdk/openai";
import { buildApplication, buildCommand } from "@stricli/core";
import { generateText } from "ai";
import pc from "picocolors";
import { error, info, success } from "./utils/colors.ts";
import { copyToClipboard, executeCommand, getUserAction } from "./utils/terminal.ts";

const SYSTEM_PROMPT = `You are a helpful AI that helps user identify the shell command they are looking for.
Always provide a concise shell command.

Important:
- Think hard about the command you are going to provide, and make sure it is the best one.
- No shebang
- No explanations
- No extra text. Just the command.

User uses Linux/macOS`;

type Flags = {
  readonly provider?: "openai" | "anthropic";
  readonly model?: string;
};

function getProviderFromEnv(): "openai" | "anthropic" {
  const envProvider = Deno.env.get("DEFAULT_AI_PROVIDER");
  if (envProvider === "openai" || envProvider === "anthropic") {
    return envProvider;
  }
  return "anthropic"; // default fallback
}

function validateApiKeys(provider: string): void {
  if (provider === "openai" && !Deno.env.get("OPENAI_API_KEY")) {
    throw new Error("OPENAI_API_KEY environment variable is required when using OpenAI provider");
  }
  if (provider === "anthropic" && !Deno.env.get("ANTHROPIC_API_KEY")) {
    throw new Error("ANTHROPIC_API_KEY environment variable is required when using Anthropic provider");
  }
}

async function generateCommand(provider: string, model: string, userPrompt: string): Promise<string> {
  if (provider !== "openai" && provider !== "anthropic") {
    throw new Error(`Invalid provider: ${provider}. Must be either 'openai' or 'anthropic'`);
  }

  const { text } = await generateText({
    model: provider === "openai" ? openai(model) : anthropic(model),
    system: SYSTEM_PROMPT,
    prompt: userPrompt,
    temperature: 0.2,
  });
  return text.trim();
}

function printResponse(command: string): void {
  console.log(pc.blue("Command found:"));
  console.log(pc.blue("====================="));
  console.log(pc.green(command));
  console.log(pc.blue("====================="));
}

async function runUserAction(choice: string, command: string): Promise<void> {
  switch (choice) {
    case "c":
      success("Copying command to clipboard...");
      await copyToClipboard(command);
      success("Command copied to clipboard!");
      break;
    case "e":
      success("Executing command...");
      await executeCommand(command);
      break;
    default:
      error("Aborting...");
      Deno.exit(1);
  }
}

const generateCommandCommand = buildCommand({
  func: async (flags: Flags, ...promptParts: string[]) => {
    const provider = flags.provider ?? getProviderFromEnv();
    const model = flags.model ??
      (provider === "openai"
        ? Deno.env.get("DEFAULT_OPENAI_MODEL") ?? "gpt-5-mini"
        : Deno.env.get("DEFAULT_ANTHROPIC_MODEL") ?? "claude-3-5-haiku-20241022");

    // Validate that required API keys are available
    validateApiKeys(provider);

    info(`Using ${provider} with model: ${model}`);

    const prompt = promptParts.join(" ").trim();
    if (!prompt) {
      throw new Error("Please provide a prompt");
    }

    const command = await generateCommand(provider, model, prompt);
    printResponse(command);

    const action = await getUserAction();
    await runUserAction(action, command);
  },
  parameters: {
    flags: {
      provider: {
        brief: "AI provider to use (openai or anthropic)",
        kind: "parsed",
        parse: (value: string) => {
          if (value !== "openai" && value !== "anthropic") {
            throw new Error("Provider must be either 'openai' or 'anthropic'");
          }
          return value as "openai" | "anthropic";
        },
        optional: true,
      },
      model: {
        brief: "Model to use for the AI provider",
        kind: "parsed",
        parse: String,
        optional: true,
      },
    },
    positional: {
      kind: "array",
      parameter: {
        brief: "Natural language description of the command you want",
        parse: String,
        placeholder: "prompt",
      },
    },
  },
  docs: {
    brief: "Generate shell commands from natural language descriptions",
  },
});

export const app = buildApplication(generateCommandCommand, {
  name: "tinyassist",
});
