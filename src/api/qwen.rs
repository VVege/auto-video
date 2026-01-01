use crate::error::{Result, VideoError};
use crate::scene::Scene;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::{info, warn};

const QWEN_TEXT_API: &str = "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation";
const QWEN_IMAGE_API: &str = "https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis";
const QWEN_TTS_API: &str = "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation";

#[derive(Debug, Clone)]
pub struct QwenClient {
    api_key: String,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct TextGenerationResponse {
    output: TextOutput,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct TextOutput {
    text: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    total_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ImageGenerationResponse {
    output: ImageOutput,
}

#[derive(Debug, Deserialize)]
struct ImageOutput {
    task_id: String,
    task_status: String,
}

#[derive(Debug, Deserialize)]
struct ImageTaskResponse {
    output: ImageTaskOutput,
}

#[derive(Debug, Deserialize)]
struct ImageTaskOutput {
    task_status: String,
    results: Option<Vec<ImageResult>>,
}

#[derive(Debug, Deserialize)]
struct ImageResult {
    url: String,
}

#[derive(Debug, Deserialize)]
struct TTSResponse {
    output: TTSOutput,
}

#[derive(Debug, Deserialize)]
struct TTSOutput {
    audio: AudioResult,
}

#[derive(Debug, Deserialize)]
struct AudioResult {
    url: String,
}

impl QwenClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");

        Self { api_key, client }
    }

    /// 使用千问大模型分析文本并生成分镜
    pub async fn generate_scenes(&self, text: &str) -> Result<Vec<Scene>> {
        info!("Generating scenes from text using Qwen...");

        let prompt = format!(
            r#"请将以下文本分解为视频分镜脚本。每个分镜包含：
1. 场景描述（用于生成图片的提示词，使用英文，详细描述画面内容）
2. 对应的台词或字幕（保持原文）
3. 该场景的建议时长（秒）

请以JSON数组格式返回，每个元素包含：description（英文图片描述）、subtitle（中文字幕）、duration（数字）

文本内容：
{}

直接返回JSON数组，不要其他说明文字。"#,
            text
        );

        let request_body = json!({
            "model": "qwen-plus",
            "input": {
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ]
            },
            "parameters": {
                "result_format": "message"
            }
        });

        let response = self
            .client
            .post(QWEN_TEXT_API)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(VideoError::ApiError(format!(
                "Qwen API error: {}",
                error_text
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        
        // 提取生成的文本
        let generated_text = response_json["output"]["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| VideoError::ApiError("Failed to extract generated text".to_string()))?;

        info!("Generated scenes text: {}", generated_text);

        // 清理可能的markdown标记
        let json_text = generated_text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        #[derive(Deserialize)]
        struct SceneData {
            description: String,
            subtitle: String,
            duration: f64,
        }

        let scene_data: Vec<SceneData> = serde_json::from_str(json_text)
            .map_err(|e| VideoError::ApiError(format!("Failed to parse scenes JSON: {}", e)))?;

        let scenes: Vec<Scene> = scene_data
            .into_iter()
            .enumerate()
            .map(|(i, data)| Scene::new(i, data.description, data.subtitle, data.duration))
            .collect();

        info!("Successfully generated {} scenes", scenes.len());
        Ok(scenes)
    }

    /// 生成图片
    pub async fn generate_image(&self, prompt: &str, output_path: &str) -> Result<()> {
        info!("Generating image for prompt: {}", prompt);

        let request_body = json!({
            "model": "wanx-v1",
            "input": {
                "prompt": prompt
            },
            "parameters": {
                "style": "<photography>",
                "size": "1280*720",
                "n": 1
            }
        });

        // 提交任务
        let response = self
            .client
            .post(QWEN_IMAGE_API)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("X-DashScope-Async", "enable")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(VideoError::ApiError(format!(
                "Image generation API error: {}",
                error_text
            )));
        }

        let task_response: ImageGenerationResponse = response.json().await?;
        let task_id = task_response.output.task_id;

        info!("Image generation task submitted: {}", task_id);

        // 轮询任务状态
        let image_url = self.wait_for_image_task(&task_id).await?;

        // 下载图片
        info!("Downloading image from: {}", image_url);
        let image_data = self.client.get(&image_url).send().await?.bytes().await?;

        tokio::fs::write(output_path, image_data).await?;
        info!("Image saved to: {}", output_path);

        Ok(())
    }

    async fn wait_for_image_task(&self, task_id: &str) -> Result<String> {
        // 千问图片生成任务查询 API
        let query_url = "https://dashscope.aliyuncs.com/api/v1/tasks";
        let max_retries = 60; // 最多等待5分钟
        let retry_interval = Duration::from_secs(5);

        for i in 0..max_retries {
            tokio::time::sleep(retry_interval).await;

            // 使用正确的任务查询 API
            let get_url = format!("{}/{}", query_url, task_id);
            
            info!("Querying task status: {}", get_url);

            let response = self
                .client
                .get(&get_url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await?;
                warn!("Failed to get task status (HTTP {}): {}", status, error_text);
                continue;
            }

            let response_text = response.text().await?;
            info!("Task response: {}", response_text);
            
            let task_result: ImageTaskResponse = serde_json::from_str(&response_text)
                .map_err(|e| VideoError::ApiError(format!("Failed to parse task response: {}", e)))?;

            match task_result.output.task_status.as_str() {
                "SUCCEEDED" => {
                    if let Some(results) = task_result.output.results {
                        if let Some(first_result) = results.first() {
                            return Ok(first_result.url.clone());
                        }
                    }
                    return Err(VideoError::ApiError("No image URL in response".to_string()));
                }
                "FAILED" => {
                    return Err(VideoError::ApiError("Image generation failed".to_string()));
                }
                _ => {
                    info!("Task status: {} (retry {}/{})", task_result.output.task_status, i + 1, max_retries);
                }
            }
        }

        Err(VideoError::ApiError("Image generation timeout".to_string()))
    }

    /// 生成语音
    pub async fn generate_speech(&self, text: &str, output_path: &str) -> Result<()> {
        info!("Generating speech for text (length: {} chars)...", text.len());

        // TTS API 限制：汉字按2个字符计算，最多600字符
        // 保守起见，按最多250个汉字（500字符）来分段
        const MAX_CHUNK_SIZE: usize = 250;
        
        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();
        
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < total_chars {
            let end = std::cmp::min(start + MAX_CHUNK_SIZE, total_chars);
            let chunk: String = chars[start..end].iter().collect();
            chunks.push(chunk);
            start = end;
        }

        if chunks.len() > 1 {
            info!("Text too long, splitting into {} chunks", chunks.len());
            for (i, chunk) in chunks.iter().enumerate() {
                info!("Chunk {} length: {} chars", i + 1, chunk.chars().count());
            }
        }

        let mut audio_files = Vec::new();

        // 为每个分段生成音频
        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_file = if chunks.len() == 1 {
                output_path.to_string()
            } else {
                format!("{}.part{}.mp3", output_path, i)
            };

            info!("Generating speech chunk {}/{} ({} chars)", i + 1, chunks.len(), chunk.chars().count());
            
            let request_body = json!({
                "model": "qwen3-tts-flash",
                "input": {
                    "text": chunk
                },
                "parameters": {
                    "voice": "Cherry",
                    "format": "wav",       // API 实际返回 WAV
                    "sample_rate": 24000
                }
            });

            let response = self
                .client
                .post(QWEN_TTS_API)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(VideoError::ApiError(format!(
                    "TTS API error (chunk {} length {}): {}",
                    i + 1, chunk.chars().count(), error_text
                )));
            }

            // 解析响应获取音频 URL
            let response_json: serde_json::Value = response.json().await?;
            
            // qwen3-tts-flash 返回的结构是 output.audio.url
            let audio_url = response_json["output"]["audio"]["url"]
                .as_str()
                .ok_or_else(|| VideoError::ApiError(format!(
                    "No audio URL in response. Full response: {}", 
                    serde_json::to_string(&response_json).unwrap_or_default()
                )))?;

            // 下载音频文件
            info!("Downloading audio chunk from: {}", audio_url);
            let audio_data = self.client.get(audio_url).send().await?.bytes().await?;
            tokio::fs::write(&chunk_file, audio_data).await?;
            
            audio_files.push(chunk_file);
        }

        // 如果有多个分段，需要合并
        if audio_files.len() > 1 {
            info!("Merging {} audio chunks...", audio_files.len());
            self.merge_audio_files(&audio_files, output_path).await?;
            
            // 删除临时文件
            for file in audio_files {
                tokio::fs::remove_file(file).await.ok();
            }
        }

        info!("Speech saved to: {}", output_path);
        Ok(())
    }

    /// 合并多个音频文件
    async fn merge_audio_files(&self, files: &[String], output: &str) -> Result<()> {
        use std::process::Command;
        use std::path::PathBuf;
        
        // 创建 FFmpeg concat 列表文件
        let concat_list = format!("{}.concat.txt", output);
        let mut content = String::new();
        for file in files {
            // 转换为绝对路径
            let abs_path = PathBuf::from(file)
                .canonicalize()
                .map_err(|e| VideoError::ApiError(format!("Failed to get absolute path for {}: {}", file, e)))?;
            content.push_str(&format!("file '{}'\n", abs_path.display()));
        }
        tokio::fs::write(&concat_list, content).await?;

        // 使用 FFmpeg 合并音频，并转换为 MP3
        let output_cmd = Command::new("ffmpeg")
            .args([
                "-y",
                "-f", "concat",
                "-safe", "0",
                "-i", &concat_list,
                "-c:a", "libmp3lame",  // 使用 MP3 编码器
                "-b:a", "192k",        // 比特率
                output,
            ])
            .output()
            .map_err(|e| VideoError::ApiError(format!("Failed to merge audio: {}", e)))?;

        if !output_cmd.status.success() {
            let error = String::from_utf8_lossy(&output_cmd.stderr);
            return Err(VideoError::ApiError(format!("FFmpeg merge failed: {}", error)));
        }

        // 删除临时列表文件
        tokio::fs::remove_file(&concat_list).await.ok();
        
        Ok(())
    }
}
