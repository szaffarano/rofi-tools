use anyhow::Context;
use directories_next::{self, BaseDirs};
use std::{
    fs::{self, DirEntry},
    io::Error,
    path::PathBuf,
};

/// Abstraction to store files in a $XDG_CACHE_DIR folder.
pub struct SimpleCache {
    cache_dir: PathBuf,
}

pub trait CacheEntry {
    fn id(&self) -> String;
}

impl SimpleCache {
    /// Create a new SimpleCache instance on $XDG_CACHE_DIR/hierarchy.
    /// Where `whierarchy` is a valid unix-like path.
    pub fn new(hierarchy: &str) -> anyhow::Result<Self> {
        Ok(Self {
            cache_dir: Self::init_cache_dir(hierarchy)?,
        })
    }

    /// Prune the cache directory, removing all files that are not in the `excludes` list.
    pub fn prune(&self, excludes: Vec<String>) -> anyhow::Result<usize> {
        let to_delete = fs::read_dir(&self.cache_dir)
            .context(format!("Error reading cache folder: {:?}", self.cache_dir))?
            .filter(|p| {
                if let Ok(entry) = p {
                    let path = entry.path();
                    let id = path.file_name().unwrap();
                    !excludes.iter().any(|e| *e.as_str() == *id)
                } else {
                    false
                }
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<Vec<DirEntry>, Error>>()
            .context("Error collecting entries to delete")?;

        let deleted = to_delete
            .iter()
            .inspect(|entry| {
                fs::remove_file(entry.path()).unwrap();
            })
            .count();

        Ok(deleted)
    }

    /// Add a file to the cache directory.
    pub fn add(&self, entry: &dyn CacheEntry, value: Vec<u8>) {
        let path = self.path(entry.id().as_str());
        fs::write(&path, value).unwrap();
    }

    /// Check if a file exists in the cache directory.
    pub fn exists(&self, id: &str) -> bool {
        self.cache_dir.join(id).is_file()
    }

    /// Get the path of a file in the cache directory.
    pub fn path(&self, id: &str) -> PathBuf {
        self.cache_dir.join(id)
    }

    /// Initialize the cache directory, creating the folders if they don't exist.
    fn init_cache_dir(hierarchy: &str) -> anyhow::Result<PathBuf> {
        let dirs =
            BaseDirs::new().ok_or_else(|| anyhow::anyhow!("Error getting base directories"))?;

        let cache_dir = &hierarchy
            .split('/')
            .fold(dirs.cache_dir().to_path_buf(), |acc, dir| acc.join(dir));

        if let Ok(false) = fs::exists(cache_dir) {
            fs::create_dir_all(cache_dir)
                .context(format!("Error creating cache folder: {:?}", cache_dir))?;
        }

        Ok(cache_dir.to_path_buf())
    }
}
