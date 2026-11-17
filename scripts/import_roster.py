"""Generate PRIVATE D1 seed from sheets 6/7. Never logs names or NIMs."""
import argparse,json,os,re
from pathlib import Path
import openpyxl
p=argparse.ArgumentParser();p.add_argument('xlsx');p.add_argument('--out',default='private/roster.sql');a=p.parse_args()
w=openpyxl.load_workbook(a.xlsx,data_only=True)
if len(w.worksheets)<7: raise ValueError('Workbook requires 7 sheets')
records=[];seen=set();counts={}
for index,role in [(5,'student'),(6,'test')]:
 s=w.worksheets[index];header=None
 def value(row,col):
  c=s.cell(row,col)
  if c.value is not None:return c.value
  for m in s.merged_cells.ranges:
   if c.coordinate in m:return s.cell(m.min_row,m.min_col).value
  return None
 for row in range(1,s.max_row+1):
  cells={str(c.value).strip().upper():c.column for c in s[row] if c.value is not None}
  if all(k in cells for k in ('NIM','NAMA','KELOMPOK')):header=cells;continue
  if not header:continue
  nim=value(row,header['NIM'])
  if nim is None:continue
  if isinstance(nim,float):
   if not nim.is_integer():raise ValueError('Non-integer NIM')
   nim=int(nim)
  nim=str(nim).strip();name=str(value(row,header['NAMA']) or '').strip();group=str(value(row,header['KELOMPOK']) or '').strip()
  if not re.fullmatch(r'\d{6,25}',nim) or not name or not group:raise ValueError(f'Invalid roster row at sheet {index+1} row {row}')
  if nim in seen:raise ValueError('Duplicate NIM; review before import')
  seen.add(nim);records.append((nim,name,group,role));counts[role]=counts.get(role,0)+1
 if not header:raise ValueError('Missing roster header')
q=lambda s:"'"+s.replace("'","''")+"'"
lines=['-- PRIVATE student data. Never commit or put in dist.']
for row in records:
 lines.append('INSERT INTO participants(nim,name,group_name,role) VALUES('+','.join(map(q,row))+') ON CONFLICT(nim) DO UPDATE SET name=excluded.name,group_name=excluded.group_name,role=excluded.role;')
output=Path(a.out);output.parent.mkdir(parents=True,exist_ok=True);output.write_text('\n'.join(lines)+'\n');os.chmod(output,0o600)
print(json.dumps({'counts':counts,'output':str(output),'sheet_indices':[6,7]}))
