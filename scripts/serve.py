"""Local production-artifact verification with Cloudflare-style headers."""
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory='dist', **kwargs)
    def end_headers(self):
        for line in Path('dist/_headers').read_text().splitlines():
            if line.startswith('  ') and ':' in line:
                key, value = line.strip().split(':', 1)
                self.send_header(key, value.strip())
        super().end_headers()
ThreadingHTTPServer(('127.0.0.1', 8081), Handler).serve_forever()
