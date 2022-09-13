use std::io::Read;

use reqwest::header::{self, HeaderMap};

async fn post() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let image = std::fs::File::open("./asserts/image.jpg").unwrap();
    let mut buffer = [0 as u8; 4096];
    let mut r = std::io::BufReader::new(image);
    r.read(&mut buffer);
    let buffer: Vec<u8> = buffer.into_iter().collect();

    let mut headers = HeaderMap::new();
    headers.insert("content-type", "text/html".parse().unwrap());
    headers.insert("content-length", "32".parse().unwrap());

    let _resp = client
        .post("http://127.0.0.1:8000")
        .headers(headers)
        .json(&buffer)
        .send()
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    post().await;
    // let mut l = Vec::new();
    // for _i in 0..4 {
    //     // println!("spawn thread {} start", i);
    //     let thread_handle = tokio::spawn(async move {
    //         for _k in 0..100 {
    //             // println!("thread {} spawn task {}", i, k);
    //             let _task = post().await;
    //         }
    //     });
    //     l.push(thread_handle);
    // }
    // for h_l in l {
    //     let _res = h_l.await;
    // }
}
