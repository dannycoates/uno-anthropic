//! Comprehensive serde round-trip tests for all discriminated union types.

use uno_anthropic::types::citation::*;
use uno_anthropic::types::common::*;
use uno_anthropic::types::content::*;
use uno_anthropic::types::document::*;
use uno_anthropic::types::image::*;
use uno_anthropic::types::message::*;
use uno_anthropic::types::metadata::*;
use uno_anthropic::types::model::*;
use uno_anthropic::types::thinking::*;
use uno_anthropic::types::tool::*;

/// Helper: serialize to JSON, deserialize back, serialize again, and compare.
fn roundtrip_json<T: serde::Serialize + serde::de::DeserializeOwned>(val: &T) -> String {
    let json1 = serde_json::to_string(val).expect("first serialize");
    let deserialized: T = serde_json::from_str(&json1).expect("deserialize");
    let json2 = serde_json::to_string(&deserialized).expect("second serialize");
    assert_eq!(json1, json2, "roundtrip mismatch");
    json1
}

// ── Model ────────────────────────────────────────────────────────────

#[test]
fn roundtrip_model_known_variants() {
    let models = vec![
        Model::ClaudeOpus4_6,
        Model::ClaudeOpus4_5,
        Model::ClaudeSonnet4_5,
        Model::ClaudeSonnet4_0,
        Model::ClaudeHaiku4_5,
        Model::Claude3_7SonnetLatest,
        Model::Claude3_5HaikuLatest,
        Model::Claude3OpusLatest,
    ];
    for model in &models {
        roundtrip_json(model);
    }
}

