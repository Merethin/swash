CREATE TABLE regions_dump (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    canon_name TEXT NOT NULL UNIQUE,
    factbook TEXT,
    numnations BIGINT,
    totalnations BIGINT,
    updateorder BIGINT,
    nations TEXT[],
    delegate TEXT,
    delegatevotes BIGINT,
    delegateauth TEXT,
    frontier BIGINT,
    founder TEXT,
    governor TEXT,
    officers JSONB,
    power TEXT,
    magnetism DOUBLE PRECISION,
    flag_url TEXT,
    banner_id TEXT,
    banner_url TEXT,
    embassies TEXT[],
    lastupdate BIGINT,
    lastmajorupdate BIGINT,
    lastminorupdate BIGINT
);

CREATE UNIQUE INDEX idx_region_canon_name ON regions_dump(canon_name);