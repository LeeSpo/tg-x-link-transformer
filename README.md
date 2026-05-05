# Telegram X/Twitter Link Transformer Bot

Telegram bot that watches messages for X/Twitter status links and posts preview-friendly replacements.

## Behavior

- `https://x.com/<user>/status/<id>` -> `https://fixupx.com/<user>/status/<id>`
- `https://twitter.com/<user>/status/<id>` -> `https://fxtwitter.com/<user>/status/<id>`
- Keeps query strings and fragments, such as `?s=20`.
- Converts all matching links in one message.
- Ignores non-status links and already converted `fixupx.com` / `fxtwitter.com` links.

## Telegram Setup

1. Open Telegram and talk to `@BotFather`.
2. Run `/newbot`, then copy the bot token.
3. If the bot needs to read all group messages, use BotFather `/setprivacy` and disable privacy mode for this bot.
4. Add the bot to your group.
5. For `LINK_ACTION=delete_resend`, promote the bot to group admin and grant delete-message permission.

## Docker Run Deployment

Use a local build:

```bash
docker build -t tg-x-link-transformer .
docker run -d \
  --name tg-x-link-transformer \
  --restart unless-stopped \
  -e TELEGRAM_BOT_TOKEN="123456789:replace-with-your-bot-token" \
  -e LINK_ACTION="reply" \
  tg-x-link-transformer
```

Or use the image published by GitHub Packages:

```bash
docker run -d \
  --name tg-x-link-transformer \
  --restart unless-stopped \
  -e TELEGRAM_BOT_TOKEN="123456789:replace-with-your-bot-token" \
  -e LINK_ACTION="reply" \
  ghcr.io/<owner>/<repo>:latest
```

Check logs:

```bash
docker logs -f tg-x-link-transformer
```

`LINK_ACTION` values:

- `reply`: reply to the original message with converted links.
- `delete_resend`: try to delete the original message, then send converted links. If deletion fails, the bot falls back to `reply`.

To update after changing code:

```bash
docker build -t tg-x-link-transformer .
docker rm -f tg-x-link-transformer
docker run -d \
  --name tg-x-link-transformer \
  --restart unless-stopped \
  -e TELEGRAM_BOT_TOKEN="123456789:replace-with-your-bot-token" \
  -e LINK_ACTION="reply" \
  tg-x-link-transformer
```

`RUST_LOG=info` is the image default. Add `-e RUST_LOG=debug` only when you need more verbose logs.

## Docker Compose Deployment

Compose is optional. Use it only if you prefer an `.env` file:

```bash
cp .env.example .env
nano .env
docker compose up -d --build
docker compose logs -f
```

## Native Linux Service

Install Rust on Ubuntu, then build the release binary:

```bash
cargo build --release
sudo install -m 0755 target/release/tg-x-link-transformer /usr/local/bin/tg-x-link-transformer
```

Create `/etc/tg-x-link-transformer.env`:

```env
TELEGRAM_BOT_TOKEN=123456789:replace-with-your-bot-token
LINK_ACTION=reply
RUST_LOG=info
```

Create `/etc/systemd/system/tg-x-link-transformer.service`:

```ini
[Unit]
Description=Telegram X/Twitter Link Transformer Bot
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=/etc/tg-x-link-transformer.env
ExecStart=/usr/local/bin/tg-x-link-transformer
Restart=always
RestartSec=5
User=nobody
Group=nogroup

[Install]
WantedBy=multi-user.target
```

Start it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now tg-x-link-transformer
sudo journalctl -u tg-x-link-transformer -f
```

## Development

```bash
cargo test
cargo fmt --check
cargo clippy -- -D warnings
docker build -t tg-x-link-transformer .
```
