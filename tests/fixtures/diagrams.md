# Diagrams & Charts Demo

## Flowcharts

### Simple Flow

```mermaid
flowchart TD
    A[User Request] --> B{Auth Check}
    B -->|Valid| C[Process Request]
    B -->|Invalid| D[Return 401]
    C --> E[Query Database]
    E --> F{Results?}
    F -->|Found| G[Return Data]
    F -->|Empty| H[Return 404]
```

### CI/CD Pipeline

```mermaid
flowchart LR
    A[Push Code] --> B[Run Tests]
    B --> C{Tests Pass?}
    C -->|Yes| D[Build Docker]
    C -->|No| E[Notify Team]
    D --> F[Deploy Staging]
    F --> G{QA Approved?}
    G -->|Yes| H[Deploy Production]
    G -->|No| E
```

## Sequence Diagrams

### API Authentication Flow

```mermaid
sequenceDiagram
    Client->>API Gateway: POST /auth/login
    API Gateway->>Auth Service: Validate Credentials
    Auth Service->>Database: Query User
    Database-->>Auth Service: User Record
    Auth Service-->>API Gateway: JWT Token
    API Gateway-->>Client: 200 OK + Token
    Client->>API Gateway: GET /api/data (Bearer Token)
    API Gateway->>Auth Service: Verify Token
    Auth Service-->>API Gateway: Token Valid
    API Gateway->>Data Service: Fetch Data
    Data Service-->>API Gateway: Response
    API Gateway-->>Client: 200 OK + Data
```

### Microservice Communication

```mermaid
sequenceDiagram
    Order Service->>Payment Service: Process Payment
    Payment Service->>Stripe: Charge Card
    Stripe-->>Payment Service: Payment Confirmed
    Payment Service->>Order Service: Payment Success
    Order Service->>Inventory Service: Reserve Items
    Inventory Service-->>Order Service: Items Reserved
    Order Service->>Notification Service: Send Confirmation
    Notification Service->>Email Provider: Send Email
```

## Pie Charts

### Tech Stack Distribution

```mermaid
pie
    title Technology Distribution
    "Rust" : 35
    "TypeScript" : 25
    "Python" : 20
    "Go" : 12
    "Shell" : 5
    "Other" : 3
```

### Time Allocation

```mermaid
pie
    title Weekly Time Allocation
    "Coding" : 40
    "Code Review" : 15
    "Meetings" : 15
    "Planning" : 10
    "Documentation" : 10
    "Learning" : 10
```

## Gantt Charts

### Product Roadmap

```mermaid
gantt
    title Q2 2026 Roadmap
    dateFormat YYYY-MM-DD
    section Core
    Markdown Parser      : 2026-04-01, 14d
    Layout Engine        : 2026-04-10, 21d
    Theme System         : 2026-04-15, 10d
    section Features
    Search               : 2026-05-01, 7d
    TOC Panel            : 2026-05-05, 5d
    Image Support        : 2026-05-10, 14d
    section Polish
    Performance Tuning   : 2026-06-01, 10d
    Documentation        : 2026-06-05, 7d
    Release v1.0         : 2026-06-15, 3d
```

## Tree Structures

Here's a project directory tree:

```
ink/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── config.rs
│   ├── parser/
│   │   ├── mod.rs
│   │   └── frontmatter.rs
│   ├── layout/
│   │   ├── mod.rs
│   │   ├── table.rs
│   │   └── mermaid.rs
│   ├── render/
│   │   ├── mod.rs
│   │   └── plain.rs
│   ├── theme/
│   │   ├── mod.rs
│   │   ├── builtin.rs
│   │   └── detect.rs
│   └── input/
│       └── mod.rs
├── themes/
│   ├── dark.toml
│   └── dracula.toml
└── tests/
    └── fixtures/
        ├── test.md
        └── diagrams.md
```

## Nested Data Visualization

### Decision Matrix

