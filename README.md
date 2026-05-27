<p align="center"><strong>BCIP Agent</strong> (宝宸知识产权) is an intelligent coding agent that runs locally on your computer.
<p align="center">
  <img src="https://github.com/xujian519/BCIP/blob/main/.github/codex-cli-splash.png" alt="BCIP Agent splash" width="80%" />
</p>
</br>
If you want BCIP Agent in your code editor (VS Code, Cursor, Windsurf), <a href="#">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>bcip app</code> or visit <a href="#">the BCIP App page</a>.
</br>If you are looking for the <em>cloud-based agent</em>, <strong>BCIP Web</strong>, go to <a href="#">BCIP Web</a>.</p>

---

## Quickstart

### Installing and running BCIP Agent

Run the following on Mac or Linux to install BCIP Agent:

```shell
curl -fsSL # | sh
```

Run the following on Windows to install BCIP Agent:

```
powershell -ExecutionPolicy ByPass -c "irm # | iex"
```

BCIP Agent can also be installed via the following package managers:

```shell
# Install using npm
npm install -g @xujian519/bcip-agent
```

```shell
# Install using Homebrew
brew install --cask bcip-agent
```

Then simply run `bcip` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/xujian519/BCIP/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `bcip-agent-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `bcip-agent-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `bcip-agent-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `bcip-agent-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g. `bcip-agent-x86_64-unknown-linux-musl`), so you likely want to rename it to `bcip-agent` after extracting it.

</details>

### Using BCIP Agent with your ChatGPT plan

Run `bcip` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use BCIP Agent as part of your Plus, Pro, Business, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](#).

You can also use BCIP Agent with an API key, but this requires [additional setup](#).

## Docs

- [**BCIP Agent Documentation**](#)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
