# Rust Anthropic SDK - Implementation Plan

## Context

Build a complete, idiomatic Rust SDK for the Anthropic API. The project is currently an empty git repo. The SDK covers the Messages API (create, streaming, count_tokens), Batch API, Models API, Beta features, and Bedrock/Vertex AI integrations. Uses reqwest for HTTP, tokio for async, serde for serialization. Use the Rust skill while implementing.

Reference: Go SDK at `/tmp/anthropic-sdk-go` for API patterns and design decisions.

## Architecture

### Crate: `anthropic`

Single crate with feature flags for cloud integrations (`bedrock`, `vertex`).

**Key design decisions:**

1. **Serde tagged enums for discriminated unions** -- `#[serde(tag = "type")]` where all variants share a `type` field (content blocks, tool choice, thinking config, stream events, citations). `#[serde(untagged)]` for unions where variants have structurally different shapes (tool definitions, message content, system content).

2. **`Model` enum with `#[serde(other)]` Unknown variant** -- Known variants give autocomplete and type safety; `Unknown(String)` via `#[serde(untagged)]` on the last variant keeps forward compatibility. All public enums that may gain API variants use `#[non_exhaustive]`.

3. **`bon::Builder`** for request param structs -- Required fields are positional, optional fields default to `None`. The `stream` field is NOT on params; it's injected internally by `create()` vs `create_stream()`.

4. **Service layer with borrow** -- `client.messages().create(...)`. Services borrow `&Client`, `Client` holds `Arc<ClientInner>` for cheap cloning.

5. **Middleware trait** for cloud providers -- Bedrock/Vertex rewrite requests transparently via middleware.

6. **Separate request/response types** -- Request types implement `Serialize`, response types implement `Deserialize`.

### Dependencies

```toml
[package]
name = "anthropic"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
tokio = { version = "1", features = ["rt", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
futures = "0.3"
tokio-stream = "0.1"
pin-project-lite = "0.2"
bon = "3"
tracing = "0.1"
rand = "0.9"

# Optional: Bedrock
aws-config = { version = "1", optional = true }
aws-credential-types = { version = "1", optional = true }
aws-sigv4 = { version = "1", optional = true }
aws-smithy-runtime-api = { version = "1", optional = true }

# Optional: Vertex
gcp_auth = { version = "0.12", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
wiremock = "0.6"

[features]
default = []
bedrock = ["dep:aws-config", "dep:aws-credential-types", "dep:aws-sigv4", "dep:aws-smithy-runtime-api"]
vertex = ["dep:gcp_auth"]
```

## File Layout

```
src/
  lib.rs                      -- Re-exports, crate docs
  client.rs                   -- Client, ClientBuilder, execute/retry logic
  config.rs                   -- ClientConfig, env var reading, default headers
  error.rs                    -- Error enum (thiserror), ApiErrorResponse, retryability
  retry.rs                    -- RetryPolicy, backoff with jitter, Retry-After parsing
  middleware.rs               -- Middleware trait, Next, chain executor
  types/
    mod.rs                    -- Re-exports all public types
    model.rs                  -- Model enum (all known variants + Other), ModelInfo
    message.rs                -- Message (response), MessageParam, MessageContent, SystemContent, Role
    content.rs                -- ContentBlock (response), ContentBlockParam (request), all block structs
    tool.rs                   -- Tool, ToolDefinition (untagged), ToolChoice, server tool types
    thinking.rs               -- ThinkingConfig, ThinkingBlock, RedactedThinkingBlock
    usage.rs                  -- Usage, CacheCreation, ServerToolUsage, MessageDeltaUsage
    metadata.rs               -- Metadata, CacheControl, ServiceTier, OutputConfig, JsonOutputFormat
    image.rs                  -- ImageBlockParam, ImageSource (Base64/Url), MediaType
    document.rs               -- DocumentBlockParam, DocumentSource (Base64Pdf/PlainText/Content/Url)
    citation.rs               -- TextCitation enum (5 variants), CitationsConfig
    search.rs                 -- SearchResultBlockParam, WebSearchToolResultBlock, WebSearchResultBlock
    common.rs                 -- StopReason, Role, shared small enums
    page.rs                   -- Page<T> pagination type
  messages/
    mod.rs                    -- MessageService: create, create_stream, count_tokens
    params.rs                 -- MessageCreateParams (bon builder), CountTokensParams (separate struct)
    streaming.rs              -- StreamEvent, ContentBlockDelta, MessageStream, accumulate()
  streaming/
    mod.rs                    -- Re-exports
    sse.rs                    -- SSE line parser (async byte stream -> RawSseEvent)
  batches/
    mod.rs                    -- BatchService: create, get, list, cancel, delete, results
    types.rs                  -- MessageBatch, BatchCreateParams, BatchRequestCounts, BatchResult
  models/
    mod.rs                    -- ModelService: get, list
  beta/
    mod.rs                    -- BetaService, beta header injection, AnthropicBeta constants
  bedrock.rs                  -- BedrockConfig, SigV4 middleware (feature-gated)
  vertex.rs                   -- VertexConfig, OAuth middleware (feature-gated)
examples/
  message.rs
  streaming.rs
  tools.rs
```

