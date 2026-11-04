# zed-nextflow

Nextflow language support for [Zed](https://zed.dev/), built on the [Nextflow tree-sitter grammar](https://github.com/nextflow-io/tree-sitter-nextflow) and the [Nextflow language server](https://github.com/nextflow-io/language-server).

<img width="1092" alt="Screenshot" src="assets/img.png">

## Features

- Syntax highlighting for `.nf` scripts and `.config` files
- Bash highlighting inside `script` and `stub` blocks
- Diagnostics, go-to-definition, completion, hover, and formatting from the language server
- Outline view for processes, workflows, and functions
- Automatic download of the language server JAR, pinned to a Nextflow version

## Requirements

Java 17 or later, to run the language server.

## Install

Open the command palette, run `zed: extensions`, search for Nextflow, and install.

Open a `.nf` file to start the language server. It downloads its JAR on first use, so the first open takes a moment and needs network access.

## Configuration

Settings go under the `lsp` key in your global Zed `settings.json`, or in a project's `.zed/settings.json`.

```json
{
  "lsp": {
    "nextflow-language-server": {
      "settings": {
        "nextflow": {
          "languageVersion": "26.10",
          "java": {
            "home": "/path/to/jdk-17-or-later"
          },
          "languageServer": {
            "path": "/absolute/path/to/language-server-all.jar"
          },
          "errorReportingMode": "warnings",
          "files": {
            "exclude": [".git", ".lineage", ".nf-test", ".pixi", ".venv", "work"]
          }
        }
      }
    }
  }
}
```

The extension sends the same language server defaults as the [VS Code extension](https://github.com/nextflow-io/vscode-language-nextflow#configuration):

| Setting | Default |
| --- | --- |
| `nextflow.completion.extended` | `false` |
| `nextflow.completion.maxItems` | `100` |
| `nextflow.debug` | `false` |
| `nextflow.errorReportingMode` | `"warnings"` |
| `nextflow.files.exclude` | `.git`, `.lineage`, `.nf-test`, `.pixi`, `.venv`, `work` |
| `nextflow.formatting.harshilAlignment` | `false` |
| `nextflow.formatting.maheshForm` | `false` |
| `nextflow.formatting.sortDeclarations` | `false` |

Anything you set in `settings.json` overrides these. See [Configuring language servers](https://zed.dev/docs/configuring-languages#configuring-language-servers).

Two settings are extension-specific rather than language server settings:

- `nextflow.java.home` points at a JDK. Without it the extension looks at `JAVA_HOME`, then `PATH`.
- `nextflow.languageServer.path` points at a JAR you built yourself, which skips the download. Useful when working on the language server.

`nextflow.languageVersion` picks which language server release to download. Supported values are `26.10` (the default), `26.04`, `25.10`, `25.04`, and `24.10`. The extension downloads the newest patch release for that version. Restart the language server after changing the version, the JDK, or the JAR path.

## Development

Clone the repository, then run `zed: install dev extension` from the command palette and pick the directory. Zed builds and installs it, replacing the registry version until you uninstall it. This needs [Rust](https://www.rust-lang.org/tools/install).

```bash
cargo build --release --target wasm32-wasip2
```

Query files live in `languages/nextflow/`. After editing one, run `zed: reload extensions` to see the change. To check a query against the grammar:

```bash
tree-sitter query languages/nextflow/highlights.scm path/to/script.nf
```

The grammar revision is pinned in `extension.toml`. Bump it when `tree-sitter-nextflow` cuts a release, and re-run the queries against the new revision, since node names can change.

To publish a new version, bump `version` in `extension.toml` and `Cargo.toml`, then open a PR against [zed-industries/extensions](https://github.com/zed-industries/extensions) updating the `version` for `nextflow` in `extensions.toml`.

## Credits

This extension is built on several community contributions:

- [ctuni/zed-nextflow](https://github.com/ctuni/zed-nextflow)
- [DLBPointon/zed_nextflow](https://github.com/DLBPointon/zed_nextflow)
- [Sam-Sims/nextflow-zed](https://github.com/Sam-Sims/nextflow-zed)
- [valentinegb/zed-groovy](https://github.com/valentinegb/zed-groovy)
