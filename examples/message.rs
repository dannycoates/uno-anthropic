use uno_anthropic::messages::params::MessageCreateParams;
use uno_anthropic::types::{ContentBlock, MessageParam, Model};
use uno_anthropic::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let content = "Write a haiku about Rust programming.";
    println!("[user]: {}", content);

    let message = client
        .messages()
        .create(
            MessageCreateParams::builder()
                .model(Model::ClaudeSonnet4_5)
                .max_tokens(1024)
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
