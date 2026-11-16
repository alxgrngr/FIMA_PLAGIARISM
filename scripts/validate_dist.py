"""Fail closed if the deploy directory contains source HTML or missing assets."""
from pathlib import Path
import re

root = Path('dist')
html = (root / 'index.html').read_text()
assert 'data-trunk' not in html, 'Source HTML detected: run trunk build --release --locked'
assert '<script' in html and '.js' in html, 'Missing compiled JS bootstrap'
assert '.wasm' in html, 'Missing WASM bootstrap'
for asset in re.findall(r'''["'](/[^"'<>\s]+\.(?:js|wasm|css))["']''', html):
    assert (root / asset.lstrip('/')).is_file(), f'Missing compiled asset: {asset}'
assert (root / '_headers').is_file(), 'Missing security headers'
assert any(root.glob('*.wasm')), 'No compiled WASM found'
for p in root.rglob('*'):
    if p.is_file():
        assert p.stat().st_size <= 25 * 1024 * 1024, f'Oversized asset: {p}'
print('PASS: compiled bootstrap, referenced assets, WASM, headers and size limits')
