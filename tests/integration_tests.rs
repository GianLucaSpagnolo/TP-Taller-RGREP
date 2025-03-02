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
fn tests_funcionamiento_pipe_simple() {
    let path_file = "res/test0.txt".to_string();
    let file_text = read_file(path_file).unwrap();

    let regex = "z|o".to_string();
    let program_output = run_rgrep(regex, file_text.clone()).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(
        program_output[0],
        "El archivo ha sido abierto correctamente!"
    );
    assert_eq!(program_output[1], "no regex");
}

#[test]
fn tests_funcionamiento_pipe_multiples_match() {
    let path_file = "res/test0.txt".to_string();
    let file_text = read_file(path_file).unwrap();

    let regex = "z|o|regex".to_string();
    let program_output = run_rgrep(regex, file_text.clone()).unwrap();

    println!("{:?}", program_output);
    assert_eq!(program_output.len(), 4);
    assert_eq!(
        program_output[0],
        "El archivo ha sido abierto correctamente!"
    );
    assert_eq!(program_output[1], "regex");
    assert_eq!(program_output[2], "no regex");
    assert_eq!(program_output[3], "multiple regex");
}

#[test]
fn tests_funcionamiento_pipe_start_and_end() {
    let path_file = "res/test0.txt".to_string();
    let file_text = read_file(path_file).unwrap();

    let regex = "|a|regex|".to_string();
    let program_output = run_rgrep(regex, file_text.clone()).unwrap();

    assert_eq!(program_output.len(), 5);
    assert_eq!(
        program_output[0],
        "El archivo ha sido abierto correctamente!"
    );
    assert_eq!(program_output[1], "Segunda linea");
    assert_eq!(program_output[2], "regex");
    assert_eq!(program_output[3], "no regex");
    assert_eq!(program_output[4], "multiple regex");
}

#[test]
fn tests_funcionamiento_backslash_y_pipe() {
    let path_file = "res/test0.txt".to_string();
    let file_text = read_file(path_file).unwrap();

    let regex = "z|a|e\\|o".to_string();
    let program_output = run_rgrep(regex, file_text.clone()).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(
        program_output[0],
        "El archivo ha sido abierto correctamente!"
    );
    assert_eq!(program_output[1], "Segunda linea");
}

#[test]
fn tests_funcionamiento_backslash_on_file() {
    let path_file = "res/test2.txt".to_string();
    let file_text = read_file(path_file).unwrap();

    let regex = "z|a|e\\|o".to_string();
    let program_output = run_rgrep(regex, file_text.clone()).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(program_output[0], "aaa");
    assert_eq!(program_output[1], "ee|oo");
}

#[test]
fn tests_funcionamiento_backslash_at_the_end() {
    let path_file = "res/test2.txt".to_string();
    let file_text = read_file(path_file).unwrap();

    let regex = "z|q\\|".to_string();
    let program_output = run_rgrep(regex, file_text.clone()).unwrap();

    assert_eq!(program_output.len(), 1);
    assert_eq!(program_output[0], "qqqq|");
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

#[test]
fn test_correcciones_entrega_4() {
    let regex = "^start|end$".to_string();
    let text = "start middle end\nstart with start\nend with end\nonly this line".to_string();

    let program_output = run_rgrep(regex, text).unwrap();

    assert_eq!(program_output.len(), 3);
    assert_eq!(program_output[0], "start middle end");
    assert_eq!(program_output[1], "start with start");
    assert_eq!(program_output[2], "end with end");
}

#[test]
fn test_correcciones_entrega_5() {
    let regex = "end$".to_string();
    let text = "start middle end\nstart with whatever but end not\nend with end\nonly this line"
        .to_string();

    let program_output = run_rgrep(regex, text).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(program_output[0], "start middle end");
    assert_eq!(program_output[1], "end with end");
}

#[test]
fn test_correcciones_entrega_6() {
    let regex = "[[:punct:]]".to_string();
    let text = "abc!123\nAB-12\n123\nabc-123".to_string();

    let program_output = run_rgrep(regex, text).unwrap();

    assert_eq!(program_output.len(), 3);
    assert_eq!(program_output[0], "abc!123");
    assert_eq!(program_output[1], "AB-12");
    assert_eq!(program_output[2], "abc-123");
}

#[test]
fn test_correcciones_entrega_7() {
    let regex = "[[:lower:]]".to_string();
    let text = "abc123\nAB12\n123\nabc-123".to_string();

    let program_output = run_rgrep(regex, text).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(program_output[0], "abc123");
    assert_eq!(program_output[1], "abc-123");
}
