use crate::cache::CacheEntry;
use crate::rofi::RofiEntry;

/// Neutral clipboard entry representation.
pub enum ClipEntry {
    Text { id: String, title: String },
    Image { id: String, content_type: String },
}

impl ClipEntry {
    pub fn id(&self) -> &str {
        match self {
            ClipEntry::Text { id, .. } => id,
            ClipEntry::Image { id, .. } => id,
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(self, ClipEntry::Image { .. })
    }
}

/// Generic clipboard backend trait.
pub trait ClipboardBackend {
    fn list(&self) -> anyhow::Result<Vec<ClipEntry>>;
    fn raw_bytes(&self, id: &str) -> anyhow::Result<Vec<u8>>;
    fn remove(&self, id: &str) -> anyhow::Result<()>;
    fn activate(&self, id: &str) -> anyhow::Result<()>;
}

impl CacheEntry for ClipEntry {
    fn id(&self) -> String {
        match self {
            ClipEntry::Text { id, .. } => id.to_string(),
            ClipEntry::Image { id, content_type } => format!("{id}.{content_type}"),
        }
    }
}

impl RofiEntry for ClipEntry {
    fn id(&self) -> String {
        match self {
            ClipEntry::Text { id, .. } => id.into(),
            ClipEntry::Image { id, .. } => id.into(),
        }
    }

    fn icon(&self) -> Option<String> {
        match self {
            ClipEntry::Text { .. } => None,
            ClipEntry::Image { id, content_type } => Some(format!("{id}.{content_type}")),
        }
    }

    fn label(&self) -> String {
        match self {
            ClipEntry::Text { title, .. } => title.into(),
            ClipEntry::Image { id, content_type } => {
                format!("{id}.{content_type}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipentry_id_and_image() {
        let text = ClipEntry::Text {
            id: "123".to_string(),
            title: "Sample text".to_string(),
        };
        assert_eq!(text.id(), "123");
        assert!(!text.is_image());

        let image = ClipEntry::Image {
            id: "456".to_string(),
            content_type: "png".to_string(),
        };
        assert_eq!(image.id(), "456");
        assert!(image.is_image());

        // CacheEntry::id for image should be "{id}.{content_type}"
        assert_eq!(CacheEntry::id(&image), "456.png");
        assert_eq!(CacheEntry::id(&text), "123");
    }
}
