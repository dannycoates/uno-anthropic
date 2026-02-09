use futures::StreamExt;

use uno_anthropic::messages::params::MessageCreateParams;
use uno_anthropic::messages::streaming::{ContentBlockDelta, StreamEvent};
use uno_anthropic::types::{MessageParam, Model};
use uno_anthropic::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let content = "Write a short story about a robot learning to paint. Keep it under 200 words.";
    println!("[user]: {}", content);
    print!("[assistant]: ");

    let mut stream = client
        .messages()
        .create_stream(
            MessageCreateParams::builder()
                .model(Model::ClaudeSonnet4_5)
                .max_tokens(1024)
                .messages(vec![MessageParam::user(content)])
                .build(),
        )
        .await?;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockDelta {
                delta: ContentBlockDelta::TextDelta { text },
                ..
            } => {
                print!("{}", text);
            }
            StreamEvent::MessageDelta { delta, .. } => {
                if let Some(ref stop_sequence) = delta.stop_sequence {
                    print!("{}", stop_sequence);
                }
            }
            _ => {}
        }
    }

    println!();
    Ok(())
}
