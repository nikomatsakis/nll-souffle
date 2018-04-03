#![cfg(test)]

use assert_cli::Assert;
use std::error::Error;

macro_rules! run_souffle {
    (in ($test_dir:expr) expecting ($expected:expr)) => {
        const TEST_DIR: &str = $test_dir;

        Assert::main_binary()
            .with_args(&[&format!("tests/{}/test.txt", TEST_DIR)])
            .succeeds()
            .execute()?;

        Assert::command(&["souffle"])
            .with_args(&["regions.dl", "-F", &format!("tests/{}/", TEST_DIR), "-D", "-"])
            .stdout()
            .contains($expected)
            .unwrap();

        Ok(())
    }
}

macro_rules! run_differential_dataflow {
    (in ($test_dir:expr) expecting ($expected:expr)) => {
        const TEST_DIR: &str = $test_dir;

        Assert::main_binary()
            .with_args(&["--execute", &format!("tests/{}/test.txt", TEST_DIR)])
            .stdout()
            .contains($expected)
            .unwrap();

        Ok(())
    }
}

#[test]
fn carry_nest() -> Result<(), Box<dyn Error>> {
    run_souffle! {
        in("carry-nest") expecting (
            r#"===============
"B_foo"	"B0/1"
"B_foo"	"B0/2"
"B_bar"	"B0/3"
"B_foo"	"B0/3"
"B_foo"	"B0/4"
"B_foo"	"B0/5"
===============
"#
        )
    }
}

#[test]
fn carry_nest_differential() -> Result<(), Box<dyn Error>> {
    run_differential_dataflow! {
        in("carry-nest") expecting (
            r#"vvv borrowLiveAt vvv
borrow B_foo live at B0/2
borrow B_foo live at B0/1
borrow B_foo live at B0/3
borrow B_foo live at B0/4
borrow B_foo live at B0/5
borrow B_bar live at B0/3
^^^ borrowLiveAt ^^^
"#
        )
    }
}

#[test]
fn problem_case_1() -> Result<(), Box<dyn Error>> {
    run_souffle! {
        in("problem-case-1") expecting (
            r#"===============
"B0"	"B1/2"
===============
"#
        )
    }
}

#[test]
fn problem_case_1_differential() -> Result<(), Box<dyn Error>> {
    run_differential_dataflow! {
        in("problem-case-1") expecting (
            r#"vvv borrowLiveAt vvv
borrow B0 live at B1/2
^^^ borrowLiveAt ^^^
"#
        )
    }
}

#[test]
fn issue_47680() -> Result<(), Box<dyn Error>> {
    // Borrow is not live around loop.
    run_souffle! {
        in("issue-47680") expecting (
            r#"===============
"B_x"	"B/1"
"B_x"	"C/0"
"B_x"	"B/2"
===============
"#
        )
    }
}

#[test]
fn issue_47680_differential() -> Result<(), Box<dyn Error>> {
    run_differential_dataflow! {
        in("issue-47680") expecting (
            r#"vvv borrowLiveAt vvv
borrow B_x live at B/1
borrow B_x live at B/2
borrow B_x live at C/0
^^^ borrowLiveAt ^^^
"#
        )
    }
}
