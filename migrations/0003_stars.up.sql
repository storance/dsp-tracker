CREATE TYPE spectral_class AS ENUM (
    'class_a',
    'class_b',
    'class_f',
    'class_g',
    'class_k',
    'class_m',
    'class_o',
    'red_giant',
    'yellow_giant',
    'white_giant',
    'blue_giant',
    'white_dwarf',
    'black_hole',
    'neutron'
);

CREATE TABLE stars (
    id UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE,
    version INTEGER NOT NULL,
    solar_system_id UUID NOT NULL REFERENCES solar_systems(id),
    spectral_class spectral_class NOT NULL,
    luminosity REAL NOT NULL,
    radius REAL NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT positive_version CHECK (version >= 0),
    CONSTRAINT positive_luminosity CHECK (luminosity > 0.0),
    CONSTRAINT positive_radius CHECK (version > 0.0),
    UNIQUE (solar_system_id)
);