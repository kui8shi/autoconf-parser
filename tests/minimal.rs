#![deny(rust_2018_idioms)]
use autoconf_parser::lexer::Lexer;
use autoconf_parser::parse::{MinimalParser, ParseErrorKind::*};
mod minimal_util;
use minimal_util::*;

pub fn make_parser_minimal(src: &str) -> MinimalParser<Lexer<std::str::Chars<'_>>> {
    MinimalParser::new(Lexer::new(src.chars()))
}

#[test]
fn test_minimal_macro_and_unusual_style_of_newline() {
    let input = r#"dnl comment
MACRO(arg1, arg2)[]dnl unusual style of comment
"#;
    let mut p = make_parser_minimal(input);
    let correct = m4_macro_as_cmd("MACRO", &[m4_raw("arg1"), m4_raw("arg2")]);
    let result = p.complete_command();
    match result {
        Ok(c) => assert_eq!(Some(correct), c),
        Err(e) => {
            println!("{}", e);
            panic!();
        }
    }
}

#[test]
fn test_minimal_macro_test() {
    let input =
        r#"GMP_DEFINE_RAW("define_not_for_expansion(\`HAVE_DOUBLE_IEEE_BIG_ENDIAN')", POST)"#;
    let mut p = make_parser_minimal(input);
    dbg!(p.complete_command().unwrap());
}

#[test]
fn test_minimal_condition() {
    let input = r#"test "$foo" = "yes" && foo=1"#;
    let mut p = make_parser_minimal(input);
    dbg!(p.complete_command().unwrap());
}

#[test]
fn test_minimal_macro_word_and_empty_quotes() {
    let input = r#"WORD_[]MACRO([$var],[arg2],[arg3])[]_SUFFIX)"#;
    let mut p = make_parser_minimal(input);
    let correct = words(&[
        lit("WORD_"),
        m4_macro_as_word("MACRO", &[m4_raw("$var"), m4_raw("arg2"), m4_raw("arg3")]),
        lit("_SUFFIX"),
    ]);
    match p.word() {
        Ok(w) => assert_eq!(Some(correct), w),
        Err(e) => {
            println!("{}", e);
            panic!();
        }
    }
}

#[test]
fn test_minimal_macro_with_quoted_command_group() {
    let input = r#"m4_if([$var],,[echo found; echo $var],[])"#;
    let mut p = make_parser_minimal(input);
    let correct = m4_macro_as_cmd(
        "m4_if",
        &[
            m4_var("var"),
            m4_lit(""),
            m4_cmds(&[
                cmd_lits("echo", &["found"]),
                cmd_words("echo", &[word(var("var"))]),
            ]),
            m4_cmds(&[]),
        ],
    );
    match p.complete_command() {
        Ok(c) => assert_eq!(Some(correct), c),
        Err(e) => {
            println!("{}", e);
            panic!();
        }
    }
}

#[test]
fn test_macro_define_recursive() {
    let input = r#"
AC_DEFUN([GMP_SUBST_CHECK_FUNCS],
[m4_if([$1],,,
[_GMP_SUBST_CHECK_FUNCS(ac_cv_func_[$1],HAVE_[]m4_translit([$1],[a-z],[A-Z])_01)
GMP_SUBST_CHECK_FUNCS(m4_shift($@))])])"#;
    let mut p = make_parser_minimal(input);
    match p.complete_command() {
        Ok(cmd) => {
            dbg!(&cmd);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

#[test]
fn test_macro_define() {
    let input = r#"define(GMP_FAT_SUFFIX,
[[$1=`echo $2 | sed -e '/\//s:^[^/]*/::' -e 's:[\\/]:_:g'`]])"#;
    let mut p = make_parser_minimal(input);
    match p.complete_command() {
        Ok(cmd) => {
            dbg!(&cmd);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}

#[test]
fn test_macro_patsubst() {
    let input = "AC_INIT(PROGRAM, [patsubst(patsubst(esyscmd(grep \"^#define VERSION \" config.h.in /dev/null 2>/dev/null),^.*VERSION\t+,),[\t*$],).patsubst(patsubst(esyscmd(grep \"^#define VERSION_MINOR \" config.h.in /dev/null 2>/dev/null),^.*VERSION_MINOR \t+,),[\t*$],).patsubst(patsubst(esyscmd(grep \"^#define VERSION_PATCHLEVEL \" config.h.in /dev/null 2>/dev/null),^.*VERSION_PATCHLEVEL\t+,),[\t*$],)], [report@mail, see https://example.com], PROGRAM.tar)";
    let mut p = make_parser_minimal(input);
    match p.complete_command() {
        Ok(cmd) => {
            dbg!(&cmd);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
