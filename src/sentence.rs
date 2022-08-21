use crate::score::Score;
use chrono::prelude::*;
use chrono::Duration;

const ONE_DAY_IN_MINS: u64 = 24 * 60;
const SIX_DAYS_IN_MINS: u64 = ONE_DAY_IN_MINS * 6;

#[derive(Debug)]
pub struct Sentence {
    // TODO make fields private
    pub id: Option<usize>,

    pub text: String,
    pub created_at: DateTime<Utc>,

    pub last_answered_at: Option<DateTime<Utc>>,
    pub due_at: DateTime<Utc>,
    pub ease: f32,
    pub interval_in_mins: Option<u64>,
    pub reps: usize,

    pub is_suspended: bool,
}

impl Sentence {
    pub fn schedule(&mut self, score: Score) {
        let ease_delta = match score {
            Score::Hard => -0.2,
            Score::Good => 0.0,
            Score::Easy => 0.2,
        };

        self.ease = (self.ease + ease_delta).clamp(1.3, 3.5);

        let updated_interval;
        if self.reps == 0 {
            updated_interval = ONE_DAY_IN_MINS;
        } else if self.reps == 1 {
            updated_interval = SIX_DAYS_IN_MINS;
        } else {
            if let Some(prev_interval) = self.interval_in_mins {
                updated_interval = (prev_interval as f32 * self.ease) as u64;
            } else {
                panic!("Interval is None when it shouldn't be");
            }
        }
        self.interval_in_mins = Some(updated_interval);

        self.reps += 1;

        self.last_answered_at = Some(Utc::now());
        self.due_at = Utc::now() + Duration::minutes(updated_interval.try_into().unwrap());
    }

    pub fn suspend(&mut self) {
        self.is_suspended = true;
    }
}
