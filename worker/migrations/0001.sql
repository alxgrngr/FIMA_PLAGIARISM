CREATE TABLE participants(nim TEXT PRIMARY KEY,name TEXT NOT NULL,group_name TEXT NOT NULL,role TEXT NOT NULL CHECK(role IN ('student','test')),active INTEGER NOT NULL DEFAULT 1);
CREATE TABLE sessions(token_hash TEXT PRIMARY KEY,nim TEXT NOT NULL REFERENCES participants(nim),expires INTEGER NOT NULL);
CREATE INDEX sessions_expiry ON sessions(expires);
CREATE TABLE receipts(id TEXT PRIMARY KEY,nim TEXT NOT NULL REFERENCES participants(nim),payload TEXT NOT NULL,signature TEXT NOT NULL,created INTEGER NOT NULL);
CREATE INDEX receipt_owner ON receipts(nim,created);
CREATE TABLE limits(key TEXT PRIMARY KEY,count INTEGER NOT NULL,expires INTEGER NOT NULL);
CREATE INDEX limit_expiry ON limits(expires);
