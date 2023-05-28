// Dependencies
use std::{
    path::PathBuf,
    io::{
        Cursor,
        Write
    },
    fs::{
        self,
        File
    },
    cmp::min
};
use flate2::bufread::GzDecoder;
use indicatif::{
    ProgressBar,
    ProgressStyle
};
use reqwest::{
    header,
    Client
};
use futures_util::StreamExt;
use clap::ValueEnum;
use async_recursion::async_recursion;

use crate::types::GitHubReleases;

// Server stuff
trait ServerTrait {
    fn name(&self) -> &'static str;
    fn download_link(&self) -> String;
    fn download_link_config(&self) -> String;
}
#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum Servers {
    T6,
    T5,
    T4,
    IW5
}
impl ServerTrait for Servers {
    fn name(&self) -> &'static str {
        match self {
            Servers::T6 => "T6",
            Servers::T5 => "T5",
            Servers::T4 => "T4",
            Servers::IW5 => "IW5"
        }
    }

    fn download_link(&self) -> String {
        let id = match self {
            Servers::T6 => "1RCqhm_1oMEDSk-VoeQy_tWTE-9jZ6Exd",
            Servers::T5 => "1bDArK1W2kVse753C0Ht_n0hRYiaQ8ZfE",
            Servers::T4 => "1AqTkGMXj2B2UTnm6hg_WFfQLVxXJDn3K",
            _ => panic!("download link not implemented for this game type")
        };
        format!("https://drive.google.com/uc?export=download&id={}&confirm=t", id)
    }

    fn download_link_config(&self) -> String {
        format!("https://api.github.com/repos/xerxes-at/{}ServerConfigs/zipball/master", self.name())
    }
}

// Downloads a file
#[async_recursion]
pub async fn download_file(client: &Client, url: &str) -> Result<Vec<u8>, String> {
    // Initialise the request
    let response = client
        .get(url)
        .send()
        .await
        .expect(&format!("Failed to GET from '{}'", &url));
    let total_size_r = response
        .content_length();
    
    // Check we got the size
    if total_size_r.is_none() {
        println!("failed to get content-length of {}, retrying...", url);
        return download_file(client, url).await;
    }
    
    // Indicatif setup
    let total_size = total_size_r.unwrap();
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .expect("unable to initialise pb template")
            .progress_chars("#>-")
    );
    pb.set_message(format!("Downloading {}", url));

    // Listen to when we download more bytes
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(item) = stream.next().await {
        // Grab the latest chunk
        let chunk = item.expect("Error while downloading file");
        let new = min(downloaded + (chunk.len() as u64), total_size);

        // Update the progress bar
        downloaded = new;
        pb.set_position(new);

        // Append the chunk to the bytes vector
        bytes.extend_from_slice(&chunk);
    }

    // Done
    pb.finish_with_message(format!("Downloaded {}", url));
    Ok(bytes)
}

// Downloads a file, output to a file
pub async fn download_file_out(client: &Client, url: &str, output: &PathBuf) -> Result<File, String> {
    // Initialise the request
    let response = client
        .get(url)
        .send()
        .await
        .expect(&format!("Failed to GET from '{}'", &url));
    let total_size = response
        .content_length()
        .expect(&format!("Failed to get content length from '{}'", &url));
    
    // Indicatif setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .expect("unable to initialise pb template")
            .progress_chars("#>-")
    );
    pb.set_message(format!("Downloading {}", url));

    // Create the file
    let mut file = File::create(output).expect(&format!("Failed to create file {}", output.to_str().unwrap()));

    // Listen to when we download more bytes
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        // Grab the latest chunk
        let chunk = item.expect("Error while downloading file");
        let new = min(downloaded + (chunk.len() as u64), total_size);

        // Update the progress bar
        downloaded = new;
        pb.set_position(new);

        // Append the chunk to the file
        file.write_all(&chunk)
            .expect("failed to write chunk to file");
    }

    // Done
    pb.finish_with_message(format!("Downloaded {}", url));
    Ok(file)
}

// Installs a game's server files
pub async fn install_server(server: &Servers, target_dir: Option<&str>) {
    // Initialise
    let target_dir = PathBuf::from(target_dir.unwrap_or("."));
    fs::create_dir_all(&target_dir).expect("unable to create installation directory");

    // Create a reqwest client
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("plutonium-manager"));
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("compress, deflate, gzip"));
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // Download the release (output to a file due to large size > 1 GB)
    let release_zip_path = PathBuf::from("./server_files.zip");
    download_file_out(&client, &server.download_link(), &release_zip_path).await.unwrap();
    let release_zip = File::open(&release_zip_path).expect("uunable to open server file zip");

    // Extract
    zip_extract::extract(release_zip, &target_dir, true).expect("unable to extract server files - server files don't exist?");

    // Delete the file
    fs::remove_file(release_zip_path)
        .expect("unable to remove server zip");
}

