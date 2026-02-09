# uno-anthropic

Unofficial Rust SDK for the [Anthropic API](https://docs.anthropic.com/en/api).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
uno-anthropic = { path = "." }
tokio = { version = "1", features = ["rt", "macros"] }
```

Optional feature flags:

```toml
uno-anthropic = { path = ".", features = ["bedrock"] }  # AWS Bedrock
uno-anthropic = { path = ".", features = ["vertex"] }    # Google Vertex AI
```

## Usage

### Basic message

```rust
use uno_anthropic::{Client, Model, MessageCreateParams, MessageParam};
use uno_anthropic::types::ContentBlock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(); // reads ANTHROPIC_API_KEY from env

    let message = client
        .messages()
        .create(
            MessageCreateParams::builder()
                .model(Model::ClaudeSonnet4_5)
                .max_tokens(1024)
                .messages(vec![MessageParam::user("Hello, Claude!")])
                .build(),
        )
        .await?;

    for block in &message.content {
        if let ContentBlock::Text(text_block) = block {
            println!("{}", text_block.text);
        }
    }

    Ok(())
}
```

### Streaming

```rust
use futures::StreamExt;
use uno_anthropic::{Client, Model, MessageCreateParams, MessageParam};
use uno_anthropic::messages::streaming::{ContentBlockDelta, StreamEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let mut stream = client
        .messages()
        .create_stream(
            MessageCreateParams::builder()
                .model(Model::ClaudeSonnet4_5)
                .max_tokens(1024)
                .messages(vec![MessageParam::user("Tell me a story.")])
                .build(),
        )
        .await?;

    while let Some(event) = stream.next().await {
        if let StreamEvent::ContentBlockDelta {
            delta: ContentBlockDelta::TextDelta { text }, ..
        } = event? {
            print!("{text}");
        }
    }

    Ok(())
}
```

### Tool use

```rust
use uno_anthropic::{Client, Model, MessageCreateParams, MessageParam};
use uno_anthropic::types::tool::{Tool, ToolDefinition, ToolInputSchema};

let tools = vec![ToolDefinition::Custom(Tool {
    name: "get_weather".into(),
    description: Some("Get weather for a location".into()),
    input_schema: ToolInputSchema {
        schema_type: "object".into(),
        properties: Some(serde_json::json!({
            "location": { "type": "string" }
        })),
        required: Some(vec!["location".into()]),
    },
    ..Default::default()
})];

let message = client
    .messages()
    .create(
        MessageCreateParams::builder()
            .model(Model::ClaudeSonnet4_5)
            .max_tokens(1024)
            .messages(vec![MessageParam::user("What's the weather in SF?")])
            .tools(tools)
            .build(),
    )
    .await?;
```

### Extended thinking

```rust
use uno_anthropic::types::ThinkingConfig;

let message = client
    .messages()
    .create(
        MessageCreateParams::builder()
            .model(Model::ClaudeSonnet4_5)
            .max_tokens(16000)
            .messages(vec![MessageParam::user("Explain quantum entanglement.")])
            .thinking(ThinkingConfig::Enabled { budget_tokens: 10000 })
            .build(),
    )
    .await?;
```

### System prompt

```rust
use uno_anthropic::types::SystemContent;

let message = client
    .messages()
    .create(
        MessageCreateParams::builder()
            .model(Model::ClaudeSonnet4_5)
            .max_tokens(1024)
            .messages(vec![MessageParam::user("Hello")])
            .system(SystemContent::from("You are a helpful assistant."))
            .build(),
    )
    .await?;
```

### Count tokens

```rust
use uno_anthropic::CountTokensParams;

let result = client
    .messages()
    .count_tokens(
        CountTokensParams::builder()
            .model(Model::ClaudeSonnet4_5)
            .messages(vec![MessageParam::user("Hello")])
            .build(),
    )
    .await?;

println!("Input tokens: {}", result.input_tokens);
```

## Configuration

The client reads these environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `ANTHROPIC_API_KEY` | API key (required) | - |
| `ANTHROPIC_BASE_URL` | API base URL | `https://api.anthropic.com` |

Or configure explicitly:

```rust
use uno_anthropic::client::ClientBuilder;

let client = ClientBuilder::new()
    .api_key("sk-...")
    .base_url("https://custom.endpoint.com")
    .max_retries(3)
    .build();
```

## Cloud integrations

### AWS Bedrock

```rust
use uno_anthropic::bedrock::BedrockConfig;

let client = BedrockConfig::from_env("us-east-1").await?.into_client();
```

### Google Vertex AI

```rust
use uno_anthropic::vertex::VertexConfig;

let client = VertexConfig::from_env("us-central1", "my-project").await?.into_client();
```

## API coverage

| API | Methods |
|-----|---------|
| Messages | `create`, `create_stream`, `count_tokens` |
| Models | `get`, `list` |
| Batches | `create`, `get`, `list`, `cancel`, `delete`, `results` |
| Beta | Header injection for beta features |

## Examples

```sh
cargo run --example message
cargo run --example streaming
cargo run --example tools
```

## Development

Requires Rust 1.85+ (2024 edition).

```sh
just check   # fmt + clippy + tests
just test    # run tests
just clippy  # lint
just doc     # generate docs
```

## License

MIT
