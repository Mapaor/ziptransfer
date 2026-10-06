# ZipTransfer Architecture Overview

This document explains the architecture of the ZipTransfer application. It describes the frontend, the Rust backend, and the SQLite database.

## 1. High-Level Architecture

ZipTransfer has a separate frontend and backend.
*   **Frontend**: The frontend uses Next.js (React) and TypeScript. It uses TailwindCSS for styles. It communicates with the backend through REST APIs and WebSockets.
*   **Backend**: The backend uses Rust and the `axum` web framework. It provides high performance and safe memory management.
*   **Database**: The database is `SQLite`. The backend manages it with `sqlx`.
*   **Storage**: The application stores files in the local `./uploads` directory.

## 2. Database Schema (SQLite)

The database has four tables. The tables record users, transfers, transfer requests, and files.

### `users`
This table records registered users.
*   `id` (TEXT, Primary Key)
*   `email` (TEXT, Unique)
*   `password_hash` (TEXT)
*   `session_token` (TEXT)
*   `reset_token` (TEXT) - Used  for password resets.
*   `reset_token_expires_at` (DATETIME)
*   `storage_limit` (INTEGER) - User settings.
*   `default_expiration_days` (INTEGER) - User settings.
*   `created_at` (DATETIME)

### `transfers`
This table records groups of files. Users transfer or store these files.
*   `id` (TEXT, Primary Key)
*   `user_id` (TEXT, Foreign Key -> `users.id`)
*   `type` (TEXT)
*   `name` (TEXT)
*   `description` (TEXT)
*   `expires_at` (DATETIME)
*   `created_at` (DATETIME)

### `files`
This table records each uploaded file.
*   `id` (TEXT, Primary Key)
*   `transfer_id` (TEXT, Foreign Key -> `transfers.id`)
*   `filename` (TEXT)
*   `size` (INTEGER)
*   `disk_path` (TEXT)
*   `created_at` (DATETIME)

### `transfer_requests`
This table records file requests. A user sends a request to an external uploader.
*   `id` (TEXT, Primary Key)
*   `user_id` (TEXT, Foreign Key -> `users.id`)
*   `name` (TEXT)
*   `description` (TEXT)
*   `status` (TEXT)
*   `created_at` (DATETIME)

## 3. Backend API (Rust + Axum)

The backend has API routes under the `/api` path.

### Authentication and Users (`auth.rs`)
*   `POST /api/auth/register`: Register a new user.
*   `POST /api/auth/login`: Authenticate a user.
*   `POST /api/auth/logout`: Delete the session token and close the session.
*   `POST /api/auth/passwordreset/request`: Start a password reset procedure.
*   `POST /api/auth/passwordreset/do`: Complete a password reset.
*   `PUT /api/user/settings`: Change user settings.

### File Transfers and Requests (`transfer.rs`)
*   `GET /api/transfer/list`: Returns a list of transfers for the user.
*   `POST /api/transfer/new`: Creates a new transfer record.
*   `GET /api/transfer/:id`: Returns data related to a specific transfer (metadata, not the files themselves).
*   `DELETE /api/transfer/:id`: Deletes a transfer and removes the associated files.
*   `GET /api/transferrequest/list`: Gets a list of transfer requests for the user.
*   `POST /api/transferrequest/new`: Creates a new transfer request.
*   `POST /api/sign`: Generates a token for authorized operations.

### TUS Upload Protocol
*   `POST /api/upload`: Starts a new file upload.
*   `HEAD /api/upload/:id`: Returns the current `Upload-Offset`.
*   `PATCH /api/upload/:id`: Appends binary data to the file.
*   `POST /api/upload/:id/complete`: Marks a file as completed (changes its status to completed).
*   `GET /api/upload/:id`: Returns public transfer data without requiring authentication.

### Downloads
*   `GET /api/download/:id`: Send a specific file to the user as an `application/octet-stream`.
*   `POST /api/download`: Reads a `DownloadForm` (token) and finds the corresponding file.
*   `POST /api/download/:id/downloaded`: Adds one more download to the stats.

### WebRTC Signaling (`signaling.rs`)
*   `GET /api/signaling`: Changes the connection to a WebSocket connection. The backend uses a custom signaling protocol (`CPKT_*` / `SPKT_*`). It finds peers, sends SDP Offers/Answers, and sends ICE candidates. It also sends binary data (`Message::Binary`) if the peer-to-peer connection fails.

## 4. Frontend TypeScript API Wrapper (`Api.ts`)

The frontend uses `Api.ts` to connect with the backend. This file contains all the backend routes. The frontend uses TypeScript interfaces (`Transfer`, `User`, `ApiResponse`) to check data types.

### Completed Functions
*   Authentication (Login, Register, Logout, Password Reset).
*   Transfer Management (List, Create, Retrieve, Delete).
*   Transfer Requests (List, Create).
*   User Settings (Change Storage limits).
*   File Uploads (TUS Protocol, WebRTC P2P Direct).
*   File Downloads.

### Missing Functions
The frontend has routes for these functions, but the backend does not have them yet. They return `404 Not Found`:
*   `PUT /transfer/:id` (Change Transfer Name/Description)
*   `POST /transfer/:id/sendbyemail`
*   `POST /transferrequest/:id/sendbyemail`
*   `POST /transferrequest/:id/activate`
*   `POST /transferrequest/:id/deactivate`

*Note: I will implement transfers via email and transfer requests in the next release.*

## 5. WebRTC Peer-to-Peer and Relay Architecture

ZipTransfer tries to use peer-to-peer (P2P) connections for file transfers. The WebRTC system has three connection levels. This makes sure that the file transfers correctly, regardless of the type of LAN each device is in, or the strictness of the firewall configurations each NAT might have.

1. **Direct connection via STUN (`typ host` / `typ srflx`) -> Works when both devices are on the same LAN**
   * If two devices use the same local network, WebRTC connects them with local IP addresses.
   * If the devices use different networks, the frontend asks Google STUN servers for their public IP addresses. The backend supplies the STUN server addresses.
2. **Relay connection via custom TURN server (`typ relay`) -> Always works (but requires setting up the turn server on a server with a VPS)**
   * If the direct connection fails, the application sends data through a TURN server (`turn-rs`).
   * The backend makes temporary (24-hour) HMAC-SHA1 credentials (`turn.rs: /api/turn-credentials`). The frontend gets these credentials to connect to the TURN server.
   * *Note: Install the TURN server on a VPS, not on your home server. Tunnels like newt or WireGuard and relay ranges of UDP ports don't get along well. Also you'll need a public IP that is a basic requirement for any TURN server, so you'll need a VPS either way.*
3. **WebSocket Binary Connection (`SPKT_SWITCH_TO_FALLBACK`) -> Works as a fallback, it's slower and doesn't handle well very large files but it will always work for small files.**
   * If the WebRTC connection fails or stops (8.2 seconds), the frontend sends binary data through the signaling WebSocket.
   * The backend proxy (`signaling.rs`) sends the binary data between the two devices.
