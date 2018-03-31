#![feature(catch_expr)]
#![feature(crate_visibility_modifier)]
#![feature(match_default_bindings)]

mod ir;
mod parser;

use self::ir::*;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

fn main() {
    let parser = parser::InputParser::new();
    for input_file in env::args().skip(1) {
        let mut input_text = &mut String::new();
        let result: Result<(), Box<Error>> = do catch {
            let mut file = File::open(&input_file)?;
            file.read_to_string(input_text)?;
            let ir = parser.parse(input_text)?;

            let mut path = PathBuf::from(&input_file);
            let parent_path = match path.parent() {
                Some(p) => p.to_owned(),
                None => env::current_dir().unwrap(),
            };

            write_to(&parent_path.join("borrowRegion.facts"), |file| {
                for block in &ir.blocks {
                    for (index, statement) in block.statements.iter().enumerate() {
                        let point = format!("{}/{}", block.name, index);
                        for effect in &statement.effects {
                            if let Effect::Borrow { region } = effect {
                                write!(
                                    file,
                                    "\"{region}\"\t\"B@{borrow}\"\t\"{point}\"\n",
                                    region = region,
                                    borrow = point,
                                    point = point,
                                )?
                            }
                        }
                    }
                }
                Ok(())
            })?;

            write_to(&parent_path.join("cfgEdge.facts"), |file| {
                for block in &ir.blocks {
                    let mut prev_point = None;
                    for index in 0..block.statements.len() {
                        let point = format!("{}/{}", block.name, index);
                        if let Some(prev_point) = prev_point {
                            write!(
                                file,
                                "\"{prev_point}\"\t\"{point}\"\n",
                                prev_point = prev_point,
                                point = point,
                            )?;
                        }
                        prev_point = Some(point);
                    }

                    let term_point = format!("{}/{}", block.name, block.statements.len());
                    if let Some(prev_point) = prev_point {
                        write!(
                            file,
                            "\"{prev_point}\"\t\"{term_point}\"\n",
                            prev_point = prev_point,
                            term_point = term_point,
                        )?;
                    }

                    for goto in &block.goto {
                        write!(
                            file,
                            "\"{term_point}\"\t\"{goto}/0\"\n",
                            term_point = term_point,
                            goto = goto,
                        )?;
                    }
                }
                Ok(())
            })?;

            write_to(&parent_path.join("regionLiveAt.facts"), |file| {
                for block in &ir.blocks {
                    for (index, statement) in block.statements.iter().enumerate() {
                        let point = format!("{}/{}", block.name, index);
                        for effect in &statement.effects {
                            if let Effect::Live { region } = effect {
                                write!(
                                    file,
                                    "\"{region}\"\t\"{point}\"\n",
                                    region = region,
                                    point = point,
                                )?;
                            }
                        }
                    }
                }
                Ok(())
            })?;

            write_to(&parent_path.join("outlives.facts"), |file| {
                for block in &ir.blocks {
                    for (index, statement) in block.statements.iter().enumerate() {
                        let point = format!("{}/{}", block.name, index);
                        for effect in &statement.effects {
                            if let Effect::PreOutlives { a, b } = effect {
                                write!(
                                    file,
                                    "\"{a}\"\t\"{b}\"\t\"{point}\"\n",
                                    a = a,
                                    b = b,
                                    point = point,
                                )?;
                            }
                        }

                        let point = format!("{}/{}", block.name, index + 1);
                        for effect in &statement.effects {
                            if let Effect::PostOutlives { a, b } = effect {
                                write!(
                                    file,
                                    "\"{a}\"\t\"{b}\"\t\"{point}\"\n",
                                    a = a,
                                    b = b,
                                    point = point,
                                )?;
                            }
                        }
                    }
                }
                Ok(())
            })?;

            Ok(())
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
