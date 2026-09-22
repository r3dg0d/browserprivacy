# browserprivacy

**Read-only** privacy auditor for **Chromium-family** and **Firefox-family** browser profiles on Linux.

Owner: **r3dg0d** · License: **MIT**

## Hard privacy guarantees

This tool **NEVER** extracts:

- passwords / Login Data  
- auth tokens  
- session cookies / cookie **values**  
- `key4.db` / NSS credential material  

It only reads **settings/prefs**, **permission defaults**, **extension inventories**, and **presence counts** for storage/service workers (not secret contents).

## Profiles searched

| Path | Browser |
|------|---------|
| `~/.mozilla/firefox` | Firefox |
| `~/.config/chromium` | Chromium |
| `~/.config/google-chrome` | Chrome |
| `~/.config/brave-browser` | Brave |
| `~/.config/microsoft-edge`, `vivaldi`, `opera` | Other Chromium |
| `~/.var/app/...` | Flatpak variants (optional) |

## Commands

```bash
browserprivacy              # same as audit
browserprivacy audit
browserprivacy list-profiles
browserprivacy report
browserprivacy --json audit
browserprivacy --dry-run audit
browserprivacy completions bash
```

Global flags: `--json` `--verbose` `--quiet` `--config` `--dry-run` `--help` `--version`

## What is checked

- WebRTC / peerconnection related prefs  
- DNS-over-HTTPS / TRR  
- Proxy mode  
- Cookie policy & third-party cookies  
- Local storage **presence counts** (not values)  
- Service worker-related entry counts  
- Site permissions (camera / mic / location / notifications)  
- Extension inventory  

Plus **informational** remediation suggestions.

## Config (XDG)

`~/.config/browserprivacy/config.toml`:

```toml
include_flatpak = true
# extra_profile_roots = ~/some/custom/profile-root
```

## Install

```bash
cargo install --path .
```

## Development

```bash
cargo test
cargo build --release
cargo run -- --help
```

## Disclaimer

Suggestions are educational. Browser privacy is a moving target — verify critical settings in the browser UI (`about:config`, `chrome://settings`) yourself.
