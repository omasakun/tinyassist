# tinyassist

Simple, cross-platform command-line assistant that generates shell commands from natural language descriptions.

Statically linked binaries are available for Linux (~2.0 MB) and Windows (~1.6 MB).

## Install

Download a static binary from the releases page, or build from source:

```sh
just build           # native (glibc)
just build-musl      # static musl
just build-windows   # static CRT
```

## Configuration

Set an API key for the provider that matches your model:

| Provider  | Environment variable |
| --------- | -------------------- |
| OpenAI    | `OPENAI_API_KEY`     |
| Anthropic | `ANTHROPIC_API_KEY`  |
| Gemini    | `GEMINI_API_KEY`     |
| Groq      | `GROQ_API_KEY`       |
| DeepSeek  | `DEEPSEEK_API_KEY`   |

Run `tinyassist --info` to check config location.

## Usage

```sh
ta list large files  # suggest and run a command
ta -f                # fix the last command
ta -c hello          # free-form chat
```

## Shell integration

The shell function records the last command and exit code, which `-f` uses:

```sh
# bash
eval "$(tinyassist --init bash)"

# zsh
eval "$(tinyassist --init zsh)"

# fish
tinyassist --init fish | source

# PowerShell
Invoke-Expression (& tinyassist --init pwsh | Out-String)
```

`--alias <name>` changes the function name (default: `ta`).

## Development

Enter the Nix devShell (via `direnv` or `nix develop`), then:

```sh
just        # list all tasks
just check  # fmt, check, clippy, test
just run    # run the CLI
```

Releases are built by `.github/workflows/release.yml` on `v*` tags.
