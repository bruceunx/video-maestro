use serde::Deserialize;

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct GeminiCandidate {
    content: GeminiContent,
    // finishReason: Option<String>,
}

#[derive(Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiPart {
    text: String,
}

pub fn parse_gemini(chunk: &str) -> Result<String, String> {
    let gemini_response: GeminiResponse =
        serde_json::from_str(chunk).map_err(|e| format!("JSON parse error: {}", e))?;

    if let Some(candidate) = gemini_response.candidates.first() {
        if let Some(part) = candidate.content.parts.first() {
            Ok(part.text.clone())
        } else {
            Err("No parts found in Gemini response".to_string())
        }
    } else {
        Err("No candidates found in Gemini response".to_string())
    }
}
