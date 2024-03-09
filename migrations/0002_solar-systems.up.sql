CREATE TABLE solar_systems (
    id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE,
    version INTEGER NOT NULL,
    save_id UUID NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    notes TEXT,
    PRIMARY KEY (id),
    CONSTRAINT positive_version CHECK (version >= 0),
    UNIQUE (save_id, name)
);