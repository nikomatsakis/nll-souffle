#![feature(catch_expr)]
#![feature(crate_in_paths)]
#![feature(crate_visibility_modifier)]
#![feature(dyn_trait)]
#![feature(in_band_lifetimes)]
#![feature(match_default_bindings)]
#![feature(termination_trait_test)]

#[cfg(test)]
extern crate assert_cli;

#[macro_use]
extern crate abomonation_derive;
extern crate abomonation;
extern crate differential_dataflow;
extern crate timely;

mod ir;
mod lower;
mod parser;
mod solve;
mod tests;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

fn main() {
    let parser = parser::InputParser::new();

    let mut args = env::args().skip(1).peekable();

    let mut execute_mode = false;
    if args.peek().map_or(false, |arg| arg == "--execute") {
        args.next();
        execute_mode = true;
    }

    for input_file in args {
        let mut input_text = &mut String::new();
        let result: Result<(), Box<Error>> = do catch {
            let mut file = File::open(&input_file)?;
            file.read_to_string(input_text)?;
            let ir = parser.parse(input_text)?;

            if execute_mode {
                solve_facts(&ir)
            } else {
                dump_facts(&input_file, &ir)
            }
        };

        match result {
            Ok(()) => {}
            Err(err) => {
                eprintln!("`{}`: {}", input_file, err);
            }
        }
    }
}

fn write_to(
    path: &Path,
    output: impl FnOnce(&mut File) -> Result<(), Box<Error>>,
) -> Result<(), Box<Error>> {
    let mut file = File::create(path)?;
    output(&mut file)?;
    Ok(())
}

fn solve_facts(ir: &ir::Input) -> Result<(), Box<Error>> {
    Ok(solve::region_computation(ir))
}

fn dump_facts(input_file: &String, ir: &ir::Input) -> Result<(), Box<Error>> {
    let path = PathBuf::from(input_file);
    let parent_path = match path.parent() {
        Some(p) => p.to_owned(),
        None => env::current_dir().unwrap(),
    };

    write_to(&parent_path.join("borrowRegion.facts"), |file| {
        ir.for_each_borrow_region_fact(|region, borrow, point| {
            write!(
                file,
                "\"{region}\"\t\"{borrow}\"\t\"{point}\"\n",
                region = region,
                borrow = borrow,
                point = point,
            )
        })?;
        Ok(())
    })?;

    write_to(&parent_path.join("nextStatement.facts"), |file| {
        ir.for_each_next_statement_fact(|prev_point, point| {
            write!(
                file,
                "\"{prev_point}\"\t\"{point}\"\n",
                prev_point = prev_point,
                point = point,
            )
        })?;
        Ok(())
    })?;

    write_to(&parent_path.join("goto.facts"), |file| {
        ir.for_each_goto_fact(|prev_point, point| {
            write!(
                file,
                "\"{prev_point}\"\t\"{point}\"\n",
                prev_point = prev_point,
                point = point,
            )
        })?;
        Ok(())
    })?;

    write_to(
        &parent_path.join("regionLiveOnEntryToStatement.facts"),
        |file| {
            ir.for_each_region_live_on_entry_fact(|region, point| {
                write!(
                    file,
                    "\"{region}\"\t\"{point}\"\n",
                    region = region,
                    point = point,
                )
            })?;
            Ok(())
        },
    )?;

    write_to(&parent_path.join("killed.facts"), |file| {
        ir.for_each_killed_fact(|borrow, point| {
            write!(
                file,
                "\"{borrow}\"\t\"{point}\"\n",
                borrow = borrow,
                point = point,
            )
        })?;
        Ok(())
    })?;

    write_to(&parent_path.join("outlives.facts"), |file| {
        ir.for_each_outlives_fact(|p, a, b, q| {
            write!(
                file,
                "\"{p}\"\t\"{a}\"\t\"{b}\"\t\"{q}\"\n",
                p = p,
                a = a,
                b = b,
                q = q,
            )
        })?;
        Ok(())
    })?;

    Ok(())
}
