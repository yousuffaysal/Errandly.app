# Releasing Errandly

Releases are built, signed, notarized and published by
`.github/workflows/release.yml` when you push a version tag. Installed copies
update themselves from the published release.

## One-time setup

### 1. Apple Developer Program ($99/year)
Needed so macOS opens Errandly without a security warning.

1. Join at <https://developer.apple.com/programs/>.
2. In Xcode → Settings → Accounts, or at developer.apple.com → Certificates,
   create a **Developer ID Application** certificate.
3. Export it from Keychain Access as a `.p12` with a password.
4. Create an **app-specific password** at <https://account.apple.com> → Sign-In and Security.

### 2. GitHub secrets
Repository → Settings → Secrets and variables → Actions → New repository secret:

| Secret | Value |
|---|---|
| `APPLE_CERTIFICATE` | `base64 -i certificate.p12 \| pbcopy`, then paste |
| `APPLE_CERTIFICATE_PASSWORD` | The `.p12` export password |
| `APPLE_SIGNING_IDENTITY` | e.g. `Developer ID Application: Your Name (TEAMID)` |
| `APPLE_ID` | Your Apple ID email |
| `APPLE_PASSWORD` | The app-specific password |
| `APPLE_TEAM_ID` | Your 10-character Team ID |
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of `~/.tauri/errandly.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Empty (the key was created without a password) |
| `VITE_SUPABASE_URL` | `https://spksmpgnjsjydlhefqjc.supabase.co` |
| `VITE_SUPABASE_ANON_KEY` | The publishable key |

**Keep `~/.tauri/errandly.key` safe and backed up.** Every update is signed with
it, and installed copies only accept updates signed by it. If it's lost, existing
users can't auto-update; if it leaks, someone could sign updates as you.

### 3. Supabase (once)
- **Authentication → URL Configuration → Site URL**: your website (or any page
  you control). Email confirmation links land there.
- **Redirect URLs**: `http://127.0.0.1:53682/auth/callback` (Google sign-in).
- **SQL Editor**: run `supabase/migrations/20261008000000_crash_reports.sql`
  so opt-in crash reports have somewhere to go.

## Each release

1. Bump `version` in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and `package.json`.
2. Commit, then tag and push:
   ```sh
   git tag v0.2.0 && git push origin v0.2.0
   ```
3. When the workflow finishes, open the **draft release** on GitHub, write the
   notes, and **Publish**. Installed apps see it within one launch.

## Building locally

```sh
sh scripts/fetch-runtime.sh            # bundled AI runtime (pinned, checksum-verified)
TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/errandly.key)" TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" \
  pnpm tauri build                      # unsigned unless APPLE_SIGNING_IDENTITY is set
```
The app is at `src-tauri/target/release/bundle/macos/Errandly.app`, the disk
image under `bundle/dmg/`.
