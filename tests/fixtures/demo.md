# Welcome to ink

A terminal markdown reader that actually looks good.

## Code highlighting

```rust
fn main() {
    let name = "ink";
    println!("Hello from {}!", name);
}
```

```python
def greet(name: str) -> str:
    return f"Hello, {name}!"
```

## Tables

| Feature | Status |
|---------|--------|
| Syntax highlighting | Done |
| Inline images | Done |
| Mermaid diagrams | Done |
| Themes | 8 built-in |
| Search | Done |

## Admonitions

> [!NOTE]
> ink renders markdown in your terminal with full styling.

> [!TIP]
> Press `T` to open the theme picker and preview themes live.

> [!WARNING]
> This is a warning block with important information.

## Task list

- [x] Syntax highlighting
- [x] Table of contents
- [x] Multi-tab support
- [ ] LaTeX math rendering
- [ ] PDF export

## Blockquotes

> "The best way to read markdown is in the terminal."
>
> — Someone who gets it

## Links and formatting

Visit the [GitHub repo](https://github.com/borghei/ink) for more info.

This text has **bold**, *italic*, and ~~strikethrough~~ formatting. You can also use `inline code` for short snippets.

---

Built with Rust. Fast startup. No dependencies.
