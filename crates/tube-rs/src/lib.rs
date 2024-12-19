use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
    Client, Proxy,
};
use serde::{Deserialize, Serialize};

pub struct YoutubeAudio {
    client: Client,
}

#[derive(Debug, Serialize)]
struct ClientInfo {
    client_name: String,
    client_version: String,
    device_make: String,
    device_model: String,
    os_name: String,
    os_version: String,
    android_sdk_version: String,
}

#[derive(Serialize, Debug)]
struct RequestContext {
    client: ClientInfo,
}

#[derive(Serialize, Debug)]
struct RequestBody {
    context: RequestContext,
    video_id: String,
    #[serde(rename = "contentCheckOk")]
    content_check_ok: String,
}

impl ClientInfo {
    pub fn default() -> Self {
        Self {
            client_name: "ANDROID_VR".to_string(),
            client_version: "1.60.19".to_string(),
            device_make: "Oculus".to_string(),
            device_model: "Quest 3".to_string(),
            os_name: "Android".to_string(),
            os_version: "12L".to_string(),
            android_sdk_version: "32".to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResponseBody {
    streaming_data: StreamingData,
    captions: Option<Captions>,
    video_details: VideoDetail,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct VideoDetail {
    video_id: String,
    title: String,
    length_seconds: String,
    keywords: Option<Vec<String>>,
    short_description: Option<String>,
    thumbnail: ThumbNail,
}

#[derive(Deserialize, Debug)]
struct ThumbNail {
    thumbnails: Vec<ThumbNailItem>,
}

#[derive(Deserialize, Debug)]
struct ThumbNailItem {
    url: String,
    width: u32,
    height: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Captions {
    player_captions_tracklist_renderer: PlayerCaptionsTracklistRenderer,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PlayerCaptionsTracklistRenderer {
    caption_tracks: Vec<CaptionItem>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CaptionItem {
    base_url: String,
    vss_id: String,
    language_code: String,
}

#[derive(Debug, Deserialize)]
struct Format {
    #[serde(rename = "mimeType")]
    mime_type: String,
    bitrate: u32,
    url: String,
    #[serde(rename = "contentLength")]
    content_length: String,
}

#[derive(Debug, Deserialize)]
struct StreamingData {
    formats: Option<Vec<Format>>,
    #[serde(rename = "adaptiveFormats")]
    adaptive_formats: Option<Vec<Format>>,
}

pub fn extract_id(url: &str) -> Option<String> {
    let re =
        Regex::new(r"(?:v=|\/v\/|youtu\.be\/|\/embed\/|\/shorts\/)([A-Za-z0-9_-]{11})").unwrap();

    if let Some(captures) = re.captures(url) {
        return captures.get(1).map(|m| m.as_str().to_string());
    }

    None
}

pub fn extract_xml_captions(content: &str) -> Option<Vec<String>> {
    todo!()
}

impl YoutubeAudio {
    pub fn new(proxy: Option<&str>) -> Self {
        let client_builder = Client::builder();
        let client = match proxy {
            Some(proxy_str) => match Proxy::https(proxy_str) {
                Ok(ok_proxy) => client_builder.proxy(ok_proxy).build().unwrap(),
                Err(_) => client_builder.build().unwrap(),
            },
            None => client_builder.build().unwrap(),
        };
        Self { client }
    }

    pub async fn get_video_info(&self, url: &str) -> Option<ResponseBody> {
        let video_id = match extract_id(url) {
            Some(_id) => _id,
            None => return None,
        };
        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_static("com.google.android.apps.youtube.vr.oculus/1.60.19 (Linux; U; Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let request_body = RequestBody {
            context: RequestContext {
                client: ClientInfo::default(),
            },
            video_id: video_id.to_string(),
            content_check_ok: "true".to_string(),
        };

        let response = self
            .client
            .post("https://www.youtube.com/youtubei/v1/player?prettyPrint=false")
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .ok()?;

        let response_data: ResponseBody = match response.json().await {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        };
        Some(response_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    #[tokio::test]
    async fn check_response_body_works() {
        dotenv().ok();
        let proxy = env::var("PROXY").ok();
        let youtube_client = YoutubeAudio::new(proxy.as_deref());
        let url = "https://www.youtube.com/watch?v=Q0cvzaPJJas&ab_channel=TJDeVries";
        let response = youtube_client.get_video_info(url).await;
        assert!(response.is_some());
        let response_data = response.unwrap();
        assert_eq!(
            response_data.video_details.video_id,
            "Q0cvzaPJJas".to_string()
        );

        let captions = response_data
            .captions
            .unwrap()
            .player_captions_tracklist_renderer
            .caption_tracks;

        assert_eq!(captions.len(), 1);
        assert_eq!(captions[0].vss_id, "a.en".to_string()); // "a.en" for en auto-generated
    }

    #[test]
    fn extract_id_works() {
        let test_cases = vec![
            (
                "https://youtu.be/FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Short YouTube URL
            (
                "https://www.youtube.com/watch?v=FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Standard YouTube URL
            (
                "https://youtube.com/watch?v=FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // YouTube without 'www'
            (
                "https://www.youtube.com/v/FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // /v/ style URL
            (
                "https://www.youtube.com/embed/FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Embedded URL
            (
                "https://www.youtube.com/shorts/FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Shorts URL
            (
                "https://youtu.be/FdeioVndUhs?t=30",
                Some("FdeioVndUhs".to_string()),
            ), // URL with timestamp
            (
                "https://www.youtube.com/watch?v=FdeioVndUhs&feature=share",
                Some("FdeioVndUhs".to_string()),
            ), // With additional params
            (
                "https://www.youtube.com/watch?v=FdeioVndUhs&list=PL123",
                Some("FdeioVndUhs".to_string()),
            ), // With playlist
            (
                "https://www.youtube.com/watch?list=PL123&v=FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Playlist first
            ("https://invalid.url/FdeioVndUhs", None), // Invalid URL
            ("https://www.youtube.com/", None),        // Homepage
            ("https://youtu.be/", None),               // Invalid Short URL
        ];

        for (input, expected) in test_cases {
            let video_id = extract_id(input);
            assert_eq!(video_id, expected);
        }
    }
}