#[test]
fn roundtrip_model_other() {
    let model = Model::Other("custom-model-2025".to_string());
    let json = roundtrip_json(&model);
    assert_eq!(json, r#""custom-model-2025""#);
}

// ── StopReason ───────────────────────────────────────────────────────

#[test]
fn roundtrip_stop_reason() {
    let reasons = vec![
        StopReason::EndTurn,
        StopReason::MaxTokens,
        StopReason::StopSequence,
        StopReason::ToolUse,
        StopReason::Refusal,
    ];
    for reason in &reasons {
        roundtrip_json(reason);
    }
}

// ── Role ─────────────────────────────────────────────────────────────

#[test]
fn roundtrip_role() {
    roundtrip_json(&Role::User);
    roundtrip_json(&Role::Assistant);
}

// ── ThinkingConfig ───────────────────────────────────────────────────

#[test]
fn roundtrip_thinking_config() {
    roundtrip_json(&ThinkingConfig::Enabled {
        budget_tokens: 10000,
    });
    roundtrip_json(&ThinkingConfig::Disabled);
    roundtrip_json(&ThinkingConfig::Adaptive);
}

// ── ToolChoice ───────────────────────────────────────────────────────

#[test]
fn roundtrip_tool_choice() {
    roundtrip_json(&ToolChoice::Auto);
    roundtrip_json(&ToolChoice::Any);
    roundtrip_json(&ToolChoice::None);
    roundtrip_json(&ToolChoice::Tool {
        name: "get_weather".to_string(),
    });
}

// ── ToolDefinition ───────────────────────────────────────────────────

#[test]
fn roundtrip_tool_definition_bash() {
    let tool = ToolDefinition::Bash(BashTool::new());
    let json = roundtrip_json(&tool);
    assert!(json.contains(r#""type":"bash_20250124""#));
}

#[test]
fn roundtrip_tool_definition_text_editor_variants() {
    roundtrip_json(&ToolDefinition::TextEditor20250124(TextEditorTool::new()));
    roundtrip_json(&ToolDefinition::TextEditor20250429(TextEditorTool429::new()));
    roundtrip_json(&ToolDefinition::TextEditor20250728(TextEditorTool728::new()));
}

#[test]
fn roundtrip_tool_definition_web_search() {
    let tool = ToolDefinition::WebSearch(WebSearchTool {
        max_uses: Some(10),
        allowed_domains: Some(vec!["example.com".to_string()]),
        ..WebSearchTool::new()
    });
    let json = roundtrip_json(&tool);
    assert!(json.contains(r#""type":"web_search_20250305""#));
    assert!(json.contains(r#""max_uses":10"#));
}

#[test]
fn roundtrip_tool_definition_custom() {
    let tool = ToolDefinition::Custom(Tool {
        name: "get_weather".to_string(),
        description: Some("Get weather for a location".to_string()),
        input_schema: ToolInputSchema {
            schema_type: "object".to_string(),
            properties: Some(serde_json::json!({"location": {"type": "string"}})),
            required: Some(vec!["location".to_string()]),
            ..Default::default()
        },
        ..Default::default()
    });
    let json = roundtrip_json(&tool);
    assert!(json.contains(r#""name":"get_weather""#));
}

#[test]
fn tool_definition_deserialize_dispatches_correctly() {
    // Each server tool type should deserialize to the correct variant
    let cases: Vec<(&str, &str)> = vec![
        (r#"{"type":"bash_20250124","name":"bash"}"#, "Bash"),
        (
            r#"{"type":"text_editor_20250124","name":"str_replace_editor"}"#,
            "TextEditor20250124",
        ),
        (
            r#"{"type":"text_editor_20250429","name":"str_replace_editor"}"#,
            "TextEditor20250429",
        ),
        (
            r#"{"type":"text_editor_20250728","name":"str_replace_editor"}"#,
            "TextEditor20250728",
        ),
        (
            r#"{"type":"web_search_20250305","name":"web_search"}"#,
            "WebSearch",
        ),
        (
            r#"{"name":"calc","input_schema":{"type":"object"}}"#,
            "Custom",
        ),
    ];
    for (json, expected) in cases {
        let tool: ToolDefinition = serde_json::from_str(json).unwrap();
        let variant_name = match &tool {
            ToolDefinition::Bash(_) => "Bash",
            ToolDefinition::TextEditor20250124(_) => "TextEditor20250124",
            ToolDefinition::TextEditor20250429(_) => "TextEditor20250429",
            ToolDefinition::TextEditor20250728(_) => "TextEditor20250728",
            ToolDefinition::WebSearch(_) => "WebSearch",
            ToolDefinition::Custom(_) => "Custom",
            _ => "Unknown",
        };
        assert_eq!(
            variant_name, expected,
            "JSON {} should parse as {}",
            json, expected
        );
    }
}

// ── ContentBlock (response) ──────────────────────────────────────────

#[test]
fn roundtrip_content_block_text() {
    let json = r#"{"type":"text","text":"Hello, world!"}"#;
    let block: ContentBlock = serde_json::from_str(json).unwrap();
    let reserialized = serde_json::to_string(&block).unwrap();
    let _roundtrip: ContentBlock = serde_json::from_str(&reserialized).unwrap();
    match block {
        ContentBlock::Text(t) => assert_eq!(t.text, "Hello, world!"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn roundtrip_content_block_thinking() {
    let json = r#"{"type":"thinking","thinking":"Let me analyze...","signature":"abc"}"#;
    let block: ContentBlock = serde_json::from_str(json).unwrap();
    let reserialized = serde_json::to_string(&block).unwrap();
    let _: ContentBlock = serde_json::from_str(&reserialized).unwrap();
}

#[test]
fn roundtrip_content_block_redacted_thinking() {
    let json = r#"{"type":"redacted_thinking","data":"redacted_data_abc"}"#;
    let block: ContentBlock = serde_json::from_str(json).unwrap();
    let reserialized = serde_json::to_string(&block).unwrap();
    let _: ContentBlock = serde_json::from_str(&reserialized).unwrap();
}

#[test]
fn roundtrip_content_block_tool_use() {
    let json = r#"{"type":"tool_use","id":"toolu_123","name":"get_weather","input":{"location":"SF"}}"#;
    let block: ContentBlock = serde_json::from_str(json).unwrap();
    let reserialized = serde_json::to_string(&block).unwrap();
    let _: ContentBlock = serde_json::from_str(&reserialized).unwrap();
    match block {
        ContentBlock::ToolUse(t) => {
            assert_eq!(t.id, "toolu_123");
            assert_eq!(t.name, "get_weather");
        }
        _ => panic!("Expected ToolUse"),
    }
}

#[test]
fn roundtrip_content_block_server_tool_use() {
    let json =
        r#"{"type":"server_tool_use","id":"stu_1","name":"web_search","input":{"query":"rust"}}"#;
    let block: ContentBlock = serde_json::from_str(json).unwrap();
    let reserialized = serde_json::to_string(&block).unwrap();
    let _: ContentBlock = serde_json::from_str(&reserialized).unwrap();
}

// ── ContentBlockParam (request) ──────────────────────────────────────

#[test]
fn roundtrip_content_block_param_text() {
    let param = ContentBlockParam::Text(TextBlockParam::new("Hello"));
    roundtrip_json(&param);
}

#[test]
fn roundtrip_content_block_param_tool_use() {
    let param = ContentBlockParam::ToolUse(ToolUseBlockParam {
        id: "tu_1".to_string(),
        name: "calc".to_string(),
        input: serde_json::json!({"x": 42}),
        cache_control: None,
    });
    roundtrip_json(&param);
}

#[test]
fn roundtrip_content_block_param_tool_result_text() {
    let param = ContentBlockParam::ToolResult(ToolResultBlockParam {
        tool_use_id: "tu_1".to_string(),
        content: Some(ToolResultContent::Text("42".to_string())),
        is_error: None,
        cache_control: None,
    });
    roundtrip_json(&param);
}

#[test]
fn roundtrip_content_block_param_tool_result_blocks() {
    let param = ContentBlockParam::ToolResult(ToolResultBlockParam {
        tool_use_id: "tu_1".to_string(),
        content: Some(ToolResultContent::Blocks(vec![
            ToolResultContentBlock::Text(TextBlockParam::new("result")),
        ])),
        is_error: Some(false),
        cache_control: None,
    });
    roundtrip_json(&param);
}

#[test]
fn roundtrip_content_block_param_thinking() {
    let param = ContentBlockParam::Thinking(ThinkingBlockParam {
        thinking: "Deep thought".to_string(),
        signature: "sig_abc".to_string(),
        cache_control: None,
    });
    roundtrip_json(&param);
}

#[test]
fn roundtrip_content_block_param_redacted_thinking() {
    let param = ContentBlockParam::RedactedThinking(RedactedThinkingBlockParam {
        data: "redacted".to_string(),
        cache_control: None,
    });
    roundtrip_json(&param);
}

// ── ToolResultContent (untagged) ─────────────────────────────────────

#[test]
fn roundtrip_tool_result_content_text() {
    let content = ToolResultContent::Text("result".to_string());
    let json = serde_json::to_string(&content).unwrap();
    assert_eq!(json, r#""result""#);
    let deserialized: ToolResultContent = serde_json::from_str(&json).unwrap();
    match deserialized {
        ToolResultContent::Text(s) => assert_eq!(s, "result"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn roundtrip_tool_result_content_blocks() {
    let content = ToolResultContent::Blocks(vec![ToolResultContentBlock::Text(
        TextBlockParam::new("text result"),
    )]);
    roundtrip_json(&content);
}

// ── MessageContent (untagged) ────────────────────────────────────────

#[test]
fn roundtrip_message_content_text() {
    let content = MessageContent::Text("Hello".to_string());
    let json = serde_json::to_string(&content).unwrap();
    assert_eq!(json, r#""Hello""#);
    let deserialized: MessageContent = serde_json::from_str(&json).unwrap();
    match deserialized {
        MessageContent::Text(s) => assert_eq!(s, "Hello"),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn roundtrip_message_content_blocks() {
    let content =
        MessageContent::Blocks(vec![ContentBlockParam::Text(TextBlockParam::new("Hi"))]);
    roundtrip_json(&content);
}

// ── SystemContent (untagged) ─────────────────────────────────────────

#[test]
fn roundtrip_system_content_text() {
    let content = SystemContent::Text("You are helpful.".to_string());
    let json = serde_json::to_string(&content).unwrap();
    assert_eq!(json, r#""You are helpful.""#);
    let deserialized: SystemContent = serde_json::from_str(&json).unwrap();
    match deserialized {
        SystemContent::Text(s) => assert_eq!(s, "You are helpful."),
        _ => panic!("Expected Text"),
    }
}

#[test]
fn roundtrip_system_content_blocks() {
    let content = SystemContent::Blocks(vec![TextBlockParam::new("System instruction")]);
    roundtrip_json(&content);
}

// ── TextCitation (internally tagged) ─────────────────────────────────

#[test]
fn roundtrip_text_citation_all_variants() {
    let citations_json = vec![
        r#"{"type":"char_location","cited_text":"hello","document_index":0,"document_title":null,"start_char_index":0,"end_char_index":5}"#,
        r#"{"type":"page_location","cited_text":"world","document_index":1,"document_title":"doc","start_page_number":1,"end_page_number":3}"#,
        r#"{"type":"content_block_location","cited_text":"block","document_index":0,"document_title":null,"start_block_index":0,"end_block_index":2}"#,
        r#"{"type":"web_search_result_location","cited_text":"search","encrypted_index":"enc","title":"Example","url":"https://example.com"}"#,
        r#"{"type":"search_result_location","cited_text":"found","document_index":0,"document_title":null,"start_char_index":10,"end_char_index":20}"#,
    ];
    for json in citations_json {
        let citation: TextCitation = serde_json::from_str(json).unwrap();
        let reserialized = serde_json::to_string(&citation).unwrap();
        let _: TextCitation = serde_json::from_str(&reserialized).unwrap();
    }
}

#[test]
fn text_citation_variant_dispatch() {
    let char_loc: TextCitation = serde_json::from_str(
        r#"{"type":"char_location","cited_text":"a","document_index":0,"document_title":null,"start_char_index":0,"end_char_index":1}"#
    ).unwrap();
    assert!(matches!(char_loc, TextCitation::CharLocation(_)));

    let page_loc: TextCitation = serde_json::from_str(
        r#"{"type":"page_location","cited_text":"b","document_index":0,"document_title":null,"start_page_number":1,"end_page_number":2}"#
    ).unwrap();
    assert!(matches!(page_loc, TextCitation::PageLocation(_)));

    let block_loc: TextCitation = serde_json::from_str(
        r#"{"type":"content_block_location","cited_text":"c","document_index":0,"document_title":null,"start_block_index":0,"end_block_index":1}"#
    ).unwrap();
    assert!(matches!(block_loc, TextCitation::ContentBlockLocation(_)));

    let web_loc: TextCitation = serde_json::from_str(
        r#"{"type":"web_search_result_location","cited_text":"d","encrypted_index":"e","title":null,"url":"https://x.com"}"#,
    )
    .unwrap();
    assert!(matches!(
        web_loc,
        TextCitation::WebSearchResultLocation(_)
    ));

    let search_loc: TextCitation = serde_json::from_str(
        r#"{"type":"search_result_location","cited_text":"e","document_index":0,"document_title":null,"start_char_index":0,"end_char_index":1}"#
    ).unwrap();
    assert!(matches!(search_loc, TextCitation::SearchResultLocation(_)));
}

// ── DocumentSource (internally tagged) ───────────────────────────────

#[test]
fn roundtrip_document_source_all_variants() {
    let base64 = DocumentSource::Base64(Base64DocumentSource {
        media_type: "application/pdf".to_string(),
        data: "JVBERi0=".to_string(),
    });
    let json = serde_json::to_string(&base64).unwrap();
    let _: DocumentSource = serde_json::from_str(&json).unwrap();

    let text = DocumentSource::Text(PlainTextSource {
        media_type: "text/plain".to_string(),
        data: "hello".to_string(),
    });
    let json = serde_json::to_string(&text).unwrap();
    let _: DocumentSource = serde_json::from_str(&json).unwrap();

    let content = DocumentSource::Content(ContentBlockSource {
        content: vec![TextBlockParam::new("inline")],
    });
    let json = serde_json::to_string(&content).unwrap();
    let _: DocumentSource = serde_json::from_str(&json).unwrap();

    let url = DocumentSource::Url(UrlDocumentSource {
        url: "https://example.com/doc.pdf".to_string(),
    });
    let json = serde_json::to_string(&url).unwrap();
    let _: DocumentSource = serde_json::from_str(&json).unwrap();
}

// ── ImageSource (internally tagged) ──────────────────────────────────

#[test]
fn roundtrip_image_source_all_variants() {
    let base64 = ImageSource::Base64(Base64ImageSource {
        media_type: MediaType::Png,
        data: "iVBOR...".to_string(),
    });
    let json = serde_json::to_string(&base64).unwrap();
    let _: ImageSource = serde_json::from_str(&json).unwrap();

    let url = ImageSource::Url(UrlImageSource {
        url: "https://example.com/img.png".to_string(),
    });
    let json = serde_json::to_string(&url).unwrap();
    let _: ImageSource = serde_json::from_str(&json).unwrap();
}

// ── ServiceTier ──────────────────────────────────────────────────────

#[test]
fn roundtrip_service_tier() {
    roundtrip_json(&ServiceTier::Auto);
    roundtrip_json(&ServiceTier::StandardOnly);
}

// ── MessageParam ─────────────────────────────────────────────────────

#[test]
fn roundtrip_message_param_user_text() {
    let param = MessageParam::user("Hello");
    roundtrip_json(&param);
}

#[test]
fn roundtrip_message_param_assistant_blocks() {
    let param = MessageParam::assistant_blocks(vec![
        ContentBlockParam::Text(TextBlockParam::new("Response")),
        ContentBlockParam::ToolUse(ToolUseBlockParam {
            id: "tu_1".to_string(),
            name: "calc".to_string(),
            input: serde_json::json!({}),
            cache_control: None,
        }),
    ]);
    roundtrip_json(&param);
}

// ── Full Message (response, deserialize only) ────────────────────────

#[test]
fn deserialize_full_message_response() {
    let json = r#"{
        "id": "msg_abc123",
        "type": "message",
        "role": "assistant",
        "content": [
            {"type": "text", "text": "The weather is sunny."},
            {"type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": {"location": "SF"}}
        ],
        "model": "claude-opus-4-6",
        "stop_reason": "tool_use",
        "stop_sequence": null,
        "usage": {
            "input_tokens": 100,
            "output_tokens": 50,
            "cache_creation_input_tokens": 10,
            "cache_read_input_tokens": 5,
            "server_tool_use": {
                "web_search_requests": 2
            }
        }
    }"#;
    let msg: Message = serde_json::from_str(json).unwrap();
    assert_eq!(msg.id, "msg_abc123");
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.content.len(), 2);
    assert_eq!(msg.stop_reason, Some(StopReason::ToolUse));
    assert_eq!(msg.usage.input_tokens, 100);
    assert_eq!(msg.usage.cache_creation_input_tokens, Some(10));
    assert_eq!(
        msg.usage
            .server_tool_use
            .as_ref()
            .unwrap()
            .web_search_requests,
        Some(2)
    );
}

#[test]
fn message_to_param_roundtrip() {
    let json = r#"{
        "id": "msg_1",
        "type": "message",
        "role": "assistant",
        "content": [
            {"type": "text", "text": "Hello!"},
            {"type": "thinking", "thinking": "hmm", "signature": "sig"},
            {"type": "tool_use", "id": "tu_1", "name": "calc", "input": {"x": 1}}
        ],
        "model": "claude-sonnet-4-5",
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {"input_tokens": 10, "output_tokens": 5}
    }"#;
    let msg: Message = serde_json::from_str(json).unwrap();
    let param = msg.to_param();
    let param_json = serde_json::to_string(&param).unwrap();
    // Verify roundtrip of the param
    let _: MessageParam = serde_json::from_str(&param_json).unwrap();
}

// ── WebSearchToolResultContent (untagged) ────────────────────────────

#[test]
fn roundtrip_web_search_tool_result_content() {
    // Results variant
    let results_json = r#"[{"type":"web_search_result","url":"https://example.com","title":"Ex","encrypted_content":"enc"}]"#;
    let content: WebSearchToolResultContent = serde_json::from_str(results_json).unwrap();
    match &content {
        WebSearchToolResultContent::Results(r) => assert_eq!(r.len(), 1),
        _ => panic!("Expected Results"),
    }
    let reserialized = serde_json::to_string(&content).unwrap();
    let _: WebSearchToolResultContent = serde_json::from_str(&reserialized).unwrap();

    // Error variant
    let error_json = r#"{"type":"web_search_error","error_code":"rate_limited"}"#;
    let content: WebSearchToolResultContent = serde_json::from_str(error_json).unwrap();
    match content {
        WebSearchToolResultContent::Error(e) => assert_eq!(e.error_code, "rate_limited"),
        _ => panic!("Expected Error"),
    }
}

// ── CacheControl ─────────────────────────────────────────────────────

#[test]
fn roundtrip_cache_control() {
    let cc = CacheControl::ephemeral();
    roundtrip_json(&cc);
}

// ── Metadata ─────────────────────────────────────────────────────────

#[test]
fn roundtrip_metadata() {
    let meta = Metadata {
        user_id: Some("user_123".to_string()),
    };
    roundtrip_json(&meta);

    let empty = Metadata::default();
    let json = serde_json::to_string(&empty).unwrap();
    assert_eq!(json, "{}");
}

// ── Complex integration: multi-turn conversation ─────────────────────

#[test]
fn roundtrip_multi_turn_conversation() {
    let messages = vec![
        MessageParam::user("What's the weather in SF?"),
        MessageParam::assistant_blocks(vec![
            ContentBlockParam::Text(TextBlockParam::new("Let me check.")),
            ContentBlockParam::ToolUse(ToolUseBlockParam {
                id: "tu_1".to_string(),
                name: "get_weather".to_string(),
                input: serde_json::json!({"location": "San Francisco"}),
                cache_control: None,
            }),
        ]),
        MessageParam {
            role: Role::User,
            content: MessageContent::Blocks(vec![ContentBlockParam::ToolResult(
                ToolResultBlockParam {
                    tool_use_id: "tu_1".to_string(),
                    content: Some(ToolResultContent::Text("72F, sunny".to_string())),
                    is_error: None,
                    cache_control: None,
                },
            )]),
        },
        MessageParam::assistant("It's 72F and sunny in San Francisco!"),
    ];

    for msg in &messages {
        roundtrip_json(msg);
    }

    // Also test as a Vec
    let json = serde_json::to_string(&messages).unwrap();
    let roundtrip: Vec<MessageParam> = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip.len(), 4);
}

// ── ContentBlockParam with cache_control ─────────────────────────────

#[test]
fn roundtrip_content_block_param_with_cache_control() {
    let param = ContentBlockParam::Text(TextBlockParam {
        text: "Cached text".to_string(),
        cache_control: Some(CacheControl::ephemeral()),
        citations: None,
    });
    let json = roundtrip_json(&param);
    assert!(json.contains(r#""cache_control":{""#));
    assert!(json.contains(r#""type":"ephemeral""#));
}

// ── ImageBlockParam in ContentBlockParam ─────────────────────────────

#[test]
fn roundtrip_image_block_param() {
    let param = ContentBlockParam::Image(ImageBlockParam {
        source: ImageSource::Base64(Base64ImageSource {
            media_type: MediaType::Jpeg,
            data: "/9j/4AAQ...".to_string(),
        }),
        cache_control: None,
    });
    roundtrip_json(&param);
}

#[test]
fn roundtrip_image_url_block_param() {
    let param = ContentBlockParam::Image(ImageBlockParam {
        source: ImageSource::Url(UrlImageSource {
            url: "https://example.com/img.jpg".to_string(),
        }),
        cache_control: None,
    });
    roundtrip_json(&param);
}

// ── DocumentBlockParam in ContentBlockParam ──────────────────────────

#[test]
fn roundtrip_document_block_param() {
    let param = ContentBlockParam::Document(DocumentBlockParam {
        source: DocumentSource::Base64(Base64DocumentSource {
            media_type: "application/pdf".to_string(),
            data: "JVBERi0=".to_string(),
        }),
        title: Some("My Document".to_string()),
        context: None,
        citations: Some(CitationsConfig {
            enabled: Some(true),
        }),
        cache_control: None,
    });
    roundtrip_json(&param);
}
