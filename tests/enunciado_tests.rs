use rgrep::*;

#[test]
fn test_enunciado_1() {
    let regex = "ab.cd".to_string();
    let lines = "abcd\nabecd\nabccd".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(program_output[0], "abecd");
    assert_eq!(program_output[1], "abccd");
}

#[test]
fn test_enunciado_2() {
    let regex = "ab.*cd".to_string();
    let lines = "abcd\nabecd\nabccd\nabeeeeeecd".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 4);
    assert_eq!(program_output[0], "abcd");
    assert_eq!(program_output[1], "abecd");
    assert_eq!(program_output[2], "abccd");
    assert_eq!(program_output[3], "abeeeeeecd");
}

#[test]
fn test_enunciado_3() {
    let regex = "a[bc]d".to_string();
    let lines = "abcd\nabd\nacd\nad\nabbbcccd".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(program_output[0], "abd");
    assert_eq!(program_output[1], "acd");
}

#[test]
fn test_enunciado_4() {
    let regex = "ab{2,4}cd".to_string();
    let lines = "abcd\nabbcd\nabbbcd\naeecd\nabbbbcd\nabbbbbcd\nacd".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 3);
    assert_eq!(program_output[0], "abbcd");
    assert_eq!(program_output[1], "abbbcd");
    assert_eq!(program_output[2], "abbbbcd");
}

#[test]
fn test_enunciado_5() {
    let regex = "abc|de+f".to_string();
    let lines = "abcd\nabbcd\nrabcr\ndfac\nadef\nzadeeefj\nabcdef".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 5);
    assert_eq!(program_output[0], "abcd");
    assert_eq!(program_output[1], "rabcr");
    assert_eq!(program_output[2], "adef");
    assert_eq!(program_output[3], "zadeeefj");
    assert_eq!(program_output[4], "abcdef");
}

#[test]
fn test_enunciado_6() {
    let regex = "la [aeiou] es una vocal".to_string();
    let lines = "la a es una vocal\nla e es una vocal\nla i es una vocal\nla o es una vocal\nla u es una vocal\nla r es una vocal\nla   es una vocal\nla % es una vocal\nla 4 es una vocal".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 5);
    assert_eq!(program_output[0], "la a es una vocal");
    assert_eq!(program_output[1], "la e es una vocal");
    assert_eq!(program_output[2], "la i es una vocal");
    assert_eq!(program_output[3], "la o es una vocal");
    assert_eq!(program_output[4], "la u es una vocal");
}

#[test]
fn test_enunciado_7() {
    let regex = "la [^aeiou] no es una vocal".to_string();
    let lines = "la a no es una vocal\nla e no es una vocal\nla i no es una vocal\nla o no es una vocal\nla u no es una vocal\nla z no es una vocal\nla   no es una vocal\nla ! no es una vocal\nla 8 no es una vocal".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 4);
    assert_eq!(program_output[0], "la z no es una vocal");
    assert_eq!(program_output[1], "la   no es una vocal");
    assert_eq!(program_output[2], "la ! no es una vocal");
    assert_eq!(program_output[3], "la 8 no es una vocal");
}

#[test]
fn test_enunciado_8() {
    let regex = "hola [[:alpha:]]+".to_string();
    let lines = "hola mundo\nhola 123\nhola\nhola 123 mundo\n123 hola mundo\nhola !".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(program_output[0], "hola mundo");
    assert_eq!(program_output[1], "123 hola mundo");
}

#[test]
fn test_enunciado_9() {
    let regex = "[[:digit:]] es un numero".to_string();
    let lines = "1 es un numero\n2 es un numero\n3 es un numero\nel 4 es un numero\n5 es un numero!\nel 6 es un numero tambien\n7 es un numero\n8 es un numero\n9 es un numero\n0 es un numero\na es un numero\n! es un numero\n  es un numero".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 10);
    assert_eq!(program_output[0], "1 es un numero");
    assert_eq!(program_output[1], "2 es un numero");
    assert_eq!(program_output[2], "3 es un numero");
    assert_eq!(program_output[3], "el 4 es un numero");
    assert_eq!(program_output[4], "5 es un numero!");
    assert_eq!(program_output[5], "el 6 es un numero tambien");
    assert_eq!(program_output[6], "7 es un numero");
    assert_eq!(program_output[7], "8 es un numero");
    assert_eq!(program_output[8], "9 es un numero");
    assert_eq!(program_output[9], "0 es un numero");
}

#[test]
fn test_enunciado_10() {
    let regex = "el caracter [[:alnum:]] no es un simbolo".to_string();
    let lines = "el caracter a no es un simbolo\nel caracter 1 no es un simbolo\nel caracter ! no es un simbolo\nel caracter   no es un simbolo\nefectivamente el caracter P no es un simbolo!".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 3);
    assert_eq!(program_output[0], "el caracter a no es un simbolo");
    assert_eq!(program_output[1], "el caracter 1 no es un simbolo");
    assert_eq!(
        program_output[2],
        "efectivamente el caracter P no es un simbolo!"
    );
}

#[test]
fn test_enunciado_11() {
    let regex = "hola[[:space:]]mundo".to_string();
    let lines = "hola mundo\nholamundo\nhey hola mundo !\nHola mundo\n(hola mundo)\nhola  mundo"
        .to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 3);
    assert_eq!(program_output[0], "hola mundo");
    assert_eq!(program_output[1], "hey hola mundo !");
    assert_eq!(program_output[2], "(hola mundo)");
}

#[test]
fn test_enunciado_12() {
    let regex = "[[:upper:]]ascal[[:upper:]]ase".to_string();
    let lines = "CascalCase\nbascalcase\n3ascal8ase\nthis is PascalRase yeah!\nascalase\n ascal ase\n?ascal!ase".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(program_output[0], "CascalCase");
    assert_eq!(program_output[1], "this is PascalRase yeah!");
}

#[test]
fn test_enunciado_13() {
    let regex = "es el fin$".to_string();
    let lines = "es el fin\nefectivamente, es el fin\nes el fin... o no\nno es el fin \nthis is fin\nsera? si, es el fin!\n".to_string();

    let program_output = run_rgrep(regex, lines).unwrap();

    assert_eq!(program_output.len(), 2);
    assert_eq!(program_output[0], "es el fin");
    assert_eq!(program_output[1], "efectivamente, es el fin");
}
