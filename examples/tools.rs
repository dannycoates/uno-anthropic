use uno_anthropic::messages::params::MessageCreateParams;
use uno_anthropic::types::content::{ContentBlock, ContentBlockParam, ToolResultBlockParam};
use uno_anthropic::types::message::{MessageContent, MessageParam};
use uno_anthropic::types::content::ToolResultContent;
use uno_anthropic::types::tool::{Tool, ToolDefinition, ToolInputSchema};
use uno_anthropic::types::Model;
use uno_anthropic::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let tools = vec![ToolDefinition::Custom(Tool {
        name: "get_weather".to_string(),
        description: Some("Get the current weather for a location.".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: Some(serde_json::json!({
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                }
            })),
            required: Some(vec!["location".to_string()]),
            ..Default::default()
        },
        ..Default::default()
    })];

    let mut messages = vec![MessageParam::user(
        "What is the weather like in San Francisco?",
    )];

    println!("[user]: What is the weather like in San Francisco?");

    loop {
        let message = client
            .messages()
            .create(
                MessageCreateParams::builder()
                    .model(Model::ClaudeSonnet4_5)
                    .max_tokens(1024)
                    .messages(messages.clone())
                    .tools(tools.clone())
                    .build(),
            )
            .await?;

        // Print assistant response
        print!("[assistant]: ");
        for block in &message.content {
            match block {
                ContentBlock::Text(text_block) => {
                    println!("{}", text_block.text);
                }
                ContentBlock::ToolUse(tool_use) => {
                    println!(
                        "Using tool '{}' with input: {}",
                        tool_use.name, tool_use.input
                    );
                }
                _ => {}
            }
        }

        // Add assistant response to conversation
        messages.push(message.to_param());

        // Collect tool results
        let mut tool_results: Vec<ContentBlockParam> = Vec::new();
        for block in &message.content {
            if let ContentBlock::ToolUse(tool_use) = block {
                // Simulate tool execution
                let result = match tool_use.name.as_str() {
                    "get_weather" => {
                        serde_json::json!({
                            "temperature": "72",
                            "unit": "fahrenheit",
                            "conditions": "Partly cloudy",
                            "location": "San Francisco, CA"
                        })
                        .to_string()
                    }
                    _ => "Unknown tool".to_string(),
                };

                println!("[tool result]: {}", result);

                tool_results.push(ContentBlockParam::ToolResult(ToolResultBlockParam {
                    tool_use_id: tool_use.id.clone(),
                    content: Some(ToolResultContent::Text(result)),
                    is_error: None,
                    cache_control: None,
                }));
            }
        }

        if tool_results.is_empty() {
            break;
        }

        // Send tool results back
        messages.push(MessageParam {
            role: uno_anthropic::types::Role::User,
            content: MessageContent::Blocks(tool_results),
        });
    }

    Ok(())
}
