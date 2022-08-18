use assert_cmd::prelude::*;
use predicates::prelude::predicate;
use std::process::Command;

#[test]
fn test_full_program() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("contabile")?;

    cmd.arg("tests/test_data/test_full_program.csv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "client,available,held,total,locked",
        ))
        .stdout(predicate::str::contains("1,4.4994,0.0000,4.4994,false"))
        .stdout(predicate::str::contains("2,0.2246,0.0000,0.2246,true"))
        .stdout(predicate::str::contains("3,2.7659,10.9871,13.7530,false"));

    Ok(())
}

#[test]
fn test_decimal_precision() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("contabile")?;

    cmd.arg("tests/test_data/test_decimal_precision.csv");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "client,available,held,total,locked",
        ))
        .stdout(predicate::str::contains("1,5.5749,0.0000,5.5749,false"));

    Ok(())
}
