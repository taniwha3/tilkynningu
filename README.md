# Getting Started

## Usage 

### Dependencies:
* rust - https://www.rust-lang.org/
* wrangler - https://github.com/cloudflare/wrangler
* miniflare - https://miniflare.dev/

The majority of the source code is in `./src/lib.rs`.

Please populate `.env` before running the build.

Example `.env`:

```
TWITCH_TOKEN=twitch_secret
DISCORD_WEBHOOK_URL=https://example.com/callback
```

You should point DISCORD_WEBHOOK_URL towards a discord callback in a server you own or moderate.

```bash
# compiles your project to WebAssembly and run the test
make build

# run your Worker in an ideal development workflow (with a local server, file watcher & more)
# the resulting environment will monitor the filesystem for changes.
make run

# trigger an event for your local Worker to process
make trigger

# deploy your Worker globally to the Cloudflare network (update your wrangler.toml file for configuration)
make deploy

# run your Worker in a docker environment (for environments where you cannot or don't want to install rust/wrangler/miniflare)
# the resulting environment will monitor the filesystem for changes.
make docker-watch
```

The final location of your callback will require an event subscription to be created.

Store this in a file:
```json
{
    "type": "stream.online",
    "version": "1",
    "condition": {
        "broadcaster_user_id": "279419549"
    },
    "transport": {
        "method": "webhook",
        "callback": "https://example.com/callback",
        "secret": "secret"
    }
}
```

The following post will submit the subscription event to twitch:
```
twitch api post eventsub/subscriptions -b @subscription.json
```