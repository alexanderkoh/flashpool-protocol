import { NextRequest, NextResponse } from 'next/server';
import { PasskeyServer } from 'passkey-kit';

// Initialize PasskeyServer
// Ensure all required environment variables are set in your .env.local or server environment
const rpcUrl = process.env.RPC_URL;
const launchtubeUrl = process.env.LAUNCHTUBE_URL;
const launchtubeJwt = process.env.PRIVATE_LAUNCHTUBE_JWT;
const mercuryUrl = process.env.MERCURY_URL; // Optional, for event indexing
const mercuryJwt = process.env.PRIVATE_MERCURY_JWT; // Optional, for event indexing

let passkeyServer: PasskeyServer | null = null;
let initializationError: string | null = null;

try {
  if (!rpcUrl || !launchtubeUrl || !launchtubeJwt) {
    throw new Error(
      'Missing required server environment variables for PasskeyServer: RPC_URL, LAUNCHTUBE_URL, PRIVATE_LAUNCHTUBE_JWT'
    );
  }
  passkeyServer = new PasskeyServer({
    rpcUrl,
    launchtubeUrl,
    launchtubeJwt,
    mercuryUrl, // Can be undefined if not used
    mercuryJwt, // Can be undefined if not used
  });
} catch (e: any) {
  console.error('Failed to initialize PasskeyServer:', e);
  initializationError = e.message || 'Failed to initialize PasskeyServer';
}

async function handler(req: NextRequest) {
  if (initializationError || !passkeyServer) {
    return NextResponse.json(
      { error: 'PasskeyServer not initialized', details: initializationError },
      { status: 500 }
    );
  }

  // The `route` parameter will be an array of path segments.
  // For example, /api/passkey/register -> params.route = ['register']
  // For example, /api/passkey/challenge/user123 -> params.route = ['challenge', 'user123']
  const route = (req.nextUrl.pathname.split('/api/passkey/').pop() || '').split('/');
  const action = route[0];

  try {
    // Example: Handling a registration challenge request
    if (req.method === 'POST' && action === 'register-challenge') {
      const { username } = await req.json();
      if (!username) {
        return NextResponse.json({ error: 'Username is required' }, { status: 400 });
      }
      // const challenge = await passkeyServer.getRegistrationChallenge(username);
      // return NextResponse.json(challenge);
      return NextResponse.json({ message: 'Register challenge endpoint placeholder', username });
    }

    // Example: Handling a registration verification request
    if (req.method === 'POST' && action === 'register-verify') {
      const body = await req.json();
      // const result = await passkeyServer.verifyRegistration(body);
      // return NextResponse.json(result);
      return NextResponse.json({ message: 'Register verify endpoint placeholder', body });
    }

    // Add more actions for login, transaction signing, etc.
    // These would call appropriate methods on `passkeyServer`

    return NextResponse.json({ error: 'Unknown passkey action' }, { status: 404 });
  } catch (error: any) {
    console.error(`Passkey API error for action "${action}":`, error);
    return NextResponse.json(
      { error: 'Internal Server Error', details: error.message },
      { status: 500 }
    );
  }
}

export { handler as GET, handler as POST, handler as PUT, handler as DELETE }; 