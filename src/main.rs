mod datastores;
mod score;
mod sentence;

use crate::datastores::{get_datastore, Datastore};
use crate::score::Score;
use anyhow::Result;
use chrono::prelude::*;
use clap::{Args, Parser, Subcommand};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Row, Table};
use std::fs::{read_to_string, write};
use std::io::{stdin, BufRead};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Add(AddConfig),
    List(ListConfig),
    GenerateBundle(GenerateBundleConfig),
    AnswerBundle(AnswerBundleConfig),
}

#[derive(Args, Debug)]
struct AddConfig {
    path: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct ListConfig {
    #[clap(short, long, default_value_t = 200)]
    table_width: u16,
}

#[derive(Args, Debug)]
struct GenerateBundleConfig {
    #[clap(short, long, default_value_t = 3)]
    number_of_sentences: usize,
}

#[derive(Args, Debug)]
struct AnswerBundleConfig {
    #[clap(short, long)]
    bundle_id: String,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let datastore = get_datastore()?;

    match args.command {
        Command::Add(config) => add(datastore, &config)?,
        Command::List(config) => list(datastore, &config)?,
        Command::GenerateBundle(config) => generate_bundle(datastore, &config)?,
        Command::AnswerBundle(config) => answer_bundle(datastore, &config)?,
    }

    // let mut stmt = con.prepare("SELECT * FROM sentences").unwrap();
    // let mut res = stmt.query([]).unwrap();
    // while let Some(row) = res.next().unwrap() {
    //     let id: u64 = row.get(0).unwrap();
    //     println!("{}", id);
    //     let text: String = row.get(1).unwrap();
    //     println!("{}", text);
    // }
    // for row in res {
    // }
    // stmt.query_map([], |row| {
    //     println!("{:#?}", row);
    //     OK(())
    // })

    Ok(())
}

fn add(mut datastore: impl Datastore, config: &AddConfig) -> Result<()> {
    let mut raw_lines: Vec<String>;

    if let Some(filepath) = &config.path {
        let lines = read_to_string(filepath)?;
        raw_lines = lines.split('\n').map(|s| s.to_string()).collect();
    } else {
        println!("Enter sentences separated by newlines, exit with ctrl-d:");
        raw_lines = Vec::new();
        let stdin = stdin();
        let mut lines = stdin.lock().lines();

        for line in lines {
            if let Ok(str) = line {
                raw_lines.push(str)
            }
        }
    }

    let mut empty_lines = 0;
    let mut sentence_count = 0;

    for line in raw_lines.into_iter() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            empty_lines += 1;
            continue;
        }

        datastore.insert_sentence(trimmed)?;
        sentence_count += 1;
    }

    println!(
        "{} sentence(s) added, {} line(s) skipped",
        sentence_count, empty_lines
    );

    Ok(())
}

fn list(mut datastore: impl Datastore, config: &ListConfig) -> Result<()> {
    let sentences = datastore.get_sentences()?;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(config.table_width)
        .set_header(vec![
            Cell::new("ID").add_attribute(Attribute::Bold),
            Cell::new("Text").add_attribute(Attribute::Bold),
            Cell::new("Last answered").add_attribute(Attribute::Bold),
            Cell::new("Due at").add_attribute(Attribute::Bold),
            Cell::new("Ease").add_attribute(Attribute::Bold),
            Cell::new("Interval in days").add_attribute(Attribute::Bold),
            Cell::new("Reps").add_attribute(Attribute::Bold),
            Cell::new("Suspended").add_attribute(Attribute::Bold),
        ]);

    for sentence in sentences {
        let last_answered = match sentence.last_answered_at {
            Some(t) => chrono_humanize::HumanTime::from(t - Utc::now()).to_string(),
            None => "Never".to_string(),
        };
        let row = Row::from(vec![
            Cell::new(sentence.id.unwrap()),
            Cell::new(sentence.text),
            Cell::new(last_answered).set_alignment(CellAlignment::Right),
            Cell::new(chrono_humanize::HumanTime::from(
                sentence.due_at - Utc::now(),
            ))
            .set_alignment(CellAlignment::Right),
            Cell::new(sentence.ease.to_string()).set_alignment(CellAlignment::Right),
            Cell::new(
                sentence
                    .interval_in_mins
                    .map_or("None".to_string(), |i| (i / 60 / 24).to_string()),
            )
            .set_alignment(CellAlignment::Right),
            Cell::new(sentence.reps),
            Cell::new(sentence.is_suspended),
        ]);
        table.add_row(row);
    }

    println!("{table}");

    Ok(())
}

fn generate_bundle(mut datastore: impl Datastore, config: &GenerateBundleConfig) -> Result<()> {
    // let tx = datastore.start_generate_bundle_transaction()?;

    let bundle_id = datastore.create_bundle()?;
    let mut sentences = Vec::new();

    let due = datastore.get_due_sentences()?;
    let new = datastore.get_new_sentences()?;

    println!("Created bundle {}", &bundle_id);
    for mut sentence in due {
        println!(
            r#"Due sentence "{}", [a]dd, [s]kip, sus[p]end:"#,
            sentence.text
        );
        let mut action = String::new();
        stdin().read_line(&mut action)?;
        match action.trim() {
            "a" => {
                datastore.add_sentence_to_bundle(&bundle_id, &sentence)?;
                sentences.push(sentence);
            }
            "s" => {}
            "p" => {
                sentence.suspend();
                datastore.update_sentence(&sentence)?;
            }
            _ => println!("Unrecognized command: {} skipping sentence...", action),
        }
    }

    for mut sentence in new.into_iter().take(10) {
        println!(
            r#"Due sentence "{}", [a]dd, [s]kip, sus[p]end:"#,
            sentence.text
        );
        let mut action = String::new();
        stdin().read_line(&mut action)?;
        match action.trim() {
            "a" => {
                datastore.add_sentence_to_bundle(&bundle_id, &sentence)?;
                sentences.push(sentence);
            }
            "s" => {}
            "p" => {
                sentence.suspend();
                datastore.update_sentence(&sentence)?;
            }
            _ => println!("Unrecognized command: {} skipping sentence...", action),
        }
    }

    // todo add way to exit bundle creation early and commit what has been added and option to abort

    write(
        format!("{}.txt", bundle_id),
        sentences
            .iter()
            .map(|s| s.text.clone())
            .collect::<Vec<_>>()
            .join("\n"),
    )?;

    println!("Created bundle with {} sentence(s)", sentences.len());

    for s in sentences {
        println!("{}", s.text);
    }

    Ok(())
}

fn answer_bundle(mut datastore: impl Datastore, config: &AnswerBundleConfig) -> Result<()> {
    let tx = datastore.start_answer_bundle_transaction()?;

    let sentences = tx.get_sentences_in_bundle(&config.bundle_id)?;

    for mut sentence in sentences {
        println!(r#"Sentence "{}", 1: Hard, 2: Good, 3: Easy"#, sentence.text);

        let mut score = String::new();
        stdin().read_line(&mut score)?;
        let s = match score.trim() {
            "1" => Score::Hard,
            "2" => Score::Good,
            "3" => Score::Easy,
            _ => panic!("Score isn't one of 1, 2 or 3"),
        };

        sentence.schedule(s);
        tx.update_sentence(&sentence)?;
        // datastore.update_sentence(sentence)?;
    }

    tx.mark_bundle_as_answered(&config.bundle_id)?;

    tx.commit()?;

    Ok(())
}

// Sentence
// id, text, created_at, last_answered_at, ease, interval

// Sentence in bundle
//

// Bundle
// id, created_at, answered_at,

/*
add
generate bundle
update sentences
*/
