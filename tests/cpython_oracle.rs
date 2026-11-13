#![cfg(brz_cpython_set)]

use brz_collections::BrzHashSet;
use std::process::Command;

#[test]
#[ignore = "requires a 64-bit CPython 3.12+ interpreter"]
fn agrees_with_cpython() {
    let python = std::env::var_os("BRZ_TEST_CPYTHON").unwrap_or_else(|| "python3".into());
    let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cpython_oracle.py");
    let output = Command::new(python).arg(script).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cases = String::from_utf8(output.stdout).unwrap();
    let mut lines = cases.lines();
    for case in 0..2000 {
        let mut next_line = || {
            lines
                .next()
                .expect("oracle case is incomplete")
                .split_whitespace()
                .map(|value| value.parse::<i64>().unwrap())
                .collect::<Vec<_>>()
        };
        let mut set: BrzHashSet<_> = next_line().into_iter().collect();
        let other: BrzHashSet<_> = next_line().into_iter().collect();
        assert_eq!(
            set.iter().copied().collect::<Vec<_>>(),
            next_line(),
            "initial order in case {case}"
        );
        for value in next_line() {
            set.remove(&value);
        }
        set.extend(next_line());
        let expected = next_line();
        let expected_difference = next_line();
        assert_eq!(set.iter().copied().collect::<Vec<_>>(), expected, "{case}");
        assert_eq!(
            (&set - &other).into_iter().collect::<Vec<_>>(),
            expected_difference,
            "{case}"
        );
    }
    assert!(lines.next().is_none());
}