## Type Design (Corrected)

### ToolDefinition -- `#[serde(untagged)]` (not internally tagged)

Custom tools have an **optional** `type` field (omitted or `"custom"`), while server tools have a **required** `type` field (`"bash_20250124"`, etc.). Since variants have structurally different shapes, use `#[serde(untagged)]` with server tools ordered **before** the custom tool catch-all:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(untagged)]
pub enum ToolDefinition {
    // Server tools first (have required `type` field with specific values)
    Bash(BashTool),                       // type: "bash_20250124"
    TextEditor20250124(TextEditorTool),    // type: "text_editor_20250124"
    TextEditor20250429(TextEditorTool429), // type: "text_editor_20250429"
    TextEditor20250728(TextEditorTool728), // type: "text_editor_20250728"
    WebSearch(WebSearchTool),             // type: "web_search_20250305"
    // Custom tool last (catch-all, no required type)
    Custom(Tool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: ToolInputSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eager_input_streaming: Option<bool>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>, // "custom" or omitted
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub schema_type: String, // always "object"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchTool {
    #[serde(rename = "type")]
    pub tool_type: String, // "web_search_20250305"
    pub name: String,      // "web_search"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchUserLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}
// BashTool, TextEditorTool, TextEditorTool429, TextEditorTool728 follow same pattern
```

### SystemContent -- `#[serde(untagged)]`

API accepts `system: "string"` or `system: [{type: "text", ...}]`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemContent {
    Text(String),
    Blocks(Vec<TextBlockParam>),
}

impl From<&str> for SystemContent { ... }
impl From<String> for SystemContent { ... }
impl From<Vec<TextBlockParam>> for SystemContent { ... }
```

### ThinkingConfig -- corrected Adaptive variant

`Adaptive` has **no fields** (no budget_tokens):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinkingConfig {
    Enabled { budget_tokens: u32 },
    Disabled,
    Adaptive,
}
```

### ContentBlock (response) -- `#[serde(tag = "type")]`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text(TextBlock),
    Thinking(ThinkingBlock),
    RedactedThinking(RedactedThinkingBlock),
    ToolUse(ToolUseBlock),
    ServerToolUse(ServerToolUseBlock),
    WebSearchToolResult(WebSearchToolResultBlock),
}
```

### ContentBlockParam (request) -- `#[serde(tag = "type")]`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockParam {
    Text(TextBlockParam),
    Image(ImageBlockParam),
    Document(DocumentBlockParam),
    ToolUse(ToolUseBlockParam),
    ToolResult(ToolResultBlockParam),
    Thinking(ThinkingBlockParam),
    RedactedThinking(RedactedThinkingBlockParam),
    ServerToolUse(ServerToolUseBlockParam),
    WebSearchToolResult(WebSearchToolResultBlockParam),
    SearchResult(SearchResultBlockParam),
}
```

### ToolResultBlockParam -- content is a union

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlockParam {
    pub tool_use_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolResultContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<ToolResultContentBlock>), // restricted: Text, Image, SearchResult, Document only
}
```

### Citation types -- 5 variants, internally tagged

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextCitation {
    CharLocation(CharLocationCitation),
    PageLocation(PageLocationCitation),
    ContentBlockLocation(ContentBlockLocationCitation),
    WebSearchResultLocation(WebSearchResultLocationCitation),
    SearchResultLocation(SearchResultLocationCitation),
}
```

### DocumentSource -- 4 variants, internally tagged

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentSource {
    Base64(Base64DocumentSource),  // media_type: "application/pdf"
    Text(PlainTextSource),         // media_type: "text/plain"
    Content(ContentBlockSource),
    Url(UrlDocumentSource),
}
```

### WebSearchToolResultBlock -- content is a union

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebSearchToolResultContent {
    Results(Vec<WebSearchResultBlock>),
    Error(WebSearchToolRequestError),
}
```

### Page<T> -- pagination type

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub has_more: bool,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}
```

