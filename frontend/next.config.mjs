/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "nbg1.your-objectstorage.com",
        pathname: "/**"
      }
    ]
  },
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        // In local development we proxy to the local Rust server. 
        // In docker, api resolves to the backend service.
        destination: process.env.NODE_ENV === 'development' 
            ? 'http://127.0.0.1:9000/api/:path*' 
            : 'http://api:9000/api/:path*'
      }
    ]
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  experimental: {
    serverActions: {
      bodySizeLimit: '10gb',
    },
    proxyPrefetch: 'flexible',
    proxyClientMaxBodySize: '10000mb'
  }
};

export default nextConfig;
