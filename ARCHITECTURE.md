# ZipTransfer Architecture Overview

This document provides a comprehensive overview of the architecture of the ZipTransfer application, highlighting the interaction between the frontend, the Rust backend, and the SQLite database. 

## 1. High-Level Architecture

ZipTransfer uses a decoupled frontend-backend architecture:
*   **Frontend**: Built with Next.js (React) and strict TypeScript, styled with TailwindCSS, communicating with the backend via REST endpoints and WebSockets for real-time signaling.
*   **Backend**: Built with Rust utilizing the `axum` web framework for lightning-fast performance, memory safety, and high concurrency.
*   **Database**: Uses `SQLite` as the lightweight, serverless relational database managed via `sqlx`.
*   **Storage**: Files are stored directly on the local filesystem (`./uploads` directory in the container).

*Note: The platform is designed as a free utility for friends and family, meaning all historical billing, SaaS, and payment processing features (like Stripe) have been intentionally removed.*

---

## 2. Database Schema (SQLite)

The database consists of four core tables to track users, their transfers, transfer requests, and the individual files.

### `users`
Tracks registered users.
*   `id` (TEXT, Primary Key)
*   `email` (TEXT, Unique)
*   `password_hash` (TEXT)
*   `session_token` (TEXT)
*   `reset_token` (TEXT) - Used for password resets
*   `reset_token_expires_at` (DATETIME)
*   `storage_limit` (INTEGER) - Settings
*   `default_expiration_days` (INTEGER) - Settings
*   `created_at` (DATETIME)

### `transfers`
Represents a grouped collection of files to be transferred or stored persistently.
*   `id` (TEXT, Primary Key)
*   `user_id` (TEXT, Foreign Key -> `users.id`)
*   `type` (TEXT)
*   `name` (TEXT)
*   `description` (TEXT)
*   `expires_at` (DATETIME)
*   `created_at` (DATETIME)

### `files`
Tracks individual files uploaded within a transfer.
*   `id` (TEXT, Primary Key)
*   `transfer_id` (TEXT, Foreign Key -> `transfers.id`)
*   `filename` (TEXT)
*   `size` (INTEGER)
*   `disk_path` (TEXT)
*   `created_at` (DATETIME)

### `transfer_requests`
Tracks requests for files sent by a user to external uploaders.
*   `id` (TEXT, Primary Key)
*   `user_id` (TEXT, Foreign Key -> `users.id`)
*   `name` (TEXT)
*   `description` (TEXT)
*   `status` (TEXT)
*   `created_at` (DATETIME)

---

## 3. Backend Endpoints (Rust + Axum)

The Rust backend exposes several endpoints under the `/api` route.

### Authentication & Users (`auth.rs`)
*   `POST /api/auth/register`: Registers a new user.
*   `POST /api/auth/login`: Authenticates a user.
*   `POST /api/auth/logout`: Clears the session token and terminates the user session.
*   `POST /api/auth/passwordreset/request`: Initiates a password reset.
*   `POST /api/auth/passwordreset/do`: Confirms a password reset.
*   `PUT /api/user/settings`: Updates user configurations.

### File Transfers & Requests (`transfer.rs`)
*   `GET /api/transfer/list`: Returns a list of transfers for the authenticated user.
*   `POST /api/transfer/new`: Instantiates a new transfer record.
*   `GET /api/transfer/:id`: Returns metadata for a specific transfer.
*   `DELETE /api/transfer/:id`: Safely deletes a transfer and purges associated file blobs.
*   `GET /api/transferrequest/list`: Lists transfer requests for the user.
*   `POST /api/transferrequest/new`: Creates a new transfer request.
*   `POST /api/sign`: Generates a token for authorized operations.

### TUS Upload Protocol
*   `POST /api/upload`: Initializes a new file upload.
*   `HEAD /api/upload/:id`: Returns the current `Upload-Offset`.
*   `PATCH /api/upload/:id`: Appends chunked binary data.
*   `POST /api/upload/:id/complete`: Marks a file as completed.
*   `GET /api/upload/:id`: Returns public transfer data without requiring authentication.