// Installs IW4M
pub async fn install_iw4m(target_dir: Option<&str>) {
    // Initialise
    let target_dir = PathBuf::from(target_dir.unwrap_or("."));
    fs::create_dir_all(&target_dir).expect("unable to create installation directory");

    // Create a reqwest client
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("plutonium-manager"));
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("compress, deflate, gzip"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // Query github for the download url
    let github_resp = client.get("https://api.github.com/repos/RaidMax/IW4M-Admin/releases/latest")
        .send()
        .await
        .expect("unable to request github for iw4m download link")
        .json::<GitHubReleases>()
        .await
        .expect("unable to parse json response from github");

    // Download the release
    let release_zip = download_file(&client, &github_resp.assets[0].browser_download_url).await.unwrap();

    // Extract
    zip_extract::extract(Cursor::new(release_zip), &target_dir, true).expect("unable to extract iw4m files");
}

// Installs IW4M
pub async fn install_iw4m_config(target_dir: Option<&str>) {
    // Initialise
    let target_dir = PathBuf::from(target_dir.unwrap_or("."));
    fs::create_dir_all(&target_dir).expect("unable to create installation directory");

    // Create a reqwest client
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("plutonium-manager"));
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("compress, deflate, gzip"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // Download the release
    let release_zip = download_file(&client, "https://cdn.discordapp.com/attachments/749611171216359474/1108504949836496996/Configuration.zip").await.unwrap();

    // Extract
    zip_extract::extract(Cursor::new(release_zip), &target_dir, true).expect("unable to extract iw4m files");
}

// Installs IW4M log server
pub async fn install_iw4m_log(target_dir: Option<&str>) {
    // Initialise
    let target_dir = PathBuf::from(target_dir.unwrap_or("."));
    fs::create_dir_all(&target_dir).expect("unable to create installation directory");

    // Create a reqwest client
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("plutonium-manager"));
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("compress, deflate, gzip"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // Support for windows and linux
    if cfg!(unix) {
        // Download the release
        let release_zip = download_file(&client, "https://github.com/Stefanuk12/iw4m-log-server/releases/latest/download/iw4m-log-server-x86_64-unknown-linux-gnu.tar.gz").await.unwrap();

        // Extract
        let decompressed = GzDecoder::new(release_zip.as_slice());
        let mut archive = tar::Archive::new(decompressed);
        archive.unpack(target_dir).expect("unable to extract server files - server files don't exist?");
    } else if cfg!(windows) {
        // Download the release
        let release_zip = download_file(&client, "https://github.com/Stefanuk12/iw4m-log-server/releases/latest/download/iw4m-log-server-x86_64-pc-windows-msvc.zip").await.unwrap();

        // Extract
        zip_extract::extract(Cursor::new(release_zip), &target_dir, true).expect("unable to extract iw4m files");
    }
}

// Download server config
pub async fn install_config(server: &Servers, target_dir: Option<&str>) {
    // Initialise
    let target_dir = PathBuf::from(target_dir.unwrap_or("."));
    fs::create_dir_all(&target_dir).expect("unable to create installation directory");

    // Create a reqwest client
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("plutonium-manager"));
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("compress, deflate, gzip"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // Download the release
    let release_zip = download_file(&client, &server.download_link_config()).await.unwrap();

    // Extract
    zip_extract::extract(Cursor::new(release_zip), &target_dir, true).expect("unable to extract server files - server config don't exist?");
}

// Download plutonium
pub async fn install_plutonium(target_dir: Option<&str>) {
    // Initialise
    let target_dir = PathBuf::from(target_dir.unwrap_or("."));

    // Create a reqwest client
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("plutonium-manager"));
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("compress, deflate, gzip"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // Download the release
    let plutonium = download_file(&client, "https://cdn.plutonium.pw/updater/plutonium.exe").await.unwrap();

    // Extract
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(target_dir).expect("unable to open file for plutonium");

    file.write_all(&plutonium).expect("unable to write plutonium data");
}

// Download rcon server
pub async fn install_rcon(target_dir: Option<&str>) {
    // Initialise
    let target_dir = PathBuf::from(target_dir.unwrap_or("."));
    fs::create_dir_all(&target_dir).expect("unable to create installation directory");

    // Create a reqwest client
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("plutonium-manager"));
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("compress, deflate, gzip"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    // Support for windows and linux
    if cfg!(unix) {
        // Download the release
        let release_zip = download_file(&client, "https://github.com/Stefanuk12/cod-rcon/releases/latest/download/cod-rcon-x86_64-unknown-linux-gnu.tar.gz").await.unwrap();

        // Extract
        let decompressed = GzDecoder::new(release_zip.as_slice());
        let mut archive = tar::Archive::new(decompressed);
        archive.unpack(target_dir).expect("unable to extract server files - server files don't exist?");
    } else if cfg!(windows) {
        // Download the release
        let release_zip = download_file(&client, "https://github.com/Stefanuk12/cod-rcon/releases/latest/download/cod-rcon-x86_64-pc-windows-msvc.zip").await.unwrap();

        // Extract
        zip_extract::extract(Cursor::new(release_zip), &target_dir, true).expect("unable to extract iw4m files");
    }
}