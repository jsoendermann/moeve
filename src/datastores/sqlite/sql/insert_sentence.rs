pub const INSERT_SENTENCE: &str = "
INSERT INTO sentences (
  text,
  created_at,
  
  last_answered_at,
  due_at,
  ease,
  interval_in_mins,
  reps,
  is_suspended
) VALUES (:text, :created_at, NULL, :due_at, 2.5, NULL, 0, FALSE);
";