### Downloads
*   `GET /api/download/:id`: Streams a specific file ID directly to the user as an `application/octet-stream`.
*   `POST /api/download`: Accepts a `DownloadForm` (token) and resolves the file for downloading (used by the frontend Node.js proxy layer).
*   `POST /api/download/:id/downloaded`: Registers a download statistic metric.

### WebRTC Signaling (`signaling.rs`)
*   `GET /api/signaling`: Upgrades the connection to a WebSocket. It implements a custom packet-based signaling protocol (`CPKT_*` / `SPKT_*` constants) for peer discovery, exchanging SDP Offers/Answers, ICE candidates, and relaying binary data (`Message::Binary`) if the peers fallback from a direct P2P connection to server-relayed data.

---

## 4. Frontend Typescript API Wrapper (`Api.ts`)

The frontend interacts with the backend using a strongly-typed `Api.ts` wrapper. All endpoints defined in the Rust backend are mapped here. Since the TypeScript migration, the API client relies on explicit DTO interfaces (`Transfer`, `User`, `ApiResponse`, etc.) to enforce strict type checking across the UI components.

### ✅ Implemented & Functional Features
*   Authentication (Login, Register, Logout, Password Reset)
*   Transfer Management (List, Create, Retrieve, Delete)
*   Transfer Requests (List, Create)
*   User Settings (Update Storage limits, etc.)
*   File Uploads (TUS Protocol, WebRTC P2P Direct)
*   File Downloads

### ⚠️ Missing / Pending Implementation
The following endpoints are stubbed in the frontend but return `404 Not Found` because they are not yet implemented in the Rust backend (or are missing UI elements):

*   `PUT /transfer/:id` (Update Transfer Name/Description)
*   `POST /transfer/:id/sendbyemail`
*   `POST /transferrequest/:id/sendbyemail`
*   `POST /transferrequest/:id/activate`
*   `POST /transferrequest/:id/deactivate`

---

## 5. Conclusion & Next Steps

The application has successfully migrated from a Node/Mongo/JS stack to a modern, type-safe Rust/SQLite/TS architecture. The SaaS components have been stripped away to optimize it for a private, friends-and-family use case. The core authentication loop, including logout and file deletion mechanics, is fully operational.

**Priorities for Next Week:**
1.  **Email Integrations**: Implement email sharing services (e.g., using ForwardEmail or Cloudflare Email Routing) to actually process `sendbyemail` calls for both transfers and requests.
2.  **UI Polish**: Clean up the `branding` and `settings` views to reflect the simplified, non-commercial feature set.

---

## 6. WebRTC P2P & Relay Architecture

ZipTransfer prioritizes peer-to-peer (P2P) connections for "Quick Share" file transfers. The WebRTC pipeline gracefully degrades through three tiers of connectivity to guarantee file delivery regardless of NAT strictness or firewall configurations:

1. **Direct P2P via Local Network & STUN (`typ host` / `typ srflx`)**
   * If two devices are on the same local network, WebRTC attempts to connect directly via local IPs (host candidates).
   * If they are on different networks, the frontend fetches Google's public STUN servers (always provided by the backend) to discover their public IP addresses and punch through standard NATs.
2. **Relay via Custom TURN Server (`typ relay`)**
   * If direct P2P fails (e.g., symmetric NATs or strict firewalls), traffic is relayed through a standalone TURN server (`turn-rs`).
   * The backend dynamically generates temporary (24-hour) HMAC-SHA1 signed credentials (`turn.rs: /api/turn-credentials`) that the frontend fetches asynchronously to authenticate against the TURN server.
   * *Note: The TURN server is designed to be hosted natively on a VPS to bypass local WireGuard tunnel UDP limitations, while the backend API configures it securely.*
3. **WebSocket Binary Fallback (`SPKT_SWITCH_TO_FALLBACK`)**
   * If the WebRTC ICE connection fails or times out (8.2s), the frontend gracefully falls back to sending binary file chunks directly over the signaling WebSocket.
   * The backend's `signaling.rs` seamlessly proxies binary packets between the `sender` and `receiver` as a last-resort relay mechanism.
