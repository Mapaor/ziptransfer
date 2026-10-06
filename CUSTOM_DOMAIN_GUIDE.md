## Requisites
- A VPS with Pangolin set up
- Caddy running as a container on your home server
- Newt configured on your home server so it is connected with your VPS
- A custom domain, that is already set up to work with Pangolin

## Steps

### 1. Create the Pangolin resources
Create two Pangolin Public Resources, one for the api and one for the web. Each at your desired domain (eg. transfer.yourdomain.com and api.transfer.yourdomain.com), they both should point to:

```
http://caddy:80
```

### 2. Set up Caddy
Modify your Caddyfile adding to it:
```
# TRANSFERIR
http://transfer.yourdomain.com:80 {
    reverse_proxy ziptransfer-web:9001 {
        header_up X-Forwarded-Proto https
        header_up Host {host}
        header_up X-Real-IP {remote_host}
    }
}

# API TRANSFERIR
http://api.transfer.yourdomain.com:80 {
    reverse_proxy ziptransfer-api:9000 {
        header_up X-Forwarded-Proto https
        header_up Host {host}
    }

    @options method OPTIONS
    handle @options {
        respond "" 204
    }

    header Access-Control-Allow-Origin "https://transfer.yourdomain.com"
    header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS, PATCH"
    header Access-Control-Allow-Headers "Content-Type, Authorization, tus-resumable, upload-length, upload-offset, upload-metadata"
    header Access-Control-Expose-Headers "Authorization, Content-Type, Location, Tus-Extension, Tus-Max-Size, Tus-Resumable, Tus-Version, Upload-Concat, Upload-Defer-Length, Upload-Length, Upload-Metadata, Upload-Offset, X-HTTP-Method-Override, X-Requested-With, X-Forwarded-Host, X-Forwarded-Proto, Forwarded"
    header Access-Control-Allow-Credentials "true"
}
```

Now modify your Caddy docker compose file with:
```yml
services:
  caddy:
    container_name: caddy
    image: caddy:latest
    restart: unless-stopped
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    ports:
      - "80:80"
      - "443:443"
    networks:
      - ziptransfer-net # Add this line

volumes:
  caddy_data:
  caddy_config:

networks:
  ziptransfer-net: # Also add this line
    external: true # And this one
```

Now run the following command:
```bash
docker network create ziptransfer-net
```

Now you can restart caddy:
```bash
docker compose up -d --force-recreate
```

### 3. Set up Ziptransfer
On the `.env` set:
```env
WEB_URL=https://transfer.yourdomain.com
SERVER_URL=https://api.transfer.yourdomain.com
```

Then on the `docker-compose.yml` uncomment the network lines:
```yml
services:
  web:
    container_name: ziptransfer-web
    build: frontend
    restart: unless-stopped
    env_file:
      - .env
    ports:
      - "127.0.0.1:${WEB_SERVER_FORWARD_PORT:-9001}:9001"
    depends_on:
      - api
    # Uncomment the following:
    networks:
      - ziptransfer-net
    

  api:
    container_name: ziptransfer-api
    build: backend
    restart: unless-stopped
    env_file:
      - .env
    ports:
      - "127.0.0.1:${BACKEND_FORWARD_PORT:-9000}:9000"
    volumes:
      - api_data:/app/data
    # Uncomment the following:
    networks:
      - ziptransfer-net

volumes:
  api_data:

# Uncomment the following:
networks:
  ziptransfer-net:
    external: true
```

Now run it:
```bash
docker compose up -d --build
```

### 4. Configure a Turn Server on the VPS

Because reverse tunnels like Pangolin/Newt cannot efficiently forward raw UDP port ranges, the WebRTC TURN server must run directly on your VPS.

#### 4.1 Configure the turn server
On your VPS, create a directory, call it `turn-rs` or `turn-server` or however you want, then do `cd` to that directory and in there create a file named `turn-server.toml` with the following contents:
```toml
[server]
port-range = "52000..65535"
max-threads = 4
realm = "transfer.yourdomain.com" # The domain where the ziptransfer-web will be hosted

[[server.interfaces]]
transport = "udp"
listen = "0.0.0.0:3478"
external = "YOUR_VPS_PUBLIC_IP:3478"

[[server.interfaces]]
transport = "tcp"
listen = "0.0.0.0:3478"
external = "YOUR_VPS_PUBLIC_IP:3478"

[log]
level = "info"
stdout = true

[auth]
static-auth-secret = "your_super_secret_key"
```
*Note: You need to manually change the realm, VPS public IP, and secret key*

#### 4.2 Set up the OS firewall
If you are on Ubuntu Server, you must open the STUN/TURN port and the UDP relay port range on your VPS firewall using `ufw`:
```bash
sudo ufw allow 3478/tcp
sudo ufw allow 3478/udp
sudo ufw allow 52000:65535/udp
```
#### 4.3 Set up the VPS firewall
You'll need to set them in your provider's console (AWS, Oracle, Azure, Ionos, etc.) as well.

<img width="1057" height="648" alt="imatge" src="https://github.com/user-attachments/assets/7f1e7392-9b8b-4909-887d-040e91735ad5" />


*Note: If you have other services running on ports above 52000 you can shorten the range of the ports so it does not affect your used port, Turn will continue to work as well. I replaced 49152 with 52000 precisely because newt being on port 51820.*

#### 4.4 Start the turn server with docker
Start the TURN server on the VPS. We use `network host` to avoid Docker NAT issues with UDP port allocations,m run from that same directory the following command:
```bash
docker run -d --network host --name turn-server --restart unless-stopped \
  -v $(pwd)/turn-server.toml:/etc/turn-server/config.toml \
  ghcr.io/mycrl/turn-server:4.1.5
```

#### 4.5 Restart Ziptransfer
Finally, back on your home server, update Ziptransfer's `.env` file to use it with:
```env
TURN_DOMAIN=YOUR_VPS_PUBLIC_IP
TURN_SHARED_SECRET=your_super_secret_key
```

And restart Ziptransfer to apply the changes:
```bash
docker compose up -d --force-recreate
```
