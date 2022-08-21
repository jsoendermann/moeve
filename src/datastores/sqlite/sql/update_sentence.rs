pub const UPDATE_SENTENCE: &str = "
UPDATE sentences SET
            last_answered_at = :last_answered_at,
            due_at = :due_at,
            ease = :ease,
            interval_in_mins = :interval_in_mins,
            reps = :reps,
            is_suspended = :is_suspended

         WHERE id = :id;
";
