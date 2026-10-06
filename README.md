# Ziptransfer
This project is a re-write  of [Transfer.zip](https://transfer.zip), instead of node and mongo, it uses Rust and SQLite for the backend, and reuses Next for the frontend but simplifying it a lot and migrating from javascript to typescript. The aim is to provide a simpler but more reliable way to selfhost a file transfer server.

## Selfhosting
There are two types of transfers: direct transfers and persistent transfers. The first ones work peer to peer via WebRTC, while the persistent ones are stored on the server. If you want both, as we'll be dealing with large files Cloudflare Tunnel is not an option, so you'll need a powerful VPS or a simple VPS with Pangolin and a decent home server connected to that VPS. See the [CUSTOM_DOMAIN_GUIDE.md](CUSTOM_DOMAIN_GUIDE.md) for a selfhosting guide.

For P2P transfers you can use a custom TURN server, which allows NAT-traversal for (quick) direct transfers even when the NAT is very strict or devices are on different LANs. For more information see [ARCHITECTURE.md](ARCHITECTURE.md).

## Roadmap

### In the near future
- [X] Custom TURN server
- [ ] Transfer requests
- [ ] Email SMTP (for sending transfers via email and other email notifications)
- [ ] Create admin user that can manage all transfers and other users
- [ ] Disable or enable sign-up. Only admin can sign up new users.
- [ ] Create user quota and handle logical volumes space available properly
- [ ] Password reset via email link
- [ ] Google OAuth2 option to Sign-in and Sign-up

### In the far future
- [ ] Upgrade and migrate the whole UI to a newer one, that distances it from the original Transfer.zip in a good sense, making it more original/cute and less corporative.
- [ ] Add internationalization support for the UI (Catalan, Spanish, English for now)
- [ ] Add support for multiple storage backends (local, S3, etc.)

## License
This project is a modified version of [Transfer.zip](https://github.com/robinkarlberg/transfer.zip-web), originally developed by Robin Karlberg. Due to that the same AGPL-3.0 has been preserved.

See the [LICENSE](./LICENSE) file for the full license text.