| Criteria        | Weight | Option A: Rust | Option B: Go | Option C: Python |
|-----------------|--------|----------------|--------------|------------------|
| Performance     | 30%    | 9/10           | 8/10         | 5/10             |
| Memory Safety   | 25%    | 10/10          | 7/10         | 6/10             |
| Developer Speed | 20%    | 6/10           | 8/10         | 9/10             |
| Ecosystem       | 15%    | 7/10           | 7/10         | 10/10            |
| Binary Size     | 10%    | 9/10           | 8/10         | 3/10             |
| **Total**       | **100%** | **8.35**     | **7.60**     | **6.55**         |

### Feature Comparison

| Feature               | ink   | glow  | bat   | rich  | mdcat | frogmouth |
|-----------------------|-------|-------|-------|-------|-------|-----------|
| Rendered Markdown     | ✓     | ✓     | ✗     | ✓     | ✓     | ✓         |
| Syntax Highlighting   | ✓     | ✓     | ✓     | ✓     | ✓     | ✓         |
| Inline Images         | ○     | ✗     | ✗     | ✗     | ✓     | ✗         |
| Clickable Links       | ✓     | ✗     | ✗     | ✗     | ✓     | ✗         |
| Table of Contents     | ✓     | ✗     | ✗     | ✗     | ✗     | ✓         |
| Theme Picker          | ✓     | ✗     | ✓     | ✗     | ✗     | ✗         |
| In-Document Search    | ✓     | ✗     | via less | ✗  | ✗     | ✗         |
| Mermaid Diagrams      | ✓     | ✗     | ✗     | ✗     | ✗     | ✗         |
| Admonitions           | ✓     | ✗     | ✗     | ✗     | ✗     | ✓         |
| Word-Wrapped Tables   | ✓     | ✗     | ✗     | ✗     | ✗     | ✗         |
| Multi-File Tabs       | ✓     | ✗     | ✗     | ✗     | ✗     | ✗         |
| Progress Bar          | ✓     | ✗     | ✗     | ✗     | ✗     | ✗         |
| Word Count / ETA      | ✓     | ✗     | ✗     | ✗     | ✗     | ✗         |
| Fast Startup (<10ms)  | ✓     | ○     | ✓     | ✗     | ✓     | ✗         |
| Single Binary         | ✓     | ✓     | ✓     | ✗     | ✓     | ✗         |

> **Legend:** ✓ = supported, ✗ = not supported, ○ = planned/partial

## Code Examples

### Rust — Error Handling

```rust
use std::fs;
use std::io;

#[derive(Debug)]
enum AppError {
    Io(io::Error),
    Parse(String),
    NotFound { path: String },
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

fn read_config(path: &str) -> Result<String, AppError> {
    let content = fs::read_to_string(path)?;
    if content.is_empty() {
        return Err(AppError::Parse("Empty config file".into()));
    }
    Ok(content)
}
```

### TypeScript — Async Pipeline

```typescript
interface Pipeline<T> {
  pipe<U>(fn: (value: T) => Promise<U>): Pipeline<U>;
  execute(): Promise<T>;
}

async function fetchUserData(userId: string) {
  const response = await fetch(`/api/users/${userId}`);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

const result = await createPipeline(userId)
  .pipe(fetchUserData)
  .pipe(validatePermissions)
  .pipe(enrichWithMetadata)
  .execute();
```

### Python — Data Processing

```python
from dataclasses import dataclass
from typing import Iterator

@dataclass
class Record:
    id: int
    name: str
    score: float

def process_records(records: Iterator[Record]) -> dict[str, float]:
    """Aggregate scores by first letter of name."""
    aggregated: dict[str, list[float]] = {}
    for record in records:
        key = record.name[0].upper()
        aggregated.setdefault(key, []).append(record.score)
    return {k: sum(v) / len(v) for k, v in aggregated.items()}
```

## Admonitions

> [!NOTE]
> This file demonstrates all of ink's rendering capabilities including mermaid diagrams, word-wrapped tables, syntax highlighting, and admonitions.

> [!TIP]
> Press `T` to open the theme picker and preview all 8 built-in themes in real-time.

> [!WARNING]
> Some diagram types (like gantt) use approximate visual representations since terminal rendering has inherent limitations compared to graphical output.

> [!IMPORTANT]
> All content in tables is always fully visible — ink word-wraps cells instead of truncating them.

---

*Generated for ink — the most advanced terminal markdown reader.*
