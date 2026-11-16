import unittest
from baseline import train, split, metrics, vector

class PipelineTests(unittest.TestCase):
    def fixture(self):
        # Synthetic fixtures ONLY to test software, never detector quality.
        return [dict(text=('human example ' if label==0 else 'machine sample ')+str(i), label=label, group=str(i), source='synthetic-test', license='test fixture') for i in range(30) for label in (0,1)]
    def test_groups_do_not_leak(self):
        parts=split(self.fixture()); groups=[{r['group'] for r in p} for p in parts]
        self.assertFalse(groups[0]&groups[1] or groups[0]&groups[2] or groups[1]&groups[2])
    def test_training_is_reproducible(self):
        a=train(self.fixture(),epochs=5); b=train(self.fixture(),epochs=5)
        self.assertEqual(a,b); self.assertEqual(a[0]['status'],'research_only'); self.assertFalse(a[0]['calibrated'])
    def test_metrics(self):
        m=metrics([0,0,1,1],[.1,.8,.2,.9],.5)
        self.assertEqual((m['tp'],m['fp'],m['tn'],m['fn']),(1,1,1,1))
        self.assertEqual(m['false_positive_rate'],.5)
    def test_oov(self): self.assertEqual(vector('unknown',{'word':0},[1]),{})
    def test_insufficient_groups(self):
        with self.assertRaises(ValueError): split(self.fixture()[:4])

if __name__=='__main__': unittest.main()
