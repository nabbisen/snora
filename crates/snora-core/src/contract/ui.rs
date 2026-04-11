/// アイコンの抽象表現
#[derive(Debug, Clone)]
pub enum Icon {
    Text(String),
    #[cfg(feature = "lucide-icons")]
    Lucide(lucide_icons::Icon),
    #[cfg(feature = "svg-icons")]
    Svg(std::path::PathBuf),
}

// 文字列からIcon::Textへの変換を容易にする
impl From<&str> for Icon {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}
