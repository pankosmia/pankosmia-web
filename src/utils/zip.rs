use std::fs::File;
use tempfile::NamedTempFile;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
use std::io::{Read, Write};
use crate::utils::paths::os_slash_str;

/// Make a zip archive as a temporary file, from ingredients under the specified path, and return the temporary file path.
pub fn make_zip_file(path_to_zip: &String) -> NamedTempFile {
    // Iterate over ingredients, writing zip to temp file on the way
    let ingredient_walkdir = WalkDir::new(&path_to_zip);
    let prefix = std::path::Path::new(&path_to_zip);
    let ingredient_iterator = ingredient_walkdir.into_iter();
    let temp_zip_path = NamedTempFile::new().expect("tempfile");
    let mut zip = ZipWriter::new(&temp_zip_path);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    let mut buffer = Vec::new();
    for entry_result in ingredient_iterator {
        let entry = entry_result.expect("entry");
        let path = entry.path();
        let name = path.strip_prefix(prefix).expect("strip prefix");
        let path_as_string = match name.to_str().map(str::to_owned) {
            Some(p) => p,
            None => continue,
        };
        if path.is_file() {
            // println!("file '{}'", path_as_string);
            zip.start_file(path_as_string, options).expect("start file");
            let mut f = File::open(path).expect("open file");
            f.read_to_end(&mut buffer).expect("read to end");
            zip.write_all(&buffer).expect("write");
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // println!("dir '{}'", path_as_string);
            zip.add_directory(path_as_string, options)
                .expect("add directory");
        }
    }
    zip.finish().expect("finish");
    temp_zip_path
}

/// Unpack the zip archive at the specified path to the specified destination. The destination directory's parent must exist.
pub async fn unpack_zip_file(archive_path: NamedTempFile, destination: String) -> Result<(), std::io::Error> {
    // Make zip struct
    let zip_file = File::open(archive_path).expect("open zip archive file");
    let mut archive = ZipArchive::new(zip_file)?;
    // Iterate over archive files, ignoring bad ones
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("file from zip");
        let out_path = match file.enclosed_name() {
            Some(p) => p,
            None => continue
        };
        if !file.is_file() {
            continue;
        }
        let full_out_path = format!(
            "{}{}{}",
            destination,
            os_slash_str(),
            out_path.display()
        );
        let out_path_parent = std::path::Path::new(&full_out_path).parent().expect("parent");
        if !out_path_parent.exists() {
            std::fs::create_dir_all(&out_path_parent).expect("create all dirs");
        }
        let mut out_file = std::fs::File::create(&full_out_path).expect("create");
        std::io::copy(&mut file, &mut out_file).expect("write");
    }
    Ok(())
}