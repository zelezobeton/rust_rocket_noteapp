CREATE TABLE IF NOT EXISTS note_table
(
    id          INTEGER PRIMARY KEY NOT NULL,
    created     INTEGER             NOT NULL,
    changed     INTEGER             NOT NULL,
    title       TEXT                NOT NULL,
    content     TEXT                NOT NULL
);