use crate::error::{Result, VideoError};
use crate::scene::Scene;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

pub struct VideoGenerator {
    output_dir: String,
}

impl VideoGenerator {
    pub fn new(output_dir: String) -> Self {
        Self { output_dir }
    }

    /// 合成最终视频
    pub async fn generate_video(
        &self,
        scenes: &[Scene],
        audio_path: &str,
        output_path: &str,
    ) -> Result<()> {
        info!("Starting video generation...");

        // 创建临时文件列表
        let concat_file = format!("{}/concat.txt", self.output_dir);
        let mut concat_content = String::new();

        // 为每个场景创建带字幕的视频片段
        let mut segment_paths = Vec::new();

        for scene in scenes {
            if let Some(image_path) = &scene.image_path {
                let segment_path = format!("{}/segment_{}.mp4", self.output_dir, scene.index);
                
                // 使用FFmpeg创建视频片段：图片 + 字幕
                self.create_video_segment(
                    image_path,
                    &scene.subtitle,
                    scene.duration,
                    &segment_path,
                )
                .await?;

                // 转换为绝对路径
                let abs_segment_path = PathBuf::from(&segment_path)
                    .canonicalize()
                    .map_err(|e| VideoError::VideoGenerationError(format!("Failed to get absolute path: {}", e)))?;
                
                concat_content.push_str(&format!("file '{}'\n", abs_segment_path.display()));
                segment_paths.push(segment_path);
            }
        }

        // 写入concat文件
        tokio::fs::write(&concat_file, concat_content).await?;

        // 合并所有视频片段
        let merged_video = format!("{}/merged.mp4", self.output_dir);
        self.concat_videos(&concat_file, &merged_video).await?;

        // 添加音频
        self.add_audio(&merged_video, audio_path, output_path)
            .await?;

        info!("Video generation completed: {}", output_path);

        // 清理临时文件
        tokio::fs::remove_file(&concat_file).await.ok();
        tokio::fs::remove_file(&merged_video).await.ok();
        for segment in segment_paths {
            tokio::fs::remove_file(&segment).await.ok();
        }

        Ok(())
    }

    async fn create_video_segment(
        &self,
        image_path: &str,
        subtitle: &str,
        duration: f64,
        output_path: &str,
    ) -> Result<()> {
        info!("Creating video segment for: {}", subtitle);

        // 转义字幕文本中的特殊字符
        let escaped_subtitle = subtitle
            .replace('\\', "\\\\")
            .replace('\'', "'\\''")
            .replace(':', "\\:")
            .replace(',', "\\,");

        // 使用FFmpeg创建带字幕的视频片段
        // -loop 1: 循环图片
        // -i: 输入图片
        // -vf: 视频过滤器，添加字幕
        // -t: 持续时间
        // -pix_fmt yuv420p: 像素格式，确保兼容性
        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-loop",
                "1",
                "-i",
                image_path,
                "-vf",
                &format!(
                    "drawtext=text='{}':fontfile=/System/Library/Fonts/PingFang.ttc:fontsize=48:fontcolor=white:x=(w-text_w)/2:y=h-100:box=1:boxcolor=black@0.5:boxborderw=10",
                    escaped_subtitle
                ),
                "-t",
                &duration.to_string(),
                "-pix_fmt",
                "yuv420p",
                "-r",
                "30",
                output_path,
            ])
            .output()
            .map_err(|e| VideoError::VideoGenerationError(format!("Failed to run FFmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VideoError::VideoGenerationError(format!(
                "FFmpeg segment creation failed: {}",
                error
            )));
        }

        info!("Created segment: {}", output_path);
        Ok(())
    }

    async fn concat_videos(&self, concat_file: &str, output_path: &str) -> Result<()> {
        info!("Concatenating video segments...");

        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                concat_file,
                "-c",
                "copy",
                output_path,
            ])
            .output()
            .map_err(|e| VideoError::VideoGenerationError(format!("Failed to run FFmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VideoError::VideoGenerationError(format!(
                "FFmpeg concat failed: {}",
                error
            )));
        }

        info!("Concatenated video: {}", output_path);
        Ok(())
    }

    async fn add_audio(&self, video_path: &str, audio_path: &str, output_path: &str) -> Result<()> {
        info!("Adding audio to video...");

        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                video_path,
                "-i",
                audio_path,
                "-c:v",
                "copy",
                "-c:a",
                "aac",
                "-map",
                "0:v:0",
                "-map",
                "1:a:0",
                "-shortest",
                output_path,
            ])
            .output()
            .map_err(|e| VideoError::VideoGenerationError(format!("Failed to run FFmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VideoError::VideoGenerationError(format!(
                "FFmpeg audio merge failed: {}",
                error
            )));
        }

        info!("Added audio to video: {}", output_path);
        Ok(())
    }
}
