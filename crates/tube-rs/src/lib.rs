use quick_xml::{events::Event, Reader};
use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE, USER_AGENT},
    Client, Proxy,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::Write, path::Path};

pub struct YoutubeAudio {
    client: Client,
}

#[derive(Serialize)]
struct ClientInfo {
    client_name: String,
    client_version: String,
    device_make: String,
    device_model: String,
    os_name: String,
    os_version: String,
    android_sdk_version: String,
}

#[derive(Serialize)]
struct RequestContext {
    client: ClientInfo,
}

#[derive(Serialize)]
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseBody {
    streaming_data: StreamingData,
    captions: Option<Captions>,
    video_details: VideoDetail,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoDetail {
    // video_id: String,
    title: String,
    length_seconds: String,
    keywords: Option<Vec<String>>,
    short_description: Option<String>,
    // thumbnail: ThumbNail,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Captions {
    player_captions_tracklist_renderer: PlayerCaptionsTracklistRenderer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerCaptionsTracklistRenderer {
    caption_tracks: Vec<CaptionItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaptionItem {
    base_url: String,
    vss_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Format {
    mime_type: String,
    bitrate: u32,
    url: String,
    content_length: String,
    last_modified: String,
}

#[derive(Deserialize)]
struct StreamingData {
    // formats: Option<Vec<Format>>,
    #[serde(rename = "adaptiveFormats")]
    adaptive_formats: Option<Vec<Format>>,
}

// export the struct
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudioData {
    pub video_id: String,
    pub title: String,
    pub duration: u64,
    pub timestamp: u64,
    pub keywords: Option<Vec<String>>,
    pub description: Option<String>,
    pub caption_lang: Option<String>,
    pub caption_url: Option<String>,
    pub audio_url: String,
    pub audio_filesize: u64,
    pub thumbnail_url: String,
    pub mime_type: String,
}

pub struct SubtitleEntry {
    pub timestamp: u64,
    pub duration: u32,
    pub text: String,
}

fn extract_id(url: &str) -> Option<String> {
    let re = Regex::new(r"(?:v=|\/v\/|youtu\.be\/|\/embed\/|\/shorts\/)([A-Za-z0-9_-]+)").unwrap();

    if let Some(captures) = re.captures(url) {
        return captures.get(1).map(|m| m.as_str().to_string());
    }
    None
}
fn parse_xml_with_auto(xml_string: &str) -> Result<Vec<SubtitleEntry>, Box<dyn Error>> {
    let mut reader = Reader::from_str(xml_string);
    reader.config_mut().trim_text(false);

    let mut subtitles = Vec::new();
    let mut current_subtitle: Option<SubtitleEntry> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"p" {
                    let mut timestamp = 0;
                    let mut duration = 0;
                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            b"t" => timestamp = attr.unescape_value()?.parse()?,
                            b"d" => duration = attr.unescape_value()?.parse()?,
                            _ => {}
                        }
                    }
                    current_subtitle = Some(SubtitleEntry {
                        timestamp,
                        duration,
                        text: String::new(),
                    });
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(ref mut subtitle) = current_subtitle {
                    subtitle.text.push_str(&e.unescape()?);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"p" {
                    if let Some(subtitle) = current_subtitle.take() {
                        if !subtitle.text.is_empty() {
                            subtitles.push(subtitle);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
    }

    Ok(subtitles)
}

fn parse_xml(xml: &str) -> Result<Vec<SubtitleEntry>, Box<dyn Error>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut subtitles = Vec::new();
    let mut current_text = String::new();
    let mut current_timestamp = 0;
    let mut current_duration = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() == b"p" => {
                current_text.clear();
                for attr in e.attributes() {
                    let attr = attr?;
                    match attr.key.as_ref() {
                        b"t" => current_timestamp = attr.unescape_value()?.parse()?,
                        b"d" => current_duration = attr.unescape_value()?.parse()?,
                        _ => {}
                    }
                }
            }
            Ok(Event::Text(e)) => {
                current_text.push_str(&e.unescape()?);
            }
            Ok(Event::End(e)) if e.name().as_ref() == b"p" => {
                if !current_text.is_empty() {
                    subtitles.push(SubtitleEntry {
                        timestamp: current_timestamp,
                        duration: current_duration,
                        text: current_text.trim().to_string(),
                    })
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
    }
    Ok(subtitles)
}

impl YoutubeAudio {
    pub fn new(proxy: Option<&str>) -> Self {
        let client_builder = Client::builder();
        let client = match proxy {
            Some(proxy_str) => match Proxy::https(proxy_str) {
                Ok(ok_proxy) => client_builder.proxy(ok_proxy).build().unwrap(),
                Err(_) => client_builder.build().unwrap(),
            },
            _ => client_builder.build().unwrap(),
        };
        Self { client }
    }

    pub async fn get_video_info(&self, url: &str) -> Option<AudioData> {
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

        let mut all_formats = Vec::new();

        if let Some(adaptive_formats) = response_data.streaming_data.adaptive_formats {
            all_formats.extend(adaptive_formats);
        }

        let (mime_type, last_modified, audio_url, audio_filesize) = match all_formats
            .into_iter()
            .filter(|format| format.mime_type.starts_with("audio"))
            .min_by_key(|format| format.bitrate)
        {
            Some(format) => (
                format.mime_type,
                format.last_modified.parse::<u64>().unwrap(),
                format.url,
                format.content_length.parse::<u64>().ok().unwrap_or(0),
            ),

            _ => return None,
        };

        let (caption_url, caption_lang) = match response_data.captions {
            Some(captions) => {
                let caption_array = captions.player_captions_tracklist_renderer.caption_tracks;

                let caption = if caption_array.len() > 1 {
                    let caption_en = caption_array.iter().find(|item| item.vss_id.contains("en"));
                    match caption_en {
                        Some(caption) => caption,
                        None => &caption_array[0],
                    }
                } else {
                    &caption_array[0]
                };

                (Some(caption.base_url.clone()), Some(caption.vss_id.clone()))
            }
            _ => (None, None),
        };

        let thumbnail_url = format!("https://i.ytimg.com/vi/{}/sddefault.jpg", video_id);

        Some(AudioData {
            video_id,
            title: response_data.video_details.title,
            duration: response_data
                .video_details
                .length_seconds
                .parse::<u64>()
                .unwrap(),
            timestamp: last_modified,
            keywords: response_data.video_details.keywords,
            description: response_data.video_details.short_description,
            caption_lang,
            caption_url,
            audio_url,
            audio_filesize,
            mime_type,
            thumbnail_url,
        })
    }

    pub async fn download_caption(
        &self,
        caption_url: &str,
        caption_lang: &str,
    ) -> Result<Vec<SubtitleEntry>, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-us,en"));
        let response = self.client.get(caption_url).headers(headers).send().await?;
        let xml = response.text().await?;
        if caption_lang.contains("a.") {
            parse_xml_with_auto(xml.as_ref())
        } else {
            parse_xml(xml.as_ref())
        }
    }

    pub async fn download_audio(
        &self,
        audio_url: &str,
        file_size: u64,
        file_path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-us,en"));
        let mut file = File::create(file_path)?;
        let mut downloaded = 0;
        let default_range_size = 1024 * 1024 * 9;
        while downloaded < file_size {
            let stop_pos = (downloaded + default_range_size).min(file_size) - 1;
            let chunk_reponse = self
                .client
                .get(format!("{}?range={}-{}", audio_url, downloaded, stop_pos))
                .headers(headers.clone())
                .send()
                .await?;

            let chunk = chunk_reponse.bytes().await?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::path::PathBuf;
    use std::{env, str::FromStr};

    #[tokio::test]
    async fn check_caption_lang_works() {
        dotenv().ok();
        let proxy = env::var("PROXY").ok();
        let youtube_client = YoutubeAudio::new(proxy.as_deref());
        let url = "https://www.youtube.com/watch?v=2p_Hlm6aCok&ab_channel=TheoriesofEverythingwithCurtJaimungal";
        let video_data = youtube_client.get_video_info(url).await;
        assert!(video_data.is_some());
        let video = video_data.unwrap();
        assert!(video.caption_lang.unwrap().contains("en"));
    }

    #[tokio::test]
    async fn check_response_body_works() {
        dotenv().ok();
        let proxy = env::var("PROXY").ok();
        let youtube_client = YoutubeAudio::new(proxy.as_deref());
        let url = "https://www.youtube.com/watch?v=s78hvV3QLUE&ab_channel=LexFridman";
        let video_data = youtube_client.get_video_info(url).await;
        assert!(video_data.is_some());
        // let video = video_data.unwrap();
        // assert_eq!(video.caption_lang.unwrap(), "a.en".to_string());
        // assert!(video.timestamp > 0);
    }

    #[tokio::test]
    async fn check_download_audio_works() {
        dotenv().ok();
        let proxy = env::var("PROXY").ok();
        let youtube_client = YoutubeAudio::new(proxy.as_deref());
        let url = "https://www.youtube.com/watch?v=s78hvV3QLUE&t=4s"; //"https://www.youtube.com/watch?v=Q0cvzaPJJas&ab_channel=TJDeVries";
        let video_data = youtube_client.get_video_info(url).await;
        assert!(video_data.is_some());

        let video = video_data.unwrap();

        let audio_url = &video.audio_url;
        let audio_length = video.audio_filesize;
        let file_path = if video.mime_type.contains("webm") {
            PathBuf::from_str("./sample.webm").unwrap()
        } else {
            PathBuf::from_str("./sample.m4a").unwrap()
        };
        let download = youtube_client
            .download_audio(audio_url, audio_length, &file_path)
            .await;

        assert!(download.is_ok());
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
                "https://www.youtube.com/watch?v=s78hvV3QLUE&ab_channel=LexFridman",
                Some("s78hvV3QLUE".to_string()),
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

    #[test]
    fn parse_xml_works() {
        let xml = r#"
            <?xml version="1.0" encoding="utf-8" ?><timedtext format="3">
            <body>
            <p t="80" d="7520">我们生活在弦理论描述的错误世界中。从未有物理学家</p>
            <p t="7600" d="5160">因弦理论获得过大奖。我可以绝对肯定地告诉你，这不是</p>
            <p t="12760" d="9200">我们生活的现实世界。所以我们需要重新开始。 弦理论创始人之一、斯坦福大学理论物理学教授</p>
            <p t="21960" d="4800">伦纳德·苏斯金德 (Leonard Susskind) 做出了一项开创性的揭露，</p>
            <p t="26760" d="7480">他做出了令人震惊的承认。弦理论失败了，许多物理学家不知道</p>
            <p t="34240" d="5440">该怎么办。他们很害怕。他们可能找不到工作。曾经承诺统一物理学的理论</p>
            <p t="39680" d="7560">却产生了 10 到 500 种不同的可能解决方案或真空状态。</p>
            <p t="47240" d="5200">一些理论家认为，每个数学解决方案都对应一个物理现实，即</p>
            </body>
            </timedtext>
        "#;
        let result = parse_xml(&xml);
        assert!(result.is_ok());
        let subtitles = result.unwrap();
        assert_eq!(subtitles.len(), 8);
        assert_eq!(subtitles[0].timestamp, 80);
        assert_eq!(subtitles[subtitles.len() - 1].duration, 5200);
    }

    #[test]
    fn parse_xml_auto_works() {
        let xml = r#"
            <?xml version="1.0" encoding="utf-8" ?><timedtext format="3">
            <head>
            <ws id="0"/>
            <ws id="1" mh="2" ju="0" sd="3"/>
            <wp id="0"/>
            <wp id="1" ap="6" ah="20" av="100" rc="2" cc="40"/>
            </head>
            <body>
            <w t="0" id="1" wp="1" ws="1"/>
            <p t="6839" d="4680" w="1"><s ac="0">dark</s><s t="321" ac="0"> energy</s><s t="801" ac="0"> is</s><s t="960" ac="0"> one</s><s t="1080" ac="0"> of</s><s t="1241" ac="0"> the</s><s t="1401" ac="0"> biggest</s></p>
            <p t="8629" d="2890" w="1" a="1">
            </p>
            <p t="8639" d="6120" w="1"><s ac="0">mysteries</s><s t="601" ac="0"> in</s><s t="1040" ac="0"> physics</s><s t="2040" ac="0"> it</s><s t="2241" ac="0"> dominates</s><s t="2761" ac="0"> the</s></p>
            <p t="11509" d="3250" w="1" a="1">
            </p>
            </body>
            </timedtext>
        "#;
        let result = parse_xml(&xml);
        assert!(result.is_ok());
        let subtitles = result.unwrap();
        assert_eq!(subtitles.len(), 2);
    }
}
