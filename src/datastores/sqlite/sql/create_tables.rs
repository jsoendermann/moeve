pub const CREATE_TABLES_SQL: &str = "
    BEGIN;

    CREATE TABLE IF NOT EXISTS sentences (
        id INTEGER PRIMARY KEY AUTOINCREMENT,

        text TEXT UNIQUE NOT NULL,

        created_at TEXT NOT NULL,

        last_answered_at TEXT,
        due_at TEXT NOT NULL,
        ease REAL NOT NULL,
        interval_in_mins INTEGER,
        reps INTEGER NOT NULL,
        is_suspended INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS bundles (
        id TEXT PRIMARY KEY,

        created_at TEXT NOT NULL,

        has_been_answered INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS bundle_elements (
        id INTEGER PRIMARY KEY AUTOINCREMENT,

        sentence_id INTEGER NOT NULL,
        bundle_id TEXT NOT NULL,
        FOREIGN KEY(sentence_id) REFERENCES sentences(id)
        FOREIGN KEY(bundle_id) REFERENCES bundles(id)
    );
    
    COMMIT;";