### CountTokensParams -- separate struct (not MessageCreateParams)

Different from MessageCreateParams: takes model, messages, system, tools, tool_choice, thinking -- but NOT max_tokens, temperature, top_p, top_k, stop_sequences, metadata, stream, service_tier, output_config.

```rust
#[derive(Debug, Clone, Serialize, bon::Builder)]
pub struct CountTokensParams {
    pub model: Model,
    pub messages: Vec<MessageParam>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemContent>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}
```

### MessageCreateParams -- `stream` not a user field

The `stream` field is injected internally: `create()` sends `"stream": false`, `create_stream()` sends `"stream": true`. The params struct serializes to JSON Value, and the stream field is inserted before sending.

```rust
#[derive(Debug, Clone, Serialize, bon::Builder)]
pub struct MessageCreateParams {
    pub model: Model,
    pub max_tokens: u32,
    pub messages: Vec<MessageParam>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemContent>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<OutputConfig>,
}
```

### Model enum -- all known variants

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Model {
    #[serde(rename = "claude-opus-4-6")]
    ClaudeOpus4_6,
    #[serde(rename = "claude-opus-4-5-20251101")]
    ClaudeOpus4_5_20251101,
    #[serde(rename = "claude-opus-4-5")]
    ClaudeOpus4_5,
    #[serde(rename = "claude-opus-4-1-20250805")]
    ClaudeOpus4_1_20250805,
    #[serde(rename = "claude-opus-4-0")]
    ClaudeOpus4_0,
    #[serde(rename = "claude-opus-4-20250514")]
    ClaudeOpus4_20250514,
    #[serde(rename = "claude-4-opus-20250514")]
    Claude4Opus20250514,
    #[serde(rename = "claude-sonnet-4-5")]
    ClaudeSonnet4_5,
    #[serde(rename = "claude-sonnet-4-5-20250929")]
    ClaudeSonnet4_5_20250929,
    #[serde(rename = "claude-sonnet-4-0")]
    ClaudeSonnet4_0,
    #[serde(rename = "claude-sonnet-4-20250514")]
    ClaudeSonnet4_20250514,
    #[serde(rename = "claude-4-sonnet-20250514")]
    Claude4Sonnet20250514,
    #[serde(rename = "claude-haiku-4-5")]
    ClaudeHaiku4_5,
    #[serde(rename = "claude-haiku-4-5-20251001")]
    ClaudeHaiku4_5_20251001,
    #[serde(rename = "claude-3-7-sonnet-latest")]
    Claude3_7SonnetLatest,
    #[serde(rename = "claude-3-7-sonnet-20250219")]
    Claude3_7Sonnet20250219,
    #[serde(rename = "claude-3-5-haiku-latest")]
    Claude3_5HaikuLatest,
    #[serde(rename = "claude-3-5-haiku-20241022")]
    Claude3_5Haiku20241022,
    #[serde(rename = "claude-3-opus-latest")]
    Claude3OpusLatest,
    #[serde(rename = "claude-3-opus-20240229")]
    Claude3Opus20240229,
    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku20240307,
    /// Any model ID not in the known variants.
    #[serde(untagged)]
    Other(String),
}

impl<S: Into<String>> From<S> for Model { ... } // enables Model::from("custom-model")
```

### Batch API types

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct MessageBatch {
    pub id: String,
    #[serde(rename = "type")]
    pub batch_type: String,
    pub processing_status: BatchProcessingStatus,
    pub request_counts: BatchRequestCounts,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub cancel_initiated_at: Option<String>,
    pub results_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum BatchProcessingStatus {
    InProgress,
    Canceling,
    Ended,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchRequestCounts {
    pub processing: u32,
    pub succeeded: u32,
    pub errored: u32,
    pub canceled: u32,
    pub expired: u32,
}

#[derive(Debug, Clone, Serialize, bon::Builder)]
pub struct BatchCreateParams {
    pub requests: Vec<BatchMessageRequest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchMessageRequest {
    pub custom_id: String,
    pub params: MessageCreateParams,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchResult {
    pub custom_id: String,
    pub result: BatchResultBody,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BatchResultBody {
    Succeeded { message: Message },
    Errored { error: ApiErrorBody },
    Canceled,
    Expired,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeletedMessageBatch {
    pub id: String,
    #[serde(rename = "type")]
    pub deleted_type: String,
}
```

### Streaming types

