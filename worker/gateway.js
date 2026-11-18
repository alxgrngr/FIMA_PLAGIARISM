import app from './src.js';

const receiptPath = /^\/(receipt|verify)\/([0-9a-f]{64})$/;
const apiPaths = new Map([
    ['/api/login', 'POST'],
    ['/api/me', 'GET'],
    ['/api/logout', 'POST'],
    ['/api/receipts', 'POST'],
]);

function error(status, message) {
    return Response.json({ error: message }, {
        status,
        headers: { 'cache-control': 'no-store', 'x-content-type-options': 'nosniff' },
    });
}

export default {
    async fetch(request, env) {
        const url = new URL(request.url);
        const path = url.pathname;
        if (request.method === 'POST' && request.headers.get('origin') !== url.origin) {
            return error(403, 'Asal permintaan tidak diizinkan.');
        }
        const receipt = path.match(receiptPath);
        const supported = apiPaths.get(path) === request.method ||
            (receipt && request.method === 'GET');
        if (!supported) {
            if (/^\/(api|receipt|verify)\//.test(path)) {
                return error(404, 'Tidak ditemukan.');
            }
            return env.ASSETS.fetch(request);
        }
        const session = (request.headers.get('cookie') || '')
            .match(/(?:^|;\s*)__Host-session=([0-9a-f]{64})(?:;|$)/)?.[1];
        if (path === '/api/receipts') {
            if (!session) return error(401, 'Silakan masuk.');
            if (Number(request.headers.get('content-length')) > 10 * 1024 * 1024) {
                return error(413, 'File terlalu besar. Maksimal 10 MB.');
            }
        }
        // A fixed set of 16 objects spreads simultaneous uploads without creating
        // a permanent object for each NIM, file, or request.
        const shard = receipt?.[2][0] ?? session?.[0] ?? '0';
        try {
            return await env.RECEIPT_PROCESSOR.getByName(`receipt-v1-${shard}`).fetch(request);
        } catch {
            return error(503, 'Layanan sementara tidak tersedia. Coba lagi nanti.');
        }
    },
    scheduled: app.scheduled,
};

// No use of object storage. The existing application writes signed metadata to
// D1; document contents live only in memory while processing this request.
export class ReceiptProcessor {
    constructor(_context, env) {
        this.env = env;
    }

    async fetch(request) {
        try {
            return await app.fetch(request, this.env);
        } finally {
            if (request.body && !request.bodyUsed && !request.body.locked) {
                await request.body.cancel();
            }
        }
    }
}
