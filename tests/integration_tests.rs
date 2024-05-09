use rgrep::*;

#[test]
fn test_funcionamiento_general() {
    let regex = "regex".to_string();
    let path_file = "res/test0.txt".to_string();

    let file_text = read_file(path_file).unwrap();
    let program_output = run_rgrep(regex, file_text).unwrap();

    assert_eq!(program_output.len(), 3);
    assert_eq!(program_output[0], "regex");
    assert_eq!(program_output[1], "no regex");
    assert_eq!(program_output[2], "multiple regex");
}
