use serde::{Deserialize, Serialize};

/// 表示一个场景/分镜
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// 场景序号
    pub index: usize,
    /// 场景描述文本
    pub description: String,
    /// 对应的台词/字幕
    pub subtitle: String,
    /// 生成的图片路径
    pub image_path: Option<String>,
    /// 该场景的时长（秒）
    pub duration: f64,
}

impl Scene {
    pub fn new(index: usize, description: String, subtitle: String, duration: f64) -> Self {
        Self {
            index,
            description,
            subtitle,
            image_path: None,
            duration,
        }
    }
}
