use uno_anthropic::bedrock::BedrockConfig;
use uno_anthropic::messages::params::MessageCreateParams;
use uno_anthropic::types::{ContentBlock, MessageParam, Model};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Uses the default AWS credential chain:
    // env vars (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY),
    // ~/.aws/credentials, IMDS, etc.
    let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into());
    let config = BedrockConfig::from_env(&region).await;
    let client = config.into_client();

    // Bedrock uses its own model ID format, not the standard Anthropic identifiers.
    // Cross-region IDs use the "us." (or region) prefix.
    let model_id =
        std::env::var("BEDROCK_MODEL_ID").unwrap_or_else(|_| "us.anthropic.claude-haiku-4-5-20251001-v1:0".into());

    let content = "Hello from Bedrock! What's your name and how are you today?";
    println!("[user]: {content}");

    let message = client
        .messages()
        .create(
            MessageCreateParams::builder()
                .model(Model::from(model_id))
                .max_tokens(256)
                .messages(vec![MessageParam::user(content)])
                .build(),
        )
        .await?;

    print!("[assistant]: ");
    for block in &message.content {
        if let ContentBlock::Text(text_block) = block {
            println!("{}", text_block.text);
        }
    }

    println!(
        "\n(input_tokens: {}, output_tokens: {})",
        message.usage.input_tokens, message.usage.output_tokens
    );

    Ok(())
}
