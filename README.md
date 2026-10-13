# 🔍 EVM Trace Support for Zed (`zed-trace`)

High-performance **EVM & Foundry trace navigator** for [Zed](https://zed.dev). Turn raw, thousands-of-lines ASCII Solidity/Foundry execution traces (`.trace`, `.traces`) into an interactive, structured, navigable call tree inside Zed editor.

---

## ⚡ Features

### 1. Hierarchical Symbol Outline (Zed Symbol Side Panel / Buffer Symbols)
- Parses ASCII tree branches (`├─ `, `│  `, `└─ `) into structured nested parent-child trees.
- Populate Zed's **Buffer Symbols** (`cmd-shift-o` / `ctrl-shift-o`) and **Outline Panel** (`cmd-shift-b` / `ctrl-shift-b`).
- Instant keyboard navigation: jump directly to any sub-call (e.g. `flashLoan`, `swap`, `buy`, `sync`, or `withdraw`) across massive trace files.
- Categorized symbols with native icons:
  - 🔷 **Root Test Call**: `CCCTest::setUp()`, `CCCTest::test_profit_success()`
  - ⚙️ **Contract Call**: `Proof::run()`, `0xbb4C...::balanceOf() [staticcall]`
  - 🏗️ **Contract Creation**: `new Proof@0x3255...`
  - 📢 **Event Emission**: `emit FlashLoan(...)`, `emit Transfer(...)`
  - 📦 **State / Storage**: `storage changes`

### 2. Deep Code Folding (`textDocument/foldingRange`)
- Automatically generates folding ranges for every nested execution branch.
- Click gutter fold arrows or use `cmd-alt-{` / `cmd-alt-}` (`ctrl-alt-{` / `ctrl-alt-}`) to collapse thousands of lines of sub-calls in a single keystroke.
- Folds storage change blocks and sub-traces cleanly.

### 3. Rich Hover Analysis (`textDocument/hover`)
- **Gas Metrics**: Formats raw gas into readable metrics (e.g., `14,716,127 gas (~14.72M)`).
- **Address Labels**: Resolves contract names (e.g., `WBNB`, `PancakeRouter`, `ListaDAO`) and labels registered via `VM::label`.
- **Value & Parameters**: Decodes wei amounts into formatted token units.

### 4. Interactive Phalcon-Style Web Viewer (`trace-lsp web`)
- Built-in command to export or view any `.trace` file in a browser with a **Phalcon Explorer** invocation flow interface (collapsible nodes, search filter, and toggle pills for Static Call, Storage Changes, and Events).

---

## 📁 Repository Structure

```text
zed-trace/
├── extension.toml             # Zed extension metadata
├── extension.wasm             # Compiled WebAssembly extension (wasm32-wasip1)
├── Cargo.toml                 # Workspace & extension crate manifest
├── src/
│   └── trace.rs               # Zed WASM Extension entrypoint (finds & launches trace-lsp)
├── languages/
│   └── evm-trace/
│       └── config.toml        # Language config (file suffixes: .trace, .traces, .sol.trace)
├── crates/
│   └── trace-lsp/             # Standalone Trace Language Server & CLI
│       ├── Cargo.toml
│       └── src/
│           ├── parser.rs      # High-performance trace parser (tree depth & branch recovery)
│           ├── analysis.rs    # LSP DocumentSymbol, FoldingRange, & Hover generator
│           ├── server.rs      # JSON-RPC stdio language server
│           ├── web.rs         # Phalcon-style interactive HTML generator
│           ├── lib.rs
│           └── main.rs
└── sample.trace               # Reference Foundry trace for testing & benchmarking
```

---

## 🚀 Getting Started

### 1. Install `trace-lsp`
```bash
cargo install --path crates/trace-lsp
```
Verify the installation:
```bash
trace-lsp --version
```

### 2. Install the Extension in Zed
1. In Zed, open the command palette (`cmd-shift-p` / `ctrl-shift-p`).
2. Run `zed: install dev extension`.
3. Select this repository folder (`zed-trace`).

### 3. Open a Trace File
Open `sample.trace` or any Foundry trace file.
Ensure LSP document symbols are enabled in your Zed settings (`~/.config/zed/settings.json`):
```json
{
  "languages": {
    "EVM Trace": {
      "document_symbols": "on"
    }
  }
}
```

- Press `ctrl-shift-o` (or `cmd-shift-o`) to open the **Buffer Symbols** modal.
- Press `ctrl-shift-b` (or `cmd-shift-b`) to toggle the **Outline Panel**.
- Use `ctrl-alt-{` and `ctrl-alt-}` to fold and unfold call branches.

### 4. Optional: Generate Phalcon-Style Web View
```bash
trace-lsp web sample.trace
# Generates sample_explorer.html
```
Open `sample_explorer.html` in your favorite browser for the full interactive visual flow!
