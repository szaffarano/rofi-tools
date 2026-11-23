use anyhow::Result;
use clipcat_base::ClipEntryMetadata;
use clipcat_client::Manager;
use roto_core::backend::{ClipEntry, ClipboardBackend};

/// Maximum gRPC message size (128 MiB) to handle large clipboard images
const MAX_GRPC_MESSAGE_SIZE: usize = 128 * 1024 * 1024;

/// Preview length for clipboard entries (shown in the picker)
const PREVIEW_LENGTH: usize = 80;

/// Convert clipcat metadata to a neutral ClipEntry.
fn metadata_to_entry(md: &ClipEntryMetadata) -> ClipEntry {
    if md.mime.type_() == mime::IMAGE {
        ClipEntry::Image {
            id: format!("{:016x}", md.id),
            content_type: "png".to_string(),
        }
    } else {
        ClipEntry::Text {
            id: format!("{:016x}", md.id),
            title: md.preview.clone(),
        }
    }
}

/// Parse a hex string ID back to a u64.
///
/// # Errors
///
/// Returns an error if the string is not valid hex or overflows u64.
fn parse_id(id: &str) -> Result<u64> {
    u64::from_str_radix(id, 16).map_err(|e| anyhow::anyhow!("Failed to parse ID '{}': {}", id, e))
}

pub struct ClipcatBackend {
    rt: tokio::runtime::Runtime,
    client: clipcat_client::Client,
}

impl ClipcatBackend {
    /// Create a new clipcat backend connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection to the clipcat daemon fails.
    pub fn new(endpoint: http::Uri, token: Option<String>) -> Result<Self> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let client = rt.block_on(async {
            clipcat_client::Client::builder()
                .grpc_endpoint(endpoint)
                .access_token(token)
                .max_decoding_message_size(MAX_GRPC_MESSAGE_SIZE)
                .build()
                .await
        })?;

        Ok(Self { rt, client })
    }
}

impl ClipboardBackend for ClipcatBackend {
    fn list(&self) -> Result<Vec<ClipEntry>> {
        let metadata_list = self
            .rt
            .block_on(self.client.list(PREVIEW_LENGTH))
            .map_err(|e| anyhow::anyhow!("Failed to list clipboard entries: {:?}", e))?;
        Ok(metadata_list.iter().map(metadata_to_entry).collect())
    }

    fn raw_bytes(&self, id: &str) -> Result<Vec<u8>> {
        let id = parse_id(id)?;
        let entry = self
            .rt
            .block_on(self.client.get(id))
            .map_err(|e| anyhow::anyhow!("Failed to get clipboard entry {}: {:?}", id, e))?;
        entry
            .encoded()
            .map_err(|e| anyhow::anyhow!("Failed to encode clipboard entry: {:?}", e))
    }

    fn remove(&self, id: &str) -> Result<()> {
        let id = parse_id(id)?;
        self.rt
            .block_on(self.client.batch_remove(&[id]))
            .map_err(|e| anyhow::anyhow!("Failed to remove clipboard entry {}: {:?}", id, e))?;
        Ok(())
    }

    fn activate(&self, id: &str) -> Result<()> {
        let id = parse_id(id)?;
        self.rt
            .block_on(self.client.mark(id, clipcat_base::ClipboardKind::Clipboard))
            .map_err(|e| anyhow::anyhow!("Failed to activate clipboard entry {}: {:?}", id, e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ClipcatBackend;
    use clipcat_base::ClipEntryMetadata;
    use roto_core::backend::ClipEntry;
    use time::OffsetDateTime;

    #[test]
    fn connect_error_is_err() {
        // Use a bogus unix socket path that doesn't exist
        let bogus_path = "/tmp/nonexistent-clipcat-socket-12345.sock";
        let endpoint: http::Uri = bogus_path.parse().expect("valid URI");

        let result = ClipcatBackend::new(endpoint, None);

        // Should return Err, not panic
        assert!(
            result.is_err(),
            "Expected connection error for bogus endpoint"
        );
    }

    #[test]
    fn metadata_to_text_entry() {
        let metadata = ClipEntryMetadata {
            id: 255,
            kind: clipcat_base::ClipboardKind::Clipboard,
            timestamp: OffsetDateTime::now_utc(),
            mime: mime::TEXT_PLAIN,
            preview: "Sample text".to_string(),
        };

        let entry = super::metadata_to_entry(&metadata);

        match entry {
            ClipEntry::Text { id, title } => {
                assert_eq!(id, "00000000000000ff");
                assert_eq!(title, "Sample text");
            }
            _ => panic!("Expected ClipEntry::Text"),
        }
    }

    #[test]
    fn metadata_to_image_entry() {
        let metadata = ClipEntryMetadata {
            id: 42,
            kind: clipcat_base::ClipboardKind::Clipboard,
            timestamp: OffsetDateTime::now_utc(),
            mime: mime::IMAGE_PNG,
            preview: String::new(),
        };

        let entry = super::metadata_to_entry(&metadata);

        match entry {
            ClipEntry::Image { id, content_type } => {
                assert_eq!(id, "000000000000002a");
                assert_eq!(content_type, "png");
            }
            _ => panic!("Expected ClipEntry::Image"),
        }
    }

    #[test]
    fn parse_id_roundtrip_and_rejects_non_hex() {
        // Valid hex roundtrip
        let result = super::parse_id("00000000000000ff");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 255);

        // Another valid case
        let result = super::parse_id("000000000000002a");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Non-hex string should error
        let result = super::parse_id("zzzz");
        assert!(result.is_err());

        // Over-long string (> 16 hex digits) should error
        let result = super::parse_id("10000000000000000");
        assert!(result.is_err());
    }
}
