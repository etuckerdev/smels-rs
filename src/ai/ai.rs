#[cfg(feature = "ai")]
use ollama_rs::{generation::completion::request::GenerationRequest, models::ModelOptions, Ollama};
use serde::{Deserialize, Serialize};

// Extract the first balanced JSON object from a string; tolerant of leading/trailing prose.
#[cfg(feature = "ai")]
fn extract_first_json_object(s: &str) -> Option<String> {
    let mut in_string = false;
    let mut escape = false;
    let mut depth: i32 = 0;
    let mut start_idx: Option<usize> = None;
    for (i, ch) in s.char_indices() {
        if start_idx.is_none() {
            if ch == '{' {
                start_idx = Some(i);
                depth = 1;
            }
            continue;
        } else {
            match ch {
                '"' if !escape => {
                    in_string = !in_string;
                }
                '\\' if !escape => {
                    escape = true;
                    continue;
                }
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        let start = start_idx.unwrap();
                        return Some(s[start..=i].to_string());
                    }
                }
                _ => {}
            }
            if escape {
                escape = false;
            }
        }
    }
    None
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AISummary {
    pub summary: String,
    pub causes: Vec<AICause>,
    pub fixes: Vec<String>,
    pub references: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AICause {
    pub label: String,
    pub prob: f32,
}

#[cfg(feature = "ai")]
pub async fn ai_summarize(brief: String) -> Result<AISummary, Box<dyn std::error::Error>> {
    let ollama = Ollama::default();

    // Hardened system prompt (few-shot style kept minimal). MUST emit only JSON.
    let system_prompt = r#"You are SMELS. Return ONLY strict JSON: {"summary":"string","causes":[{"label":"string","prob":0.0}],"fixes":["string"],"references":["string"]}. Be concise and technical. No extra text."#;

    // Provide the signal & trimmed log context separately to reduce rambling
    let (signal_block, log_block) = if let Some(split) = brief.split_once("\nFull input:") {
        (split.0, split.1)
    } else {
        (brief.as_str(), "")
    };

    // Trim excessive log to keep token use low
    let trimmed_log = if log_block.len() > 4000 {
        &log_block[..4000]
    } else {
        log_block
    };

    // Combine system prompt with user prompt since raw mode doesn't support system parameter
    let full_prompt = format!(
        "{}\n\nSIGNAL\n{}\nLOG_SNIPPET\n{}",
        system_prompt, signal_block, trimmed_log
    );

    let mut req = GenerationRequest::new("qwen2.5-coder:0.5b-instruct".into(), full_prompt);

    // Low temperature & tight token limit to discourage rambling
    let options = ModelOptions::default()
        .temperature(0.1)
        .num_predict(256)
        .top_p(0.9)
        .repeat_penalty(1.05);
    req = req.options(options);

    let res = ollama.generate(req).await?;
    let raw = res.response.trim();
    let ai_debug = std::env::var("SMELS_AI_DEBUG").is_ok();
    if ai_debug {
        eprintln!(
            "[smels ai-debug] raw AI output: {}",
            raw.replace('\n', " ").chars().take(500).collect::<String>()
        );
    }

    // Attempt fast path parse: direct JSON
    if let Ok(parsed) = serde_json::from_str::<AISummary>(raw) {
        return Ok(parsed);
    }

    // Strip code fences if present
    let fence_stripped = raw
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Clean up any remaining whitespace issues
    let clean_json = fence_stripped
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("")
        .replace("  ", " ")
        .replace(", }", "}")
        .replace(", ]", "]");

    if ai_debug {
        eprintln!(
            "[smels ai-debug] clean_json: {}",
            clean_json.chars().take(200).collect::<String>()
        );
    }

    // Try parsing with a more lenient approach
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&clean_json) {
        if ai_debug {
            eprintln!("[smels ai-debug] Parsed as Value successfully");
        }
        // Try to convert to our struct
        if let Ok(summary) = serde_json::from_value(parsed.clone()) {
            return Ok(summary);
        } else {
            // Manual construction as fallback
            if let Some(obj) = parsed.as_object() {
                let summary = obj
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("AI analysis failed")
                    .to_string();
                let causes: Vec<AICause> = obj
                    .get("causes")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|cause| {
                        if let Some(cause_obj) = cause.as_object() {
                            Some(AICause {
                                label: cause_obj
                                    .get("label")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown")
                                    .to_string(),
                                prob: cause_obj
                                    .get("prob")
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(0.0) as f32,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                let fixes: Vec<String> = obj
                    .get("fixes")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                let references: Vec<String> = obj
                    .get("references")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                let summary_struct = AISummary {
                    summary,
                    causes,
                    fixes,
                    references,
                };

                if ai_debug {
                    eprintln!(
                        "[smels ai-debug] Manually constructed AISummary: {:?}",
                        summary_struct
                    );
                }
                return Ok(summary_struct);
            }
        }
    }

    // Extract first balanced JSON object
    if let Some(obj) = extract_first_json_object(raw) {
        if let Ok(parsed) = serde_json::from_str::<AISummary>(&obj) {
            return Ok(parsed);
        }
    }

    // As last resort, attempt extraction after fence stripping
    if let Some(obj) = extract_first_json_object(fence_stripped) {
        if let Ok(parsed) = serde_json::from_str::<AISummary>(&obj) {
            return Ok(parsed);
        }
    }

    Err("AI feature not enabled".into())
}

#[cfg(not(feature = "ai"))]
pub async fn ai_summarize(_brief: String) -> Result<AISummary, Box<dyn std::error::Error>> {
    Err("AI feature not enabled - compile with --features ai to enable AI analysis".into())
}
