/** @type {import('next').NextConfig} */
const nextConfig = {
  // output: 'export', // Comment this out if you need dynamic features
  eslint: {
    ignoreDuringBuilds: true,
  },
  images: { unoptimized: true },
  transpilePackages: [
    'passkey-kit',
    'passkey-factory-sdk',
    'passkey-kit-sdk',
    'sac-sdk',
  ],
};

module.exports = nextConfig;
