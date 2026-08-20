use std::process::Command;

#[test]
fn command_help_works_before_or_after_the_command() {
    for arguments in [["help", "get-series"], ["get-series", "help"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_bitview-cli"))
            .args(arguments)
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();

        assert!(output.status.success());
        assert!(stdout.contains("Usage: bitview-cli [OPTIONS] get-series <series> <index>"));
        assert!(stdout.contains("Fetch data for a specific series at the given index."));
    }
}
