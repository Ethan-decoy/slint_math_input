use crate::{Error, model::MAX_DRAFT_BYTES};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::Path,
};
pub(crate) fn save(path: &Path, source: &str) -> Result<(), Error> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| Error::new(format!("Draft directory: {e}")))?;
    let name = path
        .file_name()
        .ok_or_else(|| Error::new("Draft path must name a file"))?
        .to_string_lossy();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| Error::new(e.to_string()))?
        .as_nanos();
    let temporary = parent.join(format!(".{name}.{}.{nonce}.tmp", std::process::id()));
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(source.as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|e| Error::new(format!("Draft save: {e}")))
}
pub(crate) fn load(path: &Path) -> Result<String, Error> {
    let file = fs::File::open(path).map_err(|e| Error::new(format!("Draft open: {e}")))?;
    if file
        .metadata()
        .map_err(|e| Error::new(e.to_string()))?
        .len()
        > MAX_DRAFT_BYTES as u64
    {
        return Err(Error::new("Draft size limit exceeded"));
    }
    let mut source = String::new();
    file.take(MAX_DRAFT_BYTES as u64 + 1)
        .read_to_string(&mut source)
        .map_err(|e| Error::new(format!("Draft read: {e}")))?;
    if source.len() > MAX_DRAFT_BYTES {
        return Err(Error::new("Draft size limit exceeded"));
    }
    Ok(source)
}
