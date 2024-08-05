use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process,
};

use curl::easy::Easy;
use flate2::read::GzDecoder;
use tar::Archive;

const DXVK_EXISTS_FILE: &str = "dxvk_version";

// Version in format of "x.y"
// i.e "2.4"
pub fn install_dxvk(data_path: &Path, version: &String, prefix_path: &Path) {
    let file_path = Path::new(data_path).join("dxvk");
    if file_path.join(format!("dxvk-{version}")).exists() {
        if is_dxvk_installed(&file_path, version) {
            return;
        }
        copy_dxvk_files(&file_path.join(format!("dxvk-{version}")), prefix_path);
        set_dxvk_installed(&file_path, version);
        return;
    }
    let url = format!(
        "https://github.com/doitsujin/dxvk/releases/download/v{version}/dxvk-{version}.tar.gz"
    );
    eprintln!("url: {0}", url);
    let data = match get_data_from_url(format!(
        "https://github.com/doitsujin/dxvk/releases/download/v{version}/dxvk-{version}.tar.gz"
    )) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to download data: {}", e);
            process::exit(99);
        }
    };

    eprintln!("data.len: {0}", data.len());
    let gz_data = GzDecoder::new(&data[..]);
    let mut archive = Archive::new(gz_data);
    // Should never happen, as long as the data_dir was correctly created
    fs::create_dir_all(&file_path).unwrap();
    // TODO: Handle error
    match archive.unpack(&file_path) {
        Ok(_) => println!("Successfully unpacked archive"),
        Err(e) => {
            eprintln!("Failed to unpack archive: {}", e);
            process::exit(99);
        }
    }
    copy_dxvk_files(&file_path.join(format!("dxvk-{version}")), prefix_path);
    set_dxvk_installed(&file_path, version);
}

fn get_data_from_url(url: String) -> Result<Vec<u8>, curl::Error> {
    let mut dst = Vec::new();
    let mut easy = Easy::new();
    easy.url(&url)?;
    easy.follow_location(true);
    let mut transfer = easy.transfer();
    transfer
        .write_function(|data: &[u8]| {
            dst.extend_from_slice(data);
            Ok(data.len())
        })
        .unwrap();
    transfer.perform()?;
    drop(transfer);
    Ok(dst)
}

// TODO:
fn copy_dxvk_files(dxvk_folder: &Path, prefix_path: &Path) {
    let x32 = dxvk_folder.join("x32");
    let x64 = dxvk_folder.join("x64");

    copy_dir_all(x32, prefix_path.join("drive_c/windows/system32")).unwrap();
    copy_dir_all(x64, prefix_path.join("drive_c/windows/syswow64")).unwrap();
}

fn set_dxvk_installed(data_path: &Path, version: &String) {
    File::create(data_path.join(DXVK_EXISTS_FILE))
        .unwrap()
        .write_all(version.as_bytes())
        .unwrap();
}

fn is_dxvk_installed(file_path: &Path, version: &String) -> bool {
    let file_path = file_path.join(DXVK_EXISTS_FILE);
    if !file_path.exists() {
        return false;
    }
    let file_version = fs::read_to_string(file_path).unwrap();
    *version == file_version
}

// https://stackoverflow.com/a/65192210
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
