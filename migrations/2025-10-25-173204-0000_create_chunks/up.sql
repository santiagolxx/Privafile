-- Tabla para almacenar información de chunks
CREATE TABLE chunks (
    id TEXT PRIMARY KEY NOT NULL,
    file_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    hash TEXT NOT NULL,
    size INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'uploading',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

CREATE INDEX idx_chunks_file_id ON chunks(file_id);
CREATE INDEX idx_chunks_file_chunk ON chunks(file_id, chunk_index);
CREATE UNIQUE INDEX idx_chunks_unique ON chunks(file_id, chunk_index);

-- Agregar columna de status a files
ALTER TABLE files ADD COLUMN status TEXT NOT NULL DEFAULT 'complete';
ALTER TABLE files ADD COLUMN total_size INTEGER;
ALTER TABLE files ADD COLUMN created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;