```rust
/// SSE event deserialized from the stream. Dispatched by `event:` field name.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart { message: Message },
    ContentBlockStart { index: u32, content_block: ContentBlock },
    ContentBlockDelta { index: u32, delta: ContentBlockDelta },
    ContentBlockStop { index: u32 },
    MessageDelta { delta: MessageDelta, usage: MessageDeltaUsage },
    MessageStop,
    Ping,
    Error { error: ApiErrorBody },
}

#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
    ThinkingDelta { thinking: String },
    SignatureDelta { signature: String },
    CitationsDelta { citation: TextCitation },
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageDelta {
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaUsage {
    pub output_tokens: u32,
}
```

### Beta support

```rust
pub struct BetaService<'a> {
    client: &'a Client,
}

impl<'a> BetaService<'a> {
    pub fn messages(&self) -> BetaMessageService<'a> { ... }
}

/// BetaMessageService injects `anthropic-beta` header into requests.
pub struct BetaMessageService<'a> {
    client: &'a Client,
    betas: Vec<String>,
}

impl<'a> BetaMessageService<'a> {
    pub fn with_betas(mut self, betas: Vec<String>) -> Self { ... }
    pub async fn create(&self, params: MessageCreateParams) -> Result<Message, Error> { ... }
    pub async fn create_stream(&self, params: MessageCreateParams) -> Result<MessageStream, Error> { ... }
}

// Known beta feature constants
pub const BETA_PROMPT_CACHING_2024_07_31: &str = "prompt-caching-2024-07-31";
pub const BETA_COMPUTER_USE_2024_10_22: &str = "computer-use-2024-10-22";
pub const BETA_FILES_API_2025_04_14: &str = "files-api-2025-04-14";
pub const BETA_MCP_CLIENT_2025_11_20: &str = "mcp-client-2025-11-20";
// ... etc
```

## Implementation Phases

### Phase 1: Foundation
**Files:** `Cargo.toml`, `src/lib.rs`, `src/error.rs`, `src/config.rs`, `src/types/**`

- Set up Cargo.toml with all dependencies and feature flags
- Define all type modules with serde derives
- Core enums: `ContentBlock`, `ContentBlockParam`, `Model`, `StopReason`, `Role`, `ToolChoice`, `ThinkingConfig`
- Union types: `ToolDefinition` (untagged), `MessageContent` (untagged), `SystemContent` (untagged), `ToolResultContent` (untagged)
- Citation types: 5 variants of `TextCitation`
- Document source types: 4 variants of `DocumentSource`
- Request types: `MessageParam`, `MessageContent`, `SystemContent`
- Response types: `Message`, `Usage`, `TextBlock`, `ToolUseBlock`, `ThinkingBlock`
- Pagination: `Page<T>`
- `Error` enum with thiserror, `ApiErrorResponse`, `ApiErrorBody`, `ApiErrorType`, `is_retryable()`
- `#[non_exhaustive]` on all public enums that may gain variants
- Serde round-trip tests for all discriminated unions

### Phase 2: Client Core
**Files:** `src/client.rs`, `src/retry.rs`, `src/middleware.rs`

- `ClientBuilder`: api_key, base_url, max_retries, timeout, http_client, default_header, middleware
- `Client::new()`: reads `ANTHROPIC_API_KEY`, `ANTHROPIC_BASE_URL` from env
- Internal `execute<T>()` and `execute_raw()` methods
- `stream` field injection: serialize params to `serde_json::Value`, insert `"stream": true/false`
- Retry logic: exponential backoff with jitter (`0.5s * 2^attempt`, capped at 8s), retry on 408/409/429/5xx, respect `Retry-After` and `x-should-retry` headers
- Default headers: `anthropic-version: 2023-06-01`, `x-api-key`, `Content-Type: application/json`, `User-Agent: Anthropic/Rust 0.1.0`
- Middleware trait with boxed future return (needed for dyn dispatch)

### Phase 3: Messages API (Non-Streaming)
**Files:** `src/messages/mod.rs`, `src/messages/params.rs`

- `MessageCreateParams` with bon builder (model + max_tokens + messages required; rest optional)
- `CountTokensParams` as separate struct (model + messages required; system/tools/tool_choice/thinking optional)
- `MessageService::create()` -- POST /v1/messages (injects `"stream": false`)
- `MessageService::count_tokens()` -- POST /v1/messages/count_tokens
- `Message::to_param()` conversion for multi-turn conversations
- Convenience constructors: `MessageParam::user()`, `MessageParam::assistant()`

