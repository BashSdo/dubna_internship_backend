CREATE TABLE users (
    id             UUID PRIMARY KEY,
    name           TEXT NOT NULL,
    login          TEXT NOT NULL UNIQUE,
    password_hash  TEXT NOT NULL,
    role           INT2 NOT NULL CHECK (role >= 1 AND role <= 3)
);
COMMENT ON COLUMN users.role
        IS '1 - initiator, \
            2 - purchasing manager, \
            3 - accounting manager';

CREATE TABLE tickets (
    id                     UUID PRIMARY KEY,
    title                  TEXT NOT NULL,
    description            TEXT NOT NULL,
    status                 INT2 NOT NULL CHECK (status >= 1 AND status <= 5),
    count                  INT NOT NULL,
    price                  FLOAT8,
    initiator_id           UUID NOT NULL REFERENCES users(id)
                                         ON UPDATE RESTRICT
                                         ON DELETE RESTRICT,
    purchasing_manager_id  UUID REFERENCES users(id)
                                ON UPDATE RESTRICT
                                ON DELETE RESTRICT,
    accounting_manager_id  UUID REFERENCES users(id)
                                ON UPDATE RESTRICT
                                ON DELETE RESTRICT,
    created_at             TIMESTAMPTZ NOT NULL
);
COMMENT ON COLUMN tickets.status
        IS '1 - requested, \
            2 - cancelled, \
            3 - confirmed, \
            4 - denied, \
            5 - payment completed';
