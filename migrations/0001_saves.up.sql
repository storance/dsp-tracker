CREATE TABLE saves (
    id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE,
    version INTEGER NOT NULL,
    name VARCHAR NOT NULL,
    notes TEXT,
    mining_speed INTEGER NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT mining_speed_at_least_100 CHECK (mining_speed >= 100),
    CONSTRAINT positive_version CHECK (version >= 0),
    UNIQUE (name)
);