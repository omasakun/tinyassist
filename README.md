<br>
<div align="center">
  <h1>TinyAssist</h1>
  <p>Simple cross-platform LLM assistant for shell.</p>
  <a href="https://asciinema.org/a/1265710" target="_blank"><img src="https://asciinema.org/a/1265710.svg" /></a>
</div>
<br>
<br>

Simple and small cross-platform command-line assistant that suggests shell commands from natural language descriptions.

Statically linked binaries are available for Linux (~2.0 MB) and Windows (~1.6 MB).

## Install

Download a static binary from the [releases page](https://github.com/omasakun/tinyassist/releases), or install via Cargo:

```sh
cargo install --git https://github.com/omasakun/tinyassist.git
```

Or build from source:

```sh
just build           # native (glibc)
just build-musl      # static musl
just build-windows   # static CRT
```

## Usage

```sh
# 'ta' is the default alias for 'tinyassist'
ta list large files  # suggest and run a command
ta -f                # fix the last command
ta -c hello          # free-form chat
ta --help            # show all options
```

## Shell integration

Set up the shell integration to use the `ta` alias:

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

Add the above to your shell config to make it permanent.

`--alias <name>` changes the alias from `ta` to `<name>`.

## Configuration

Set an API key for the provider that matches your model:

| Provider  | Environment variable |
| --------- | -------------------- |
| OpenAI    | `OPENAI_API_KEY`     |
| Anthropic | `ANTHROPIC_API_KEY`  |
| Gemini    | `GEMINI_API_KEY`     |
| Groq      | `GROQ_API_KEY`       |
| DeepSeek  | `DEEPSEEK_API_KEY`   |

Then set the model and reasoning effort:

| Environment variable       | Default value  | Options                           |
| -------------------------- | -------------- | --------------------------------- |
| `DEFAULT_MODEL`            | `gpt-5.6-luna` | Provider-specific model names     |
| `DEFAULT_REASONING_EFFORT` | `off`          | `off` / `low` / `medium` / `high` |

Set via environment variables, or in a config file ([example](.env.example)) typically located at `~/.config/tinyassist/.env`.

Check `tinyassist --info` for the actual config path.

## Development

Requirements (recommended): [Nix](https://nixos.org/download.html), [direnv](https://direnv.net/), and [just](https://github.com/casey/just).

Enter the Nix devShell (via `direnv` or `nix develop`), then:

```sh
just        # list all tasks
just check  # fmt, check, clippy, test
just run    # run the CLI
```

Releases are built by [`.github/workflows/release.yml`](.github/workflows/release.yml) on `v*` tags.

## License

This project is licensed under [MIT License](LICENSE).

Copyright 2026 omasakun
