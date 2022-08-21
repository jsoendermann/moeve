mod sql;

use self::sql::{
    CREATE_TABLES_SQL, GET_ALL_SENTENCES, GET_DUE_SENTENCES, GET_NEW_SENTENCES,
    GET_SENTENCES_IN_BUNDLE, INSERT_SENTENCE, UPDATE_SENTENCE,
};
use super::datastore::AnswerBundleTransaction;
use super::Datastore;
use crate::sentence::Sentence;
use anyhow::Result;
use chrono::prelude::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rusqlite::{named_params, Connection, Params, Transaction};
use std::path::Path;

pub struct SqliteAnswerBundleTransaction<'a> {
    tx: Transaction<'a>,
}

fn get_sentences(con: &Connection, sql: &str, params: impl Params) -> Result<Vec<Sentence>> {
    let mut stmt = con.prepare(sql)?;

    let sentences = stmt
        .query_map(params, |row| {
            let created_at_string: String = row.get(2)?;
            let last_answered_at_string: Option<String> = row.get(3)?;
            let due_at_string: String = row.get(4)?;

            Ok(Sentence {
                id: Some(row.get(0)?),
                text: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(&created_at_string)
                    .unwrap()
                    .into(),

                last_answered_at: last_answered_at_string
                    .map(|l| DateTime::parse_from_rfc3339(&l).unwrap().into()),
                due_at: DateTime::parse_from_rfc3339(&due_at_string).unwrap().into(),
                ease: row.get(5)?,
                interval_in_mins: row.get(6)?,

                reps: row.get(7)?,
                is_suspended: row.get(8)?,
            })
        })?
        .map(|r| r.expect("Something went wrong reading a sentence from the db"))
        .collect();

    Ok(sentences)
}

fn update_sentence(con: &Connection, sentence: &Sentence) -> Result<()> {
    con.execute(
        UPDATE_SENTENCE,
        named_params! {
            ":id": sentence.id,
            ":last_answered_at": sentence.last_answered_at.map(|d| d.to_rfc3339()),
            ":due_at": sentence.due_at.to_rfc3339(),
            ":ease": sentence.ease,
            ":interval_in_mins": sentence.interval_in_mins,
            ":reps": sentence.reps,
            ":is_suspended": sentence.is_suspended
        },
    )
    .unwrap();

    Ok(())
}

impl<'a> AnswerBundleTransaction for SqliteAnswerBundleTransaction<'a> {
    fn get_sentences_in_bundle(&self, bundle_id: &str) -> Result<Vec<Sentence>> {
        get_sentences(&self.tx, GET_SENTENCES_IN_BUNDLE, [bundle_id])
    }

    fn mark_bundle_as_answered(&self, bundle_id: &str) -> Result<()> {
        self.tx
            .execute(
                "UPDATE bundles SET has_been_answered = TRUE WHERE id = :id",
                named_params! {":id": bundle_id},
            )
            .unwrap();

        Ok(())
    }

    fn update_sentence(&self, sentence: &Sentence) -> Result<()> {
        update_sentence(&self.tx, sentence)
    }

    fn commit(self: Box<Self>) -> Result<()> {
        self.tx.commit().unwrap();
        Ok(())
    }
}

pub struct SqliteDatastore {
    con: Connection,
}

impl SqliteDatastore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let con = Connection::open(path)?;
        con.execute_batch(&CREATE_TABLES_SQL)?;

        Ok(SqliteDatastore { con })
    }
}

impl Datastore for SqliteDatastore {
    fn create_bundle(&self) -> Result<String> {
        let now = Utc::now();
        let mut rng = thread_rng();
        let chars: String = (0..5).map(|_| rng.sample(Alphanumeric) as char).collect();

        let bundle_id = format!("{}-{}", now.format("%Y-%m-%d"), chars);

        self.con.execute(
            "INSERT INTO bundles (
        id,
        created_at,
        has_been_answered
    ) VALUES (:id, :created_at, FALSE)",
            named_params! {
                ":id": bundle_id,
                ":created_at": Utc::now().to_rfc3339(),
            },
        )?;

        Ok(bundle_id)
    }

    fn add_sentence_to_bundle(&self, bundle_id: &str, sentence: &Sentence) -> Result<()> {
        self.con.execute("INSERT INTO bundle_elements (sentence_id, bundle_id) VALUES (:sentence_id, :bundle_id)",named_params! {":sentence_id": sentence.id, ":bundle_id": bundle_id})?;
        Ok(())
    }

    fn update_sentence(&self, sentence: &Sentence) -> Result<()> {
        update_sentence(&self.con, sentence)
    }

    fn get_due_sentences(&self) -> Result<Vec<Sentence>> {
        get_sentences(&self.con, GET_DUE_SENTENCES, [])
    }

    fn get_new_sentences(&self) -> Result<Vec<Sentence>> {
        // todo make number of new sentences configurable
        get_sentences(&self.con, GET_NEW_SENTENCES, [10])
    }

    fn insert_sentence(&self, text: &str) -> Result<()> {
        let res = self
            .con
            .execute(
                INSERT_SENTENCE,
                named_params! {
                    ":text": &text,
                    ":created_at": Utc::now().to_rfc3339(),
                    ":due_at": Utc::now().to_rfc3339(),
                },
            )
            .unwrap();

        Ok(())
    }

    fn start_answer_bundle_transaction<'a>(
        &'a mut self,
    ) -> Result<Box<dyn AnswerBundleTransaction + 'a>> {
        let tx = self.con.transaction()?;
        Ok(Box::new(SqliteAnswerBundleTransaction { tx }))
    }

    // todo rename to get_all_sentences
    fn get_sentences(&self) -> Result<Vec<Sentence>> {
        get_sentences(&self.con, GET_ALL_SENTENCES, [])
    }
}
