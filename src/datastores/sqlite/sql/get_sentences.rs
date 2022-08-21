pub const GET_DUE_SENTENCES: &str = "
SELECT
            id, -- 0
            text,  -- 1
            created_at, -- 2

            last_answered_at, -- 3
            due_at, -- 4
            ease, -- 5
            interval_in_mins, -- 6
            reps, -- 7
            is_suspended -- 8

            FROM sentences WHERE due_at > date() AND reps > 0 AND is_suspended = FALSE
            ORDER BY due_at ASC;";

pub const GET_NEW_SENTENCES: &str = "
SELECT
            id, -- 0
            text,  -- 1
            created_at, -- 2

            last_answered_at, -- 3
            due_at, -- 4
            ease, -- 5
            interval_in_mins, -- 6
            reps, -- 7
            is_suspended -- 8

            FROM sentences WHERE reps == 0 AND is_suspended = FALSE
            ORDER BY created_at ASC
            LIMIT ?;
";

pub const GET_ALL_SENTENCES: &str = "
SELECT
            sentences.id, -- 0
            text,  -- 1
            created_at, -- 2

            last_answered_at, -- 3
            due_at, -- 4
            ease, -- 5
            interval_in_mins, -- 6
            reps, -- 7
            is_suspended -- 8

            FROM sentences
            ORDER BY last_answered_at ASC;
";

pub const GET_SENTENCES_IN_BUNDLE: &str = "
  SELECT 
    sentences.id, -- 0
    text,  -- 1
    created_at, -- 2

    last_answered_at, -- 3
    due_at, -- 4
    ease, -- 5
    interval_in_mins, -- 6
    reps, -- 7
    is_suspended -- 8
  FROM bundle_elements
  JOIN sentences 
  ON bundle_elements.sentence_id = sentences.id
  WHERE bundle_elements.bundle_id = ? AND sentences.is_suspended = FALSE;";
