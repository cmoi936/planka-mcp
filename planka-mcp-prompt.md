
---

## 🦀 Rust MCP Server for Planka — Full Design Prompt (Copy/Paste Ready)

**SYSTEM / INSTRUCTIONS TO THE MODEL:**

You are to generate a complete, **local-only Model Context Protocol (MCP) server** written in **Rust** that integrates with a self-hosted Planka instance located at:

```text
https://kanban.local
```

Your task is to create a working, auditable, minimal MCP server that exposes Planka features as MCP tools. The server must be:

* written in idiomatic Rust,
* safe and well-structured,
* local-only,
* easy to extend.

Use modern async Rust patterns.

---

## 1. Project Goals

Build a **Rust MCP server for Planka** that:

1. Connects to Planka’s REST API using environment variables for URL + credentials.
2. Exposes a clean set of MCP tools so the AI client can read/write Planka data.
3. Runs safely on localhost, communicating with an MCP-capable client via **JSON-RPC** (stdin/stdout or TCP).
4. Is structured as a small, well-organized Rust crate:

   * Async runtime via **Tokio**.
   * HTTP via **reqwest**.
   * JSON via **serde**.
   * Command-line / env config via **clap** and `std::env`.
5. Includes all boilerplate: `Cargo.toml`, source files, build instructions, run instructions, and examples.

---

## 2. Architecture Requirements

Create a Rust project with this structure:

```text
planka-mcp-rust/
  Cargo.toml
  src/
    main.rs
    mcp/
      mod.rs
      server.rs       # JSON-RPC server implementation (stdio-based)
      types.rs        # MCP protocol types (requests, responses, tool schemas)
    planka/
      mod.rs
      client.rs       # HTTP client for Planka
      types.rs        # Data models for Planka entities (Project, Board, Card)
    tools/
      mod.rs
      list_projects.rs
      list_boards.rs
      list_cards.rs
      create_card.rs
  README.md
```

**Key requirements:**

1. Use **Tokio** as the async runtime.
2. Use **serde** / **serde_json** for JSON encoding/decoding.
3. Use **reqwest** (async) for HTTP requests to Planka.
4. Implement **JSON-RPC 2.0** as the transport between the MCP server and the MCP client:

   * Transport: stdin/stdout is preferred.
   * The server reads JSON-RPC requests from stdin, writes responses to stdout.
5. Represent MCP tools as Rust structs or enums that map to discrete JSON-RPC methods (e.g. `"planka.list_projects"`).

The project should be straightforward to `cargo build` and `cargo run`.

---

## 3. MCP Tools to Implement (MVP)

Implement these MCP tools as distinct methods/handlers:

### Tool 1: `planka.list_projects`

* **JSON-RPC method name:** `"planka.list_projects"`
* **Input:** no params
* **Output:** an array of projects:

  ```json
  [
    { "id": "string", "name": "string", "slug": "string" },
    ...
  ]
  ```

### Tool 2: `planka.list_boards`

* **JSON-RPC method name:** `"planka.list_boards"`

* **Input params:**

  ```json
  {
    "project_id": "string"
  }
  ```

* **Output:** an array of boards:

  ```json
  [
    { "id": "string", "name": "string" },
    ...
  ]
  ```

### Tool 3: `planka.list_cards`

* **JSON-RPC method name:** `"planka.list_cards"`

* **Input params:**

  ```json
  {
    "board_id": "string"
  }
  ```

* **Output:** an array of cards:

  ```json
  [
    {
      "id": "string",
      "name": "string",
      "description": "string or null",
      "list_id": "string",
      "position": "number or string as used by Planka"
    }
  ]
  ```

### Tool 4: `planka.create_card`

* **JSON-RPC method name:** `"planka.create_card"`

* **Input params:**

  ```json
  {
    "board_id": "string",
    "list_id": "string",
    "name": "string",
    "description": "string (optional)"
  }
  ```

* **Output:** the created card object in the same shape as returned by `list_cards`.

**General behavior:**

* Validate all inputs and return JSON-RPC errors (with proper error codes/message) if invalid.
* For Planka REST errors (non-2xx), return a JSON-RPC error with a safe message.
* For unexpected exceptions, return a JSON-RPC internal error with minimal leakage of internal details.

---

## 4. Planka API Client Requirements

Create a module `planka::client` that talks to Planka using **reqwest**.

1. Read configuration from environment variables:

   * `PLANKA_URL` (required, e.g. `https://kanban.local`)
   * One of:

     * `PLANKA_TOKEN` (preferred, e.g. bearer token), OR
     * `PLANKA_EMAIL` and `PLANKA_PASSWORD` for login.

2. Implement a small abstraction:

   ```rust
   pub struct PlankaClient {
       base_url: Url,
       http: reqwest::Client,
       auth: PlankaAuth, // enum for Token vs Email/Password
   }
   ```

3. Implement these methods (async):

   ```rust
   impl PlankaClient {
       pub async fn list_projects(&self) -> Result<Vec<Project>, PlankaError>;
       pub async fn list_boards(&self, project_id: &str) -> Result<Vec<Board>, PlankaError>;
       pub async fn list_cards(&self, board_id: &str) -> Result<Vec<Card>, PlankaError>;
       pub async fn create_card(
           &self,
           board_id: &str,
           list_id: &str,
           name: &str,
           description: Option<&str>,
       ) -> Result<Card, PlankaError>;
   }
   ```

4. HTTP endpoints to use (paths relative to `PLANKA_URL`):

   * `GET /api/projects`
   * `GET /api/projects/{projectId}/boards`
   * `GET /api/boards/{boardId}/cards`
   * `POST /api/boards/{boardId}/cards`

