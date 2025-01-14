use impit::emulation::Browser;
use impit::impit::Impit;

#[tokio::main]
async fn main() {
    let mut impit = Impit::builder()
        .with_browser(Browser::Firefox)
        .with_http3()
        .build();

    let response = impit.get(String::from("https://example.com"), None).await;

    match response {
        Ok(response) => {
            println!("{}", response.text().await.unwrap());
        }
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}
