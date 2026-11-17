-- Synthetic accounts for an isolated LOCAL test DB only. Not a production roster.
INSERT OR REPLACE INTO participants(nim,name,group_name,role) VALUES
('999001','Peserta Uji A','U1','student'),
('999002','Peserta Uji B','U2','student'),
('999003','Penguji Sistem','T1','test');