5. **Error handling:**

   * Define a `PlankaError` enum that includes:

     * `Http(reqwest::Error)`
     * `Status(u16, String)` (status code + body snippet)
     * `Config(String)` (e.g. missing env var)
     * `Serde(serde_json::Error)`
   * Implement `Display` and `std::error::Error` for `PlankaError` as appropriate.
   * Map `PlankaError` into JSON-RPC errors in the MCP layer.

6. **Data models (`planka::types`)** for `Project`, `Board`, `Card`:

   * Use `#[derive(Debug, Clone, Serialize, Deserialize)]`.
   * Only include fields that are needed by tools (`id`, `name`, `slug`, `description`, `list_id`, `position`, etc.).
   * Allow unknown fields with `#[serde(default)]` and `Option<T>` where appropriate.

---

## 5. MCP / JSON-RPC Server Requirements

In `mcp::server`:

1. Implement a **JSON-RPC 2.0** server over stdin/stdout:

   * Async loop using Tokio:

     * Read from `tokio::io::stdin()` line by line or via framed JSON.
     * Parse into a `JsonRpcRequest` type.
     * Dispatch to the appropriate tool handler.
     * Serialize a `JsonRpcResponse` and write to `tokio::io::stdout()`.

2. Define types in `mcp::types`:

   ```rust
   #[derive(Deserialize)]
   pub struct JsonRpcRequest {
       pub jsonrpc: String,
       pub method: String,
       pub params: Option<serde_json::Value>,
       pub id: Option<serde_json::Value>,
   }

   #[derive(Serialize)]
   pub struct JsonRpcResponse {
       pub jsonrpc: String,
       pub result: Option<serde_json::Value>,
       pub error: Option<JsonRpcError>,
       pub id: Option<serde_json::Value>,
   }

   #[derive(Serialize)]
   pub struct JsonRpcError {
       pub code: i32,
       pub message: String,
       pub data: Option<serde_json::Value>,
   }
   ```

3. For each supported method:

   * Match on `request.method`:

     * `"planka.list_projects"`
     * `"planka.list_boards"`
     * `"planka.list_cards"`
     * `"planka.create_card"`
   * Deserialize `params` into a strongly typed struct using `serde_json::from_value`.
   * Call the appropriate function in `tools::*`.
   * On success, serialize `result` to JSON and embed in `JsonRpcResponse.result`.
   * On error, embed a `JsonRpcError` with a suitable `code` and `message`.

4. Follow JSON-RPC conventions:

   * `"jsonrpc": "2.0"`
   * Use 0, -32600.. for typical error codes (e.g. -32602 for invalid params, -32603 for internal errors).
   * Always send a response if `id` is non-null.

5. The `main.rs` should:

   * Initialize logging (e.g. `env_logger`).
   * Initialize the async runtime via `#[tokio::main]`.
   * Construct a `PlankaClient` from env vars.
   * Enter the JSON-RPC loop.
   * Cleanly handle shutdown (EOF on stdin, etc.).

---

## 6. Safety and Local-Only Constraints

The server **must**:

1. Only communicate via `stdin`/`stdout` (preferred) or, if you choose TCP, bind only to `127.0.0.1` and make it explicit in the code.
2. Never log secrets:

   * Do not log `PLANKA_TOKEN`, passwords, or full URLs with credentials.
3. Provide deterministic, well-structured JSON-RPC responses:

   * Even on failure, respond with a JSON-RPC error.
4. Never write to disk except for standard logging.
5. Treat all inputs (method names, params) as untrusted; validate and handle gracefully.

---

## 7. Configuration & Running

You must include:

1. **Cargo.toml** with all required dependencies:

   * `tokio` (with full or io + macros features)
   * `serde`, `serde_json`
   * `reqwest` (features: `json`, `rustls-tls` or `native-tls`)
   * `thiserror` or similar for error definitions
   * `env_logger` or `tracing` + `tracing-subscriber` for logging
   * `clap` (optional, for CLI flags) if needed

2. **README.md** that documents:

   * How to set environment variables:

     ```bash
     export PLANKA_URL="https://kanban.local"
     export PLANKA_EMAIL="admin@example.com"
     export PLANKA_PASSWORD="super-secret"
     # or:
     # export PLANKA_TOKEN="..."
     ```

   * How to build:

     ```bash
     cargo build --release
     ```

   * How to run:

     ```bash
     ./target/release/planka-mcp-rust
     ```

   * How the JSON-RPC interface works (example request/response pairs).

3. An example of configuring a generic MCP client to talk to this binary, conceptually like:

   ```json
   {
     "mcpServers": {
       "planka": {
         "command": "planka-mcp-rust"
       }
     }
   }
   ```

   (You don’t need to tie it to any specific editor, just show the idea.)

---

## 8. Examples and Extensibility

Include in the README or comments:

1. Example JSON-RPC request/response for each tool, e.g.:

   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "planka.list_projects",
     "params": null
   }
   ```

2. Example of a successful `create_card` call and result.

3. A short “Extensibility” section describing how to add more tools, such as:

   * `planka.add_comment`
   * `planka.set_due_date`
   * `planka.update_card`
   * `planka.move_card`

Describe at a high level which module(s) would need new functions and new HTTP calls.

---

## 9. Output Format Requirements

Your output must be in **Markdown**, with each file clearly labeled and containing complete, runnable code.

Use this format:

````markdown
### file: Cargo.toml
```toml
# contents here
````

### file: src/main.rs

```rust
// contents here
```

### file: src/planka/client.rs

```rust
// contents here
```

```

Do **not** omit any files.  
Do **not** use placeholders like “// TODO” for core logic.  
Provide full implementations that are ready to compile and run.

---

## END OF PROMPT

Use this entire prompt as your complete specification. Generate a full Rust project (all files and code) that implements the described MCP server for Planka.

---

