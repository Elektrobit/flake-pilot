// This file is part of flake-pilot
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
use std::io::{Error, ErrorKind};
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;

pub async fn fetch_file(
    response: reqwest::Response, filepath: &String
) -> Result<(), Box<dyn std::error::Error>> {
    /*!
    Download file from response
    !*/
    let url = &format!("{}", response.url());
    let total_size = response
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", url))?;
    let progress = ProgressBar::new(total_size);

    progress.set_style(ProgressStyle::default_bar()
        .template(
            &format!(
                "{}\n{} [{}] [{}] {}/{} ({}, {})",
                "{msg}",
                "{spinner:.green}",
                "{elapsed_precise}",
                "{wide_bar:.cyan/blue}",
                "{bytes}",
                "{total_bytes}",
                "{bytes_per_sec}",
                "{eta}"
            )
        )
        .progress_chars("#>-"));
    progress.set_message(&format!("Downloading {}", filepath));

    let mut file = File::create(filepath)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        progress.set_position(new);
    }
    progress.finish_with_message(
        &format!("Downloaded {}", filepath)
    );
    Ok(())
}

pub async fn send_request(
    url: &String
) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    /*!
    Send GET request to the specified url and return response object
    !*/
    let client = reqwest::Client::builder()
        .build()?;

    let response = client
        .get(url)
        .send()
        .await?;

    match response.status() {
        reqwest::StatusCode::BAD_REQUEST => {
            let request_error = format!(
                "content-length:{:?} server:{:?}",
                response.headers().get(reqwest::header::CONTENT_LENGTH),
                response.headers().get(reqwest::header::SERVER),
            );
            return Err(
                Box::new(Error::new(ErrorKind::InvalidData, request_error))
            )
        },
        status => {
            let request_status = format!("{}", status);
            if request_status != "200 OK" {
                return Err(
                    Box::new(Error::new(ErrorKind::Other, request_status))
                )
            }
        },
    }
    Ok(response)
}
