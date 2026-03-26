/** @type {import('next').NextConfig} */
const nextConfig = {
  // output: 'export', // Comment this out if you need dynamic features
  eslint: {
    ignoreDuringBuilds: true,
  },
  typescript: {
    ignoreBuildErrors: true,
  },
  images: { unoptimized: true },
};

module.exports = nextConfig;
