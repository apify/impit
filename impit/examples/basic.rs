use impit::cookie::Jar;
use impit::{fingerprint::database as fingerprints, impit::Impit};

#[tokio::main]
async fn main() {
    let impit = Impit::<Jar>::builder()
        .with_fingerprint(fingerprints::firefox_144::fingerprint())
        .with_http3()
        .build()
        .unwrap();

    let response = impit
        .get(String::from("https://example.com"), None, None)
        .await;

    match response {
        Ok(response) => {
            println!("{}", response.text().await.unwrap());
        }
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}