### Phase 4: Streaming
**Files:** `src/streaming/sse.rs`, `src/streaming/mod.rs`, `src/messages/streaming.rs`

- SSE parser: read lines from `AsyncBufRead`, parse `event:` and `data:` fields, handle multi-line data (concatenate with `\n`), skip comments (`:` prefix), handle `retry:` field
- `StreamEvent` enum dispatched by SSE `event:` field: parse `data` as JSON into the correct variant
- `ContentBlockDelta` enum: TextDelta, InputJsonDelta, ThinkingDelta, SignatureDelta, CitationsDelta
- `MessageStream`: wraps `Pin<Box<dyn Stream<Item = Result<StreamEvent, Error>> + Send>>`, implements `futures::Stream`
- `MessageStream::accumulate()` -- consume stream, build final `Message` from events
- `MessageStream::accumulate_with(callback)` -- process each event while accumulating
- `MessageService::create_stream()` -- POST /v1/messages (injects `"stream": true`), return `MessageStream`

### Phase 5: Models & Batches
**Files:** `src/models/mod.rs`, `src/batches/**`, `src/types/page.rs`

- `ModelService::get(model_id)` -- GET /v1/models/{model_id}
- `ModelService::list(params)` -- GET /v1/models (returns `Page<ModelInfo>`)
- `BatchService::create(params)` -- POST /v1/messages/batches
- `BatchService::get(batch_id)` -- GET /v1/messages/batches/{batch_id}
- `BatchService::list(params)` -- GET /v1/messages/batches (returns `Page<MessageBatch>`)
- `BatchService::cancel(batch_id)` -- POST /v1/messages/batches/{batch_id}/cancel
- `BatchService::delete(batch_id)` -- DELETE /v1/messages/batches/{batch_id}
- `BatchService::results(batch_id)` -- GET .../results, returns `impl Stream<Item = Result<BatchResult>>` (JSONL)

### Phase 6: Cloud Integrations
**Files:** `src/bedrock.rs`, `src/vertex.rs`

- `BedrockConfig::from_env(region)` -- load AWS credentials, build Client with BedrockMiddleware
- BedrockMiddleware: move `model` from body to URL (`/model/{model}/invoke`), add `anthropic_version` to body, sign with SigV4
- `VertexConfig::from_env(region, project_id)` -- load GCP credentials, build Client with VertexMiddleware
- VertexMiddleware: rewrite URL to Vertex endpoint, inject OAuth bearer token, move model to URL

### Phase 7: Beta & Polish
**Files:** `src/beta/mod.rs`, `examples/**`

- `BetaService` and `BetaMessageService` with `anthropic-beta` header injection
- Known beta feature string constants
- Example programs: basic message, streaming, tool use, extended thinking, web search
- `///` documentation comments on all public items

## User-Facing API

```rust
// Basic
let client = Client::new();
let msg = client.messages().create(
    MessageCreateParams::builder()
        .model(Model::ClaudeOpus4_6)
        .max_tokens(1024)
        .messages(vec![MessageParam::user("Hello")])
        .build()
).await?;

// Streaming
let mut stream = client.messages().create_stream(params).await?;
while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::ContentBlockDelta { delta: ContentBlockDelta::TextDelta { text }, .. } => {
            print!("{text}");
        }
        _ => {}
    }
}

// Accumulate with callback
let message = client.messages().create_stream(params).await?
    .accumulate_with(|e| { /* handle deltas */ }).await?;

// Tool definitions
let tools = vec![
    ToolDefinition::Custom(Tool {
        name: "get_weather".into(),
        description: Some("Get weather for a location".into()),
        input_schema: ToolInputSchema {
            schema_type: "object".into(),
            properties: Some(json!({"location": {"type": "string"}})),
            required: Some(vec!["location".into()]),
        },
        ..Default::default()
    }),
    ToolDefinition::web_search().max_uses(5).build(),
];

// System prompt (string or blocks)
MessageCreateParams::builder()
    .model(Model::ClaudeOpus4_6)
    .max_tokens(1024)
    .messages(vec![MessageParam::user("Hello")])
    .system(SystemContent::from("You are a helpful assistant."))
    .build()
```

## Verification

1. `cargo build` -- compiles without warnings
2. `cargo test` -- serde round-trip tests for all discriminated union types, client builder tests, mock server integration tests (wiremock)
3. `cargo run --example message` -- sends a real message (requires `ANTHROPIC_API_KEY`)
4. `cargo run --example streaming` -- streams a real response
5. `cargo clippy` -- no warnings
6. `cargo doc --open` -- documentation renders correctly
