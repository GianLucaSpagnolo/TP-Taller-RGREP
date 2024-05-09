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

#[test]
fn test_correcciones_entrega_1() {
    let regex = "ab.?d".to_string();
    let text = "abcd\nabcdd\nabd\nhola abcd chau\nabhhd".to_string();

    let program_output = run_rgrep(regex, text).unwrap();

    assert_eq!(program_output.len(), 4);
    assert_eq!(program_output[0], "abcd");
    assert_eq!(program_output[1], "abcdd");
    assert_eq!(program_output[2], "abd");
    assert_eq!(program_output[3], "hola abcd chau");
}

#[test]
fn test_correcciones_entrega_2() {
    let regex = "ab.d".to_string();
    let text = "abcd\nabcdd\nabccd\nhola abcd chau".to_string();

    let program_output = run_rgrep(regex, text).unwrap();

    assert_eq!(program_output.len(), 3);
    assert_eq!(program_output[0], "abcd");
    assert_eq!(program_output[1], "abcdd");
    assert_eq!(program_output[2], "hola abcd chau");
}

#[test]
fn test_correcciones_entrega_3() {
    let regex = "abc{3}d".to_string();
    let text = "abcd\nabcccd\nhola abcccd chau".to_string();

    let program_output = run_rgrep(regex, text).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(program_output[0], "abcccd");
    assert_eq!(program_output[1], "hola abcccd chau");
}
