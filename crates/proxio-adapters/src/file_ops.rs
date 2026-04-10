use std::fs;
use std::path::Path;

pub fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "missing parent directory".to_string())?;
    fs::create_dir_all(parent).map_err(|err| err.to_string())?;

    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content).map_err(|err| err.to_string())?;
    fs::rename(&temp_path, path).map_err(|err| err.to_string())?;

    Ok(())
}
