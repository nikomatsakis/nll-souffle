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

#[test]
fn carry_nest() -> Result<(), Box<dyn Error>> {
    run_souffle! {
        in("carry-nest") expecting (
            r#"borrowLiveAt
===============
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
fn problem_case_1() -> Result<(), Box<dyn Error>> {
    run_souffle! {
        in("problem-case-1") expecting (
            r#"borrowLiveAt
===============
"B0"	"B1/2"
===============
"#
        )
    }
}

#[test]
fn issue_47680() -> Result<(), Box<dyn Error>> {
    // Borrow is not live around loop.
    run_souffle! {
        in("issue-47680") expecting (
            r#"borrowLiveAt
===============
"B_x"	"B/1"
"B_x"	"C/0"
"B_x"	"B/2"
===============
"#
        )
    }
}
