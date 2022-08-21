use crate::score::Score;
use crate::sentence::Sentence;
use anyhow::Result;
use chrono::prelude::*;
use clap::{Args, Parser, Subcommand};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rusqlite::{named_params, Connection};
use std::fs::read_to_string;
use std::io::{stdin, BufRead};
use std::path::{Path, PathBuf};

pub trait Datastore {
    fn insert_sentence(&self, text: &str) -> Result<()>;

    fn create_bundle(&self) -> Result<String>;

    fn get_due_sentences(&self) -> Result<Vec<Sentence>>;
    fn get_new_sentences(&self) -> Result<Vec<Sentence>>;

    fn add_sentence_to_bundle(&self, bundle_id: &str, sentence: &Sentence) -> Result<()>;

    fn update_sentence(&self, sentence: &Sentence) -> Result<()>;

    fn start_answer_bundle_transaction<'a>(
        &'a mut self,
    ) -> Result<Box<dyn AnswerBundleTransaction + 'a>>;

    fn get_sentences(&self) -> Result<Vec<Sentence>>;
}

pub trait AnswerBundleTransaction {
    fn get_sentences_in_bundle(&self, bundle_id: &str) -> Result<Vec<Sentence>>;
    fn update_sentence(&self, sentence: &Sentence) -> Result<()>;
    fn mark_bundle_as_answered(&self, bundle_id: &str) -> Result<()>;
    fn commit(self: Box<Self>) -> Result<()>;
}
