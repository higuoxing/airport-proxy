use std::include_str;
use std::str::FromStr;

use worker::*;

mod utils;

const SUB_URLS: &str = include_str!("subscription.def");

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

fn decode_url(url: String) -> Result<Vec<String>> {
    let decoded_bytes = base64::decode(url).unwrap();
    let decoded_str = String::from_utf8(decoded_bytes).unwrap();
    Ok(decoded_str.split('\n').map(|s| s.to_string()).collect())
}

fn encode_url(urls: Vec<String>) -> String {
    base64::encode(urls.join("\n").as_bytes())
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();

    let router = Router::new();
    router
        .get_async("/sub/:type", |_, ctx| async move {
            if let Some(sub_type) = ctx.param("type") {
                if sub_type == "sip002" {
                    let mut urls = vec![];
                    for u in SUB_URLS.split('\n') {
                        match Url::from_str(u) {
                            Ok(url) => {
                                let fetch = Fetch::Url(url);
                                match fetch.send().await {
                                    Ok(mut resp) => {
                                        urls.append(
                                            &mut decode_url(resp.text().await.unwrap()).unwrap(),
                                        );
                                    }
                                    _ => continue,
                                }
                            }
                            _ => continue,
                        }
                    }

                    return Response::ok(encode_url(urls));
                }
            }
            return Response::ok("Hello from Workers!");
        })
        .run(req, env)
        .await
}
