# Boron

## Run with Docker

Boron uses SQLite. Docker stores the database and uploaded files in the `boron_sqlite_data` volume. Back up this volume before upgrading.

Before first deployment, set a strong, persistent `JWT_SECRET` in `docker/.env` (run this once and keep the file private):

```sh
printf 'JWT_SECRET=%s\n' "$(openssl rand -hex 32)" > docker/.env
chmod 600 docker/.env
docker compose -f docker/docker-compose.yml up --build
```

Open `http://localhost:57575`. Do not regenerate the secret on every startup: changing it invalidates existing sessions. The example/default secret is not safe for production.

The backend creates the SQLite file and runs migrations on startup. Existing SQLite data is preserved. The app does not automatically import data from MySQL.

## Run locally on Windows

Keep your existing backend `.env`, database and uploads when upgrading. In particular, keep `DATABASE_URL`, `UPLOAD_DIR`, and `JWT_SECRET` pointed at the same persistent data; an extracted new folder otherwise starts with a separate database. If this is a new installation, copy `backend/.env.example` to `backend/.env` and set a strong, stable `JWT_SECRET`.

1. Stop the old frontend and backend processes. If you are unsure what is using the backend port, in PowerShell run `Get-NetTCPConnection -LocalPort 3001 -State Listen | Select-Object LocalAddress, OwningProcess`, then use `Get-Process -Id <PID>` to identify it. Stop only the old Boron process; do not stop an unrelated service.
2. In a terminal, enter the **updated** project's `backend` folder and run `cargo run --locked`. Leave it running. This rebuilds and runs the backend containing the chat routes.
3. In another terminal, enter the **updated** project's `frontend` folder, run `npm ci`, then run `npm run dev -- --host 127.0.0.1 --port 57575`.
4. Open `http://localhost:57575/api/version`. It should return JSON with `service` set to `boron-backend` and `api_revision` at least `7`. Open the app and refresh it. If the version URL is missing or returns something else, the frontend proxy is still reaching an old/different backend. Verify port 3001 and any custom `VITE_API_URL`.

If a pins request returns `404` or the version notice persists, compare both routes in PowerShell (no login token is needed):

```powershell
Invoke-RestMethod http://127.0.0.1:3001/api/version
Invoke-RestMethod http://localhost:57575/api/version
```

Both should identify `boron-backend` with `api_revision` 7 or higher. If the direct backend returns an older revision or fails, check which process is listening on port 3001 and start the updated backend. If the direct backend is correct but the frontend URL differs, check the Vite/nginx proxy or `VITE_API_URL`; a browser configured with `VITE_API_URL` may contact that separate address directly. A version 7 response alongside a missing route can indicate requests reaching different servers or a routing problem, not necessarily that a rebuild failed. The pins route is `GET /api/channels/<channel-id>/pins` and requires the user's authorization and channel access. Do not paste authorization tokens into diagnostics.

When the full pins route is unavailable, chat still loads and the pins panel shows pins from the latest 200 loaded messages; use Retry pins after fixing the server. A `GET /api/channels/unread` response of `405` with `Allow: DELETE` is a strong sign of an older or mismatched route. Restarting only the frontend will not update that server.

## Sessions and private spaces

The browser stores the signed session token locally and restores it on reload and navigation. The default session lifetime is 30 days (`JWT_EXPIRATION` in seconds); signing out removes it. Anyone with access to the browser profile or its local storage can use that token, so sign out on shared devices.

New chat channels can be private or public; private is the default. Public channels are accessible to all team members automatically. Private channels, issue boards and pages are accessible only to explicitly added people and team owners/admins. Existing spaces retain their visibility. An admin can add an existing active account by username or email, using the suggestions to disambiguate names. Adding a person to a space also adds them to the team, but does not grant access to other private spaces. Uploads are limited to 800 MiB per file and streamed to the `uploads` directory. Existing SQLite-stored avatars and attachments remain readable. Chat streams image and GIF previews; very large images may require substantial browser memory. File links expire after five minutes and recheck access. Outside Docker, set `UPLOAD_DIR` to a persistent, writable directory and back it up along with the database.

Team pictures use the persistent uploads directory and, like user profile pictures, are served by an unguessable team ID without a bearer token so browsers can display them. Back up uploads together with the database.

## Issue assignments and priority

Issues can have multiple assignees. Only people with access to the issue board can be selected. Priority is shown with a label and icon as well as color; high and critical issues also have a stronger card edge. The database migration preserves existing single-person assignments.

## Mentions and stickers

Mention someone from the chat suggestions to notify them. Unread mention counts are shown in the team navigation and clear when the channel is viewed; the mentioned message remains highlighted. Custom image stickers are stored in the same persistent uploads directory. Their owner or a global administrator can delete them from the sticker picker with a right-click.

## Profiles, switching and reminders

The account menu at the bottom of the sidebar opens settings for the display name, bio, avatar, theme and notification permission. You can add another account and switch between saved sessions; session tokens for those accounts are stored in this browser's local storage. Sign out of each account on shared devices.

Message reminders are saved on the server. While the app is open it checks for due reminders, shows an in-app alert and, if permission was granted, sends a browser notification and plays a short sound. If Boron is closed or offline, overdue reminders are delivered when you next open it; this is not background push. Browser notifications require a supported browser and a secure context (HTTPS or localhost); browsers may require a user interaction before playing sound.

When upgrading, rebuild **both** frontend and backend together and restart them so chat, unread and reminder routes match. Back up the persistent database and uploads first. Video previews depend on browser codec support; downloads work independently.

## Test the backend

```sh
cd backend
cargo build --locked
python3 tests/workspace_smoke.py
python3 tests/chat_smoke.py
python3 tests/features_smoke.py
python3 tests/message_actions_smoke.py
python3 tests/issue_assignees_smoke.py
python3 tests/issue_assignees_migration.py
python3 tests/public_pins_teams_smoke.py
```

## Updating an existing Windows installation in place

If both version URLs still show revision 6, extract the new update separately and run `UPDATE-EXISTING-WINDOWS.cmd` from the extracted folder. Close your old Boron backend and frontend terminals first. When asked, paste the path to the existing project folder that contains `backend` and `frontend`. The updater copies application source into that existing folder, preserves its `.env`, database, and uploads, cleans the Boron build artifacts, rebuilds, and starts both services in the same window. It checks that ports 3001 and 57575 are free without stopping any other processes. It verifies both version URLs; leave that window open. Back up your database and uploads before updating.
