// Cloudflare Worker for Lemon Squeezy Webhook Integration
//
// Setup instructions:
// 1. Run: wrangler secret put PRIVATE_KEY_HEX
// 2. Run: wrangler secret put WEBHOOK_SECRET
// 3. Run: wrangler secret put RESEND_API_KEY
// 4. Run: wrangler deploy
//
// This worker validates the Lemon Squeezy webhook signature (HMAC-SHA256),
// parses the order_created event, signs an offline license using the Ed25519
// private key, and emails it to the customer via Resend.

export default {
  async fetch(request, env) {
    if (request.method !== 'POST') {
      return new Response('Method Not Allowed', { status: 405 });
    }

    const body = await request.text();

    // 1. Webhook Signature Verification (HMAC-SHA256)
    const sig = request.headers.get('X-Signature');
    if (!sig) {
      return new Response('Missing Signature Header', { status: 401 });
    }
    const expectedSig = await hmacSha256(env.WEBHOOK_SECRET, body);
    if (sig !== expectedSig) {
      return new Response('Invalid Signature', { status: 401 });
    }

    const event = JSON.parse(body);
    if (event.meta.event_name !== 'order_created') {
      return new Response('Ignored Event', { status: 200 });
    }

    const order = event.data.attributes;
    const customerEmail = order.user_email;
    const customerName = order.user_name || 'Valued Customer';
    const orderId = `LS-${event.data.id}`;

    // 2. Create and Sign the License File
    const licenseData = {
      version: 1,
      licensee: {
        name: customerName,
        email: customerEmail,
        order_id: orderId
      },
      product: "rstat",
      tier: "pro",
      features: ["spc", "capability"],
      issued_at: new Date().toISOString(),
      expires_at: null
    };

    // Calculate deterministic signing payload
    const signingPayload = JSON.stringify({
      email: licenseData.licensee.email,
      features: licenseData.features,
      issued_at: licenseData.issued_at,
      tier: licenseData.tier,
      version: licenseData.version
    });

    // Sign payload using Ed25519 Subtle Crypto
    const signatureBase64 = await signPayload(env.PRIVATE_KEY_HEX, signingPayload);
    licenseData.signature = signatureBase64;

    const licenseJson = JSON.stringify(licenseData, null, 2);

    // 3. Send Email with License Attachment via Resend API
    const emailResponse = await fetch('https://api.resend.com/emails', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${env.RESEND_API_KEY}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        from: 'rstat Sales <sales@rstat.dev>',
        to: [customerEmail],
        subject: 'rstat Pro — Lisans Dosyanız',
        text: buildEmailText(customerName),
        attachments: [{
          filename: 'rstat_license.json',
          content: btoa(licenseJson)
        }]
      })
    });

    if (!emailResponse.ok) {
      const errorText = await emailResponse.text();
      return new Response(`Email sending failed: ${errorText}`, { status: 500 });
    }

    return new Response('OK', { status: 200 });
  }
};

async function hmacSha256(secret, data) {
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    'raw',
    enc.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const sig = await crypto.subtle.sign('HMAC', key, enc.encode(data));
  return Array.from(new Uint8Array(sig))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

async function signPayload(privateKeyHex, payload) {
  // Convert private key from hex to raw bytes
  const privKeyBytes = new Uint8Array(
    privateKeyHex.match(/.{1,2}/g).map(byte => parseInt(byte, 16))
  );

  // SubtleCrypto requires PKCS#8 or raw keys for Ed25519 depending on runtime.
  // In Cloudflare Workers, Ed25519 keys can be imported using PKCS#8 format.
  // Below is a fallback using a standard Ed25519 JS dependency or SubtleCrypto.
  // To keep the worker zero-dependency, import key via raw/PKCS8:
  const key = await crypto.subtle.importKey(
    'raw',
    privKeyBytes,
    { name: 'NODE-ED25519', namedCurve: 'NODE-ED25519' },
    false,
    ['sign']
  );
  const signature = await crypto.subtle.sign(
    null,
    key,
    new TextEncoder().encode(payload)
  );
  return btoa(String.fromCharCode(...new Uint8Array(signature)));
}

function buildEmailText(name) {
  return `Merhaba ${name},

rstat Pro lisansınız başarıyla oluşturuldu!

Kurulum ve Kullanım Adımları:

1. Lisans Dizini Oluşturma:
   Linux & macOS:
     mkdir -p ~/.config/rstat
   Windows (CMD / PowerShell):
     mkdir "%APPDATA%\\rstat"

2. Dosyayı Yerleştirme:
   Ekte bulunan "rstat_license.json" dosyasını oluşturduğunuz bu dizine taşıyın.
   Örn: ~/.config/rstat/license.json

3. Doğrulama:
   Lisans dosyasını yerleştirdikten sonra SPC veya Yeterlilik Analizi komutunu çalıştırabilirsiniz:
     rstat spc --chart xbar-r veriler.csv

Herhangi bir sorun yaşarsanız support@rstat.dev adresinden bizimle iletişime geçebilirsiniz.

Başarılar dileriz,
rstat Ekibi
`;
}
