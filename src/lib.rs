use serde::{Deserialize, Serialize};
use worker::*;

use crate::wasm_bindgen::JsValue;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

const TWITCH_MESSAGE_ID: &str = "twitch-eventsub-message-id";
const TWITCH_MESSAGE_TIMESTAMP: &str = "twitch-eventsub-message-timestamp";
const TWITCH_MESSAGE_SIGNATURE: &str = "twitch-eventsub-message-signature";
const TWITCH_MESSAGE_TYPE: &str = "twitch-eventsub-message-type";
const HMAC_PREFIX: &str = "sha256=";

#[derive(Serialize, Deserialize)]
struct TwitchChallenge {
    challenge: String,
}

#[derive(Serialize, Deserialize)]
struct TwitchOnlineEvent {
    event: TwitchEvent,
}

#[derive(Serialize, Deserialize)]
struct TwitchEvent {
    broadcaster_user_login: String,
}

#[derive(Serialize, Deserialize)]
struct DiscordMessage {
    content: String,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get("/", |_, _| Response::ok("OK"))
        .post_async("/callback", |mut req, ctx| async move {
            let body = req.text().await?;

            if !req.headers().has(TWITCH_MESSAGE_ID)? {
                return Response::error("Missing TWITCH_MESSAGE_ID", 403);
            }
            if !req.headers().has(TWITCH_MESSAGE_TIMESTAMP)? {
                return Response::error("Missing TWITCH_MESSAGE_TIMESTAMP", 403);
            }
            if !req.headers().has(TWITCH_MESSAGE_SIGNATURE)? {
                return Response::error("Missing TWITCH_MESSAGE_SIGNATURE", 403);
            }
            if !req.headers().has(TWITCH_MESSAGE_TYPE)? {
                return Response::error("Missing TWITCH_MESSAGE_TYPE", 403);
            }

            let twitch_message_id = match req.headers().get(TWITCH_MESSAGE_ID) {
                Ok(opt) => match opt {
                    Some(id) => id,
                    None => return Response::error("Missing TWITCH_MESSAGE_ID", 403),
                },
                Err(_e) => {
                    return Response::error("Unable to check TWITCH_MESSAGE_ID header", 403);
                }
            };

            let twitch_message_timestamp = match req.headers().get(TWITCH_MESSAGE_TIMESTAMP) {
                Ok(opt) => match opt {
                    Some(id) => id,
                    None => return Response::error("Missing TWITCH_MESSAGE_TIMESTAMP", 403),
                },
                Err(_e) => {
                    return Response::error("Unable to check TWITCH_MESSAGE_TIMESTAMP header", 403);
                }
            };

            let twitch_message_signature = match req.headers().get(TWITCH_MESSAGE_SIGNATURE) {
                Ok(opt) => match opt {
                    Some(id) => id,
                    None => return Response::error("Missing TWITCH_MESSAGE_SIGNATURE", 403),
                },
                Err(_e) => {
                    return Response::error("Unable to check TWITCH_MESSAGE_SIGNATURE header", 403);
                }
            };

            let twitch_message_type = match req.headers().get(TWITCH_MESSAGE_TYPE) {
                Ok(opt) => match opt {
                    Some(id) => id,
                    None => return Response::error("Missing TWITCH_MESSAGE_TYPE", 403),
                },
                Err(_e) => {
                    return Response::error("Unable to check TWITCH_MESSAGE_TYPE header", 403);
                }
            };

            let twitch_token = ctx.var("TWITCH_TOKEN")?.to_string();
            let message = get_hmac_message(&twitch_message_id, &twitch_message_timestamp, &body);

            let hmac = HMAC_PREFIX.to_owned() + &get_hmac(&twitch_token, &message);

            if hmac == twitch_message_signature {
            } else {
                return Response::error(
                    "Not OK!",
                    403,
                );
            };

            // if message type is 'webhook_callback_verification' then just return OK
            if twitch_message_type == "webhook_callback_verification" {
                let challenge: TwitchChallenge = serde_json::from_str(body.as_str())?;
                return Response::ok(challenge.challenge);
            }

            // parse data
            let event: TwitchOnlineEvent = serde_json::from_str(body.as_str())?;
            // extract name
            let streamer_name = event.event.broadcaster_user_login;

            // send to discord
            let discord_message = DiscordMessage {
                content: format!(
                    "@here: 🚨 {} is streaming! 🚨\nhttps://twitch.tv/{}",
                    streamer_name, streamer_name
                ),
            };
            // let mut tmp = "";
            // let mut discord_data = tmp.to_owned() + "{\"content\": \"" + streamer_name.as_str() +" is now streaming!\"}";

            let discord_data = serde_json::to_string(&discord_message)?;
            let x = JsValue::from(discord_data);

            let webhook_url_string = ctx.var("DISCORD_WEBHOOK_URL")?.to_string();
            // let webhook_url = (worker::Url::parse(webhook_url_string.as_str()))?;
            let mut request = Request::new_with_init(
                webhook_url_string.as_str(),
                RequestInit::new()
                    .with_body(Some(x))
                    .with_method(Method::Post),
            )?;
            let request_headers = request.headers_mut()?;
            match request_headers.set("Content-Type", "application/json") {
                Ok(_) => {}
                Err(e) => {
                    console_log!("Unable to set Content-Type header: {}", e)
                }
            };

            let _response = Fetch::Request(request).send().await?;
            // TODO get status code and check for failure

            Response::ok("Ok!")
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}

fn get_hmac(secret: &str, message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("");
    mac.update(message.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    hex::encode(code_bytes)
}

fn get_hmac_message(
    twitch_message_id: &String,
    twitch_message_timestamp: &String,
    body: &String,
) -> String {
    let result = "".to_string();
    result + twitch_message_id + twitch_message_timestamp + body
}
