"""Synthetic spreadsheet tests: no real participants or workbook content."""
import subprocess,tempfile,unittest
from pathlib import Path
import openpyxl
class RosterTests(unittest.TestCase):
 def make(self,root,duplicate=False):
  w=openpyxl.Workbook();w.remove(w.active)
  for i in range(7):w.create_sheet('sheet'+str(i+1))
  s=w.worksheets[5];s.append(['NO','NIM','NAMA','KELOMPOK']);s.append([1,'000001','Synthetic A','G1']);s.append([2,'000002','Synthetic B',None]);s.merge_cells('D2:D3')
  t=w.worksheets[6];t.append(['NO','NIM','NAMA','KELOMPOK']);t.append([1,'000001' if duplicate else '999999','Test Account','TEST'])
  p=root/'input.xlsx';w.save(p);return p
 def test_merge_and_leading_zero(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);p=self.make(root);out=root/'private.sql'
   r=subprocess.run(['python3','scripts/import_roster.py',str(p),'--out',str(out)],capture_output=True,text=True)
   self.assertEqual(r.returncode,0,r.stderr);sql=out.read_text();self.assertIn("'000001'",sql);self.assertIn("'000002','Synthetic B','G1','student'",sql);self.assertIn("'999999','Test Account','TEST','test'",sql);self.assertNotIn('Synthetic A',r.stdout)
 def test_cross_role_duplicate_rejected(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);p=self.make(root,True);out=root/'private.sql'
   r=subprocess.run(['python3','scripts/import_roster.py',str(p),'--out',str(out)],capture_output=True,text=True)
   self.assertNotEqual(r.returncode,0);self.assertFalse(out.exists());self.assertNotIn('Synthetic A',r.stderr)
if __name__=='__main__':unittest.main()
