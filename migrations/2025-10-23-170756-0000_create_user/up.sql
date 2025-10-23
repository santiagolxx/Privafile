-- Your SQL goes here
CREATE TABLE usuarios (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    password TEXT NOT NULL,
    b64_pubkey TEXT
);
