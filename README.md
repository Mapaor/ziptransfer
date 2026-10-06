# Ziptransfer
This project is a re-write  of [Transfer.zip](https://transfer.zip), instead of node and mongo, it uses Rust and SQLite for the backend, and reuses Next for the frontend but simplifying it a lot and migrating from javascript to typescript. The aim is to provide a simpler but more reliable way to selfhost a file transfer server.

## Selfhosting
There are two types of transfers: direct transfers and persistent transfers. The first ones work peer to peer via WebRTC, while the persistent ones are stored on the server. If you want both, as we'll be dealing with large files Cloudflare Tunnel is not an option, so you'll need a powerful VPS or a simple VPS with Pangolin and a decent home server connected to that VPS. See the [CUSTOM_DOMAIN_GUIDE.md](CUSTOM_DOMAIN_GUIDE.md) for a selfhosting guide.

## License
This project is a modified version of [Transfer.zip](https://github.com/robinkarlberg/transfer.zip-web), originally developed by Robin Karlberg. Due to that the same AGPL-3.0 has been preserved.

See the [LICENSE](./LICENSE) file for the full license text.
