-- Your SQL goes here
CREATE TABLE files (
    id TEXT PRIMARY KEY NOT NULL,
    mime TEXT NOT NULL,
    hash TEXT NOT NULL,
    owner_id TEXT NOT NULL
);
