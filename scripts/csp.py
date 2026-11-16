"""Authorize only Trunk's exact generated bootstrap script in Pages CSP."""
import base64
import hashlib
import os
import re
from pathlib import Path
root = Path(os.environ.get('TRUNK_STAGING_DIR', 'dist'))
html = (root / 'index.html').read_text()
hashes = ['\'sha256-' + base64.b64encode(hashlib.sha256(s.encode()).digest()).decode() + '\'' for s in re.findall(r'<script\b[^>]*>(.*?)</script>', html, re.S) if s.strip()]
headers = Path('assets/_headers').read_text().replace("script-src 'self'", "script-src 'self' " + ' '.join(hashes))
(root / '_headers').write_text(headers)
