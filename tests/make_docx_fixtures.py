"""Generate synthetic OOXML fixtures outside the repository."""
from pathlib import Path
from zipfile import ZipFile, ZIP_STORED, ZIP_DEFLATED
from io import BytesIO
out=Path('/tmp/nusa-docx'); out.mkdir(exist_ok=True)
ct='''<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>'''
xml='''<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Tim &amp; organisasi</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Produktivitas</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>'''
def build(xml, padding=0, method=ZIP_STORED):
 b=BytesIO()
 with ZipFile(b,'w',compression=method) as z:
  z.writestr('[Content_Types].xml',ct)
  z.writestr('word/document.xml',xml)
  z.writestr('word/media/padding.bin',b'0'*padding)
 return b.getvalue()
(out/'valid.docx').write_bytes(build(xml))
(out/'boundary.docx').write_bytes(build(xml,10*1024*1024-len(build(xml))))
assert (out/'boundary.docx').stat().st_size==10*1024*1024
(out/'bomb.docx').write_bytes(build('a'*(4*1024*1024+1),method=ZIP_DEFLATED))
(out/'malformed.docx').write_bytes(build('<w:document>'))
