mod api;
mod error;
mod scene;
mod video;

use anyhow::Context;
use api::QwenClient;
use clap::Parser;
use error::Result;
use scene::Scene;
use tracing::{error, info};
use video::VideoGenerator;

#[derive(Parser, Debug)]
#[command(name = "auto-video")]
#[command(about = "Automatic video generation tool using AI", long_about = None)]
struct Args {
    /// Input text for video generation
    #[arg(short, long)]
    text: Option<String>,

    /// Input text file path
    #[arg(short, long)]
    file: Option<String>,

    /// Output video file path
    #[arg(short, long, default_value = "output.mp4")]
    output: String,

    /// Working directory for temporary files
    #[arg(short = 'w', long, default_value = "./output")]
    work_dir: String,

    /// Skip image generation (use existing images)
    #[arg(long)]
    skip_images: bool,

    /// DashScope API key
    #[arg(long)]
    api_key: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();

    // 加载环境变量
    dotenvy::dotenv().ok();

    // 解析命令行参数
    let args = Args::parse();

    // 获取 API key
    let api_key = if let Some(key) = args.api_key {
        key
    } else if let Ok(key) = std::env::var("DASHSCOPE_API_KEY") {
        key
    } else {
        eprintln!("Error: DASHSCOPE_API_KEY not found. Please set it via --api-key or DASHSCOPE_API_KEY environment variable");
        std::process::exit(1);
    };

    // 获取输入文本
    let input_text = if let Some(text) = args.text {
        text
    } else if let Some(file_path) = args.file {
        tokio::fs::read_to_string(&file_path)
            .await
            .context(format!("Failed to read file: {}", file_path))?
    } else {
        eprintln!("Error: Either --text or --file must be provided");
        std::process::exit(1);
    };

    info!("Starting auto-video generation...");
    info!("Input text length: {} characters", input_text.len());

    // 创建工作目录
    tokio::fs::create_dir_all(&args.work_dir)
        .await
        .context("Failed to create work directory")?;

    // 运行视频生成流程
    if let Err(e) = run_generation(input_text, api_key, args.work_dir, args.output, args.skip_images).await {
        error!("Video generation failed: {}", e);
        std::process::exit(1);
    }

    info!("Video generation completed successfully!");
    Ok(())
}

async fn run_generation(
    input_text: String,
    api_key: String,
    work_dir: String,
    output_path: String,
    skip_images: bool,
) -> Result<()> {
    // 1. 创建千问客户端
    let client = QwenClient::new(api_key);

    // 2. 生成分镜或使用现有图片
    let mut scenes = if skip_images {
        info!("Skipping scene generation, using existing images...");
        // 先生成分镜以获取字幕文本
        info!("Generating scenes for subtitles...");
        let mut scenes = client.generate_scenes(&input_text).await?;
        info!("Generated {} scenes", scenes.len());
        
        // 使用现有图片
        for scene in scenes.iter_mut() {
            let image_path = format!("{}/scene_{}.png", work_dir, scene.index);
            if tokio::fs::metadata(&image_path).await.is_ok() {
                scene.image_path = Some(image_path);
            }
        }
        
        scenes
    } else {
        info!("Step 1/4: Generating scenes...");
        let scenes = client.generate_scenes(&input_text).await?;
        info!("Generated {} scenes", scenes.len());
        scenes
    };

    // 3. 为每个分镜生成图片（支持断点续传）
    if !skip_images {
        info!("Step 2/4: Generating images for each scene...");
        let scene_count = scenes.len();
        for (idx, scene) in scenes.iter_mut().enumerate() {
            let image_path = format!("{}/scene_{}.png", work_dir, scene.index);
            
            // 检查图片是否已存在，跳过已生成的
            if tokio::fs::metadata(&image_path).await.is_ok() {
                info!("Scene {} image already exists, skipping...", scene.index);
                scene.image_path = Some(image_path);
                continue;
            }
            
            client.generate_image(&scene.description, &image_path).await?;
            scene.image_path = Some(image_path.clone());
            info!("Generated image for scene {} ({}/{})", scene.index, idx + 1, scene_count);
        }
    } else {
        info!("Step 2/4: Skipped image generation");
    }

    // 4. 生成语音（支持断点续传）
    info!("Step 3/4: Generating speech...");
    let audio_path = format!("{}/audio.mp3", work_dir);
    
    if tokio::fs::metadata(&audio_path).await.is_ok() {
        info!("Audio file already exists, skipping speech generation...");
    } else {
        let full_text: String = scenes
            .iter()
            .map(|s| s.subtitle.clone())
            .collect::<Vec<_>>()
            .join("。");
        client.generate_speech(&full_text, &audio_path).await?;
    }

    // 5. 合成视频
    info!("Step 4/4: Generating final video...");
    let video_gen = VideoGenerator::new(work_dir.clone());
    video_gen
        .generate_video(&scenes, &audio_path, &output_path)
        .await?;

    Ok(())
}
