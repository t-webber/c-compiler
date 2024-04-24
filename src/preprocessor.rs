use std::env::consts::OS;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::errors::{FailError, GeneralError, PreprocessorError, SystemError};
use crate::parser::parse_preprocessor;
use crate::reader::eval_tokens;
use crate::structs::{Directive, MacroValue, ParsingState, Pips, StoreDirective};

#[rustfmt::skip]
pub fn deal_with_c(ch: char, state: &mut ParsingState, current_directive: &mut StoreDirective) -> String {
    let mut tmp_dir_state = None;
    let res = 
    match &mut state.directive_parsing {
        Pips::None => String::from(ch),
        Pips::DirectiveName(name) if ch.is_whitespace() && name.is_empty() => {String::new()},
        Pips::DirectiveName(name) if ch.is_whitespace() => {tmp_dir_state = Some(if name.trim()=="define" { Pips::DirectiveArgs(vec![]) } else { Pips::DirectiveValue(String::new()) }); current_directive.values.push(name.clone()); String::new()},
        Pips::DirectiveName(ref mut name) =>{name.push(ch); String::new()},
        Pips::DirectiveArgs(ref mut args ) if args.is_empty() && ch == '(' => {args.push(String::new()); String::new()},
        Pips::DirectiveArgs(_) if ch == '(' => panic!("Nested parenthesis are not supported"),
        Pips::DirectiveArgs(ref mut args ) if ch == ')' => {tmp_dir_state = Some(Pips::DirectiveValue(String::new())); current_directive.values.extend(args.iter().map(Clone::clone)); String::new()},
        Pips::DirectiveArgs(ref mut args ) if args.is_empty() => {tmp_dir_state = Some(Pips::DirectiveValue(String::from(ch))); String::new()},
        Pips::DirectiveArgs(ref mut args ) => {args.last_mut().expect("Fatal Error: we're fucked!").push(ch); String::new()},
        Pips::DirectiveValue(ref mut value) => {value.push(ch); String::new()},
    };
    if let Some(newstate) = tmp_dir_state {
        state.directive_parsing = newstate;
    }
    res
}

#[rustfmt::skip]
pub fn preprocess_character(ch: char, state: &mut ParsingState, previous_char: &mut char, current_directive: &mut StoreDirective) -> String {
    let in_comment = state.comment_level>0 || state.inline_comment;
    // Match double chars tokens
    let prev = *previous_char;
    *previous_char = ch;
    let character = match ch {
        '/' if prev =='*' && in_comment => {state.comment_level=state.comment_level.checked_sub(1).expect("*/ unmatched");*previous_char=' ';state.inline_comment=false;String::new()},
        '/' if prev =='*' => { GeneralError::UnclosedComment { file_position: &state.current_position, level: state.comment_level }.fail_with_panic(&state.current_position)},
        '/' if prev =='/' => {state.inline_comment=true;*previous_char=' ';String::new()} ,
        '*' if prev =='/' => {state.comment_level+=1;state.comment_unclosed_positon.push(state.current_position.clone());*previous_char=' ';String::new()},
        _ if (prev=='/' || prev=='*') && in_comment => {  String::new() },
        _ if prev=='/' || prev=='*' => { deal_with_c(prev, state, current_directive)+deal_with_c(ch, state, current_directive).as_str() },
        '/'|'*' => { String::new() }
        _ if in_comment => { String::new() },
        '#' => match state.directive_parsing {
                Pips::None => { state.directive_parsing = Pips::DirectiveName(String::new()); String::new() },
                Pips::DirectiveName(_) | Pips::DirectiveArgs(_) | Pips::DirectiveValue(_) => {deal_with_c(ch, state, current_directive)}
            },
        _ => { deal_with_c(ch, state, current_directive) }
    };
    if state.if_writing {
        character
    } else {
        String::new()
    }
}

#[rustfmt::skip]
fn preprocess_define(directive: &Directive, state: &mut ParsingState) -> String {
    if let Directive::Define { macro_name, macro_args, macro_value } = directive {
        if macro_args.is_empty() {
            state.defines.insert(macro_name.clone(), MacroValue::String(macro_value.clone()));
        } else {
            state.defines.insert(macro_name.clone(), MacroValue::Function { args: macro_args.clone(), body: macro_value.clone() });
        };
        String::new()
    } else {
        PreprocessorError::Internal("not a define directive").fail_with_panic(&state.current_position);
    }
}

fn look_for_file(filename: &String, state: &mut ParsingState) -> File {
    let mut places: Vec<PathBuf> = if OS == "linux" {
        vec![
            "/usr/include/",
            "/usr/local/include/",
            "/usr/include/x86_64-linux-gnu/",
            "/usr/lib/llvm-14/lib/clang/14.0.0/include/",
        ]
    } else if OS == "windows" {
        vec![
            "D:\\Windows Kits\\10\\Include\\10.0.22621.0\\ucrt\\",
            "D:\\Windows Kits\\10\\Include\\10.0.22621.0\\shared\\",
            "D:\\Windows\\Apps\\Visual\\Studio\\IDE\\VC\\Tools\\MSVC\\14.39.33519\\include\\",
        ]
    } else {
        SystemError::UnsupportedOS(OS).fail_with_panic(&state.current_position);
    }
    .iter()
    .map(|path: &&str| PathBuf::from(path))
    .collect();
    match PathBuf::from(&state.current_position.filepath).parent() {
        Some(path) => places.push(path.to_owned()),
        None => GeneralError::AccessLocalDenied.fail_with_warning(&state.current_position),
    };
    for place in places {
        let filepath = place.join(Path::new(&filename));
        if filepath.exists() {
            state.new_file(
                filename.clone(),
                String::from(filepath.as_os_str().to_str().expect("Invalid path")),
            );
            return File::open(filepath).expect("Failed to open file from local directory");
        }
    }
    PreprocessorError::FileNotFound(filename).fail_with_panic(&state.current_position);
}

fn preprocess_include(filename: &String, state: &mut ParsingState) -> String {
    if state
        .include_stack
        .iter()
        .any(|file| file.filename == *filename)
    {
        return String::new();
    }
    let mut content = String::new();
    let old_position = state.current_position.clone();
    look_for_file(filename, state)
        .read_to_string(&mut content)
        .expect("Failed to convert file");
    let preprocessed_file = preprocess(&content, state);
    state.current_position = old_position;
    preprocessed_file
}

#[derive(PartialEq, Debug)]
enum DirectiveParsingState {
    Name,
    AfterName,
    Args,
    Value,
}

#[rustfmt::skip]
fn convert_define_from_store(values: &&str, parsing_state: &mut ParsingState) -> Directive {
    let mut directive_state = DirectiveParsingState::Name;
    let mut brace_level: usize = 0;
    let mut macro_name = String::new();
    let mut args: Vec<String> = vec![];
    let mut value = String::new();
    values.chars().for_each(|ch| match ch {
        _ if directive_state == DirectiveParsingState::Value => value.push(ch),
        '(' if directive_state == DirectiveParsingState::Name || directive_state == DirectiveParsingState::AfterName => {
                    brace_level += 1;
                    directive_state = DirectiveParsingState::Args;
                    args.push(String::new());
                    value.push(ch);
                }
        '(' if brace_level > 0 => {
            directive_state = DirectiveParsingState::Value;
            args.clear();
            value.push(ch);
        }
        '(' => {
            brace_level += 1;
            value.push(ch);
        }
        ' ' if directive_state == DirectiveParsingState::Name => directive_state = DirectiveParsingState::AfterName,
        ' ' => {}
        ')' if directive_state == DirectiveParsingState::Name => {
            PreprocessorError::InvalidMacroName("Unexpected ')' in macro name").fail_with_panic(&parsing_state.current_position)
        }
        ')' if brace_level > 1 => {
            brace_level -= 1;
            value.push(ch);
            args.last_mut().unwrap().push(ch);
        }
        ')' if directive_state == DirectiveParsingState::Args => {
            brace_level = brace_level.checked_sub(1).expect("Unmatched (");
            directive_state = DirectiveParsingState::Value;
            value.clear();
        }
        ')' => value.push(ch),
        ',' => args.push(String::new()),
        _ if directive_state == DirectiveParsingState::Name => macro_name.push(ch),
        _ if directive_state == DirectiveParsingState::AfterName => {
            directive_state = DirectiveParsingState::Value;
            value.push(ch);
        }
        _ => {
            args.last_mut().unwrap().push(ch);
            value.push(ch);
        }
    });
    if value.is_empty() {
        value = format!(
            "({})",
            args.iter().fold(String::new(), |acc, curr_string| acc + curr_string.as_str())
        );
        args.clear();
    }

    Directive::Define {
        macro_name,
        macro_args: args,
        macro_value: value,
    }
}

#[rustfmt::skip]
fn convert_from_store(directive: &StoreDirective, state: &mut ParsingState) -> Directive {
    let d =  directive
        .values
        .iter()
        .filter_map(|s: &String| {let trimmed = s.as_str().trim(); if trimmed.is_empty() {None} else {Some(trimmed)}})
        .collect::<Vec<&str>>();
    match d.as_slice()
    {
        ["define", values] => convert_define_from_store(values, state),
        ["undef", macro_name] => Directive::Undef {
            macro_name: String::from(*macro_name),
        },
        ["if", expression_string] => {
            // let ast = vec2ternary_ast(parse_preprocessor(expression_string));
            let res = eval_tokens(&parse_preprocessor(expression_string), state);
            Directive::If {
                // expression: eval_all(&ast, state) != 0,
                expression: res != 0,
            }
        }
        ["elif", expression_string] => {
            // let ast = vec2ternary_ast(parse_preprocessor(expression_string));
            let res = eval_tokens(&parse_preprocessor(expression_string), state);
            Directive::Elif {
                // expression: eval_all(&ast, state) != 0,
                expression: res != 0,
            }
        }
        ["endif"]  => Directive::EndIf {},
        ["else"] => Directive::Else,
        ["ifdef", macro_name] => Directive::IfDef {
            macro_name: String::from(*macro_name),
        },
        ["ifndef", macro_name] => Directive::IfnDef {
            macro_name: String::from(*macro_name),
        },
        ["include", filename] => {
            let trimed_filename = filename.trim();
            let clamped_filename = String::from(&trimed_filename[1..trimed_filename.len() - 1]);
            Directive::Include {
                filename: clamped_filename,
            }
        }
        ["error"] => Directive::Error {
            message: String::new(),
        },
        ["error", message] => Directive::Error {
            message: String::from(*message),
        },
        ["warning", message] => Directive::Warning {
            message: String::from(*message),
        },
        ["pragma", message] => Directive::Pragma {
            message: String::from(*message),
        },
        x => PreprocessorError::DirectiveUnknown(&x.join("")).fail_with_panic(&state.current_position),
    }
}

#[rustfmt::skip]
fn preprocess_directive(directive: &Directive, state: &mut ParsingState) -> String {
    // println!("Directive: {directive:?}");
    if state.if_level > 0 {
        match directive {
            Directive::EndIf => {
                state.if_level = state
                    .if_level
                    .checked_sub(1)
                    .expect("We're indeniably fucked");
            }
            Directive::Else { .. } | Directive::Elif { .. } => (),
            Directive::If { .. } => {
                state.if_level += 1;
                return String::new();
            },
            _ if state.if_writing => (),
            _ => return String::new(),
        }
    }
    match directive {
        Directive::Define { .. } => preprocess_define(directive, state),
        Directive::IfDef { macro_name } => {
            state.if_level += 1;
            state.if_writing = state.defines.contains_key(macro_name);
            String::new()
        }
        Directive::IfnDef { macro_name } => {
            state.if_level += 1;
            state.if_writing = !state.defines.contains_key(macro_name);
            String::new()
        }
        Directive::If { expression } => {
            state.if_level += 1;
            state.if_writing = *expression;
            String::new()
        }
        Directive::Elif { expression } => {
            if state.if_writing {
                state.if_writing = false;
            } else {
                state.if_writing = *expression;
            }
            String::new()
        }
        Directive::Include { filename } => preprocess_include(filename, state),
        Directive::Undef { macro_name } => {
            state.defines.remove(macro_name);
            String::new()
        }
        Directive::Else => {
            state.if_writing = (!state.if_writing) && state.if_level==1;
            String::new()
        }
        Directive::EndIf => {
            state.if_writing = true;
            String::new()
        }
        Directive::Error { message } => {
            // PreprocessorError::DirectiveError(message).fail_with_panic(&state.current_position);
            PreprocessorError::DirectiveError(message).fail_with_warning(&state.current_position);
            String::new()
        }
        Directive::Warning { message } => {
            PreprocessorError::DirectiveWarning(message).fail_with_warning(&state.current_position);
            String::new()
        }
        Directive::Pragma { message } => {
            PreprocessorError::DirectiveUnknown(message).fail_with_warning(&state.current_position);
            String::new()
        }
        Directive::None => 
            PreprocessorError::DirectiveNameMissing.fail_with_panic(&state.current_position),
    }
}

#[rustfmt::skip]
pub fn preprocess(content: &str, state: &mut ParsingState) -> String {
    let mut lines: Vec<(u32, String)> = vec![];
    let mut previous_line_escaped = false;
    let mut line_number: u32 = 1;
    for line in content.lines() {
        let escaped = line.ends_with('\\');
        let trimed_line = if escaped {
            &line[0..line.len() - 1]
        } else {
            line
        };
        if previous_line_escaped {
            lines
                .last_mut()
                .expect("Unrecoverable error")
                .1
                .push_str(trimed_line.trim_start());
        } else {
            lines.push((line_number, String::from(trimed_line)));
        }
        line_number += 1;
        previous_line_escaped = escaped;
    }

    let processed_file = lines
        .iter()
        .map(|(curr_line_nb, line)| {
            state.current_position.col = 0;
            state.current_position.line = *curr_line_nb;
            state.inline_comment = false;
            state.directive_parsing = Pips::None;
            let mut current_directive = StoreDirective::default();
            let line_string = line.to_string();
            let mut previous_char = ' ';
            let mut preprocessed_line = line_string
                .chars()
                .map(|ch| {
                    state.current_position.col += 1;
                    preprocess_character(ch, state, &mut previous_char, &mut current_directive)
                })
                .collect::<String>();
            if !preprocessed_line.trim().is_empty() {
                preprocessed_line += "\n";
            }
            // println!("Hashmap: {}", format!("{:?}", state.defines).blue());
            match &state.directive_parsing {
                Pips::DirectiveValue(value) => {
                    if !value.is_empty() {current_directive.values.push(value.clone());}
                    preprocess_directive(&convert_from_store(&current_directive, state), state)
                }
                Pips::DirectiveName(name) => {
                    current_directive.values.push(name.clone());
                    preprocess_directive(&convert_from_store(&current_directive, state), state)
                }
                Pips::DirectiveArgs(args) => {
                    assert!(args.is_empty(), "{}", PreprocessorError::MacroArgsNotClosed.fail(&state.current_position));
                    preprocess_directive(&convert_from_store(&current_directive, state), state)
                }
                Pips::None => preprocessed_line,
            }
        })
        .collect();
    assert!(
        state.comment_level == 0,
        "{}",
        GeneralError::UnclosedComment { file_position: state.comment_unclosed_positon.last().expect("Error raised but everything is fine!"), level: state.comment_level }.fail(&state.current_position)
    );
    state.end_file();
    processed_file
}

#[allow(clippy::too_many_lines)]
fn add_default_macro(state: &mut ParsingState) {
    let macros = &[
        //
        // Standard Predefined Macros
        //
        ("__FILE__", "forgotten"),
        ("__LINE__", "1"), // mut
        ("__DATE__", "forgotten"),
        ("__TIME__", "forgotten"),
        ("__STDC__", "1"),
        ("__STDC_VERSION__", "199910L"),
        ("__STDC_HOSTED__", "1"),
        //
        // Common Predefined Macros
        //
        ("__COUNTER__", "0"),
        ("__GNUC__", "10"),
        ("__GNUC_MINOR__", "3"),
        ("__GNUC_PATCHLEVEL__", "0"),
        ("__GNUG__", "0"),
        ("__BASE__FILE__", "forgotten"),
        ("__FILE_NAME__", "forgotten"), // mut
        ("__INCLUDE_LEVEL__", "0"),     // mut
        ("__VERSION__", "TeleCC 1.0"),
        ("__GNUC_GNU_INLINE", "1"), // before 90
        // ("__GNUC_STDC_INLINE", "1"), after 99
        ("__USER_LABEL_PREFIX__", "not understood"),
        ("__REGISTER_PREFIX__", "not understood"),
        ("__SIZE_TYPE__", "long unsigned int"),
        ("__PTRDIFF_TYPE__", "long int"),
        ("__WCHAR_TYPE__", "int"),
        ("__WINT_TYPE__", "unsigned int"),
        ("__INTMAX_TYPE__", "long int"),
        ("__UINTMAX_TYPE__", "long unsigned int"),
        // __SIG_ATOMIC_TYPE__
        ("__INT8_TYPE__", "signed char"),
        ("__INT16_TYPE__", "short"),
        ("__INT32_TYPE__", "int"),
        ("__INT64_TYPE__", "long int"),
        ("__UINT8_TYPE__", "unsigned char"),
        ("__UINT16_TYPE__", "unsigned short"),
        ("__UINT32_TYPE__", "unsigned int"),
        ("__UINT64_TYPE__", "long unsigned int"),
        ("__INT_LEAST8_TYPE__", "signed char"),
        ("__INT_LEAST16_TYPE__", "short"),
        ("__INT_LEAST32_TYPE__", "int"),
        ("__INT_LEAST64_TYPE__", "long int"),
        ("__UINT_LEAST8_TYPE__", "unsigned char"),
        ("__UINT_LEAST16_TYPE__", "unsigned short"),
        ("__UINT_LEAST32_TYPE__", "unsigned int"),
        ("__UINT_LEAST64_TYPE__", "long unsigned int"),
        ("__INT_FAST8_TYPE__", "signed char"),
        ("__INT_FAST16_TYPE__", "short"),
        ("__INT_FAST32_TYPE__", "int"),
        ("__INT_FAST64_TYPE__", "long int"),
        ("__UINT_FAST8_TYPE__", "unsigned char"),
        ("__UINT_FAST16_TYPE__", "unsigned short"),
        ("__UINT_FAST32_TYPE__", "unsigned int"),
        ("__UINT_FAST64_TYPE__", "long unsigned int"),
        ("__INTPTR_TYPE__", "long int"),
        ("__UINTPTR_TYPE__", "long unsigned int"),
        ("__CHAR_BIT__", "8"),
        // _SCHAR_MAX__
        // __WCHAR_MAX__
        // __SHRT_MAX__
        // __INT_MAX__
        // __LONG_MAX__
        // __LONG_LONG_MAX__
        // __WINT_MAX__
        // __SIZE_MAX__
        // __PTRDIFF_MAX__
        // __INTMAX_MAX__
        // __UINTMAX_MAX__
        // __SIG_ATOMIC_MAX__
        // __INT8_MAX__
        // __INT16_MAX__
        // __INT32_MAX__
        // __INT64_MAX__
        // __UINT8_MAX__
        // __UINT16_MAX__
        // __UINT32_MAX__
        // __UINT64_MAX__
        // __INT_LEAST8_MAX__
        // __INT_LEAST16_MAX__
        // __INT_LEAST32_MAX__
        // __INT_LEAST64_MAX__
        // __UINT_LEAST8_MAX__
        // __UINT_LEAST16_MAX__
        // __UINT_LEAST32_MAX__
        // __UINT_LEAST64_MAX__
        // __INT_FAST8_MAX__
        // __INT_FAST16_MAX__
        // __INT_FAST32_MAX__
        // __INT_FAST64_MAX__
        // __UINT_FAST8_MAX__
        // __UINT_FAST16_MAX__
        // __UINT_FAST32_MAX__
        // __UINT_FAST64_MAX__
        // __INTPTR_MAX__
        // __UINTPTR_MAX__
        // __WCHAR_MIN__
        // __WINT_MIN__
        // __SIG_ATOMIC_MIN__
        // __INT8_C
        // __INT16_C
        // __INT32_C
        // __INT64_C
        // __UINT8_C
        // __UINT16_C
        // __UINT32_C
        // __UINT64_C
        // __INTMAX_C
        // __UINTMAX_C
        // __SCHAR_WIDTH__
        // __SHRT_WIDTH__
        // __INT_WIDTH__
        // __LONG_WIDTH__
        // __LONG_LONG_WIDTH__
        // __PTRDIFF_WIDTH__
        // __SIG_ATOMIC_WIDTH__
        // __SIZE_WIDTH__
        // __WCHAR_WIDTH__
        // __WINT_WIDTH__
        // __INT_LEAST8_WIDTH__
        // __INT_LEAST16_WIDTH__
        // __INT_LEAST32_WIDTH__
        // __INT_LEAST64_WIDTH__
        // __INT_FAST8_WIDTH__
        // __INT_FAST16_WIDTH__
        // __INT_FAST32_WIDTH__
        // __INT_FAST64_WIDTH__
        // __INTPTR_WIDTH__
        // __INTMAX_WIDTH__
        // __SIZEOF_INT__
        // __SIZEOF_LONG__
        // __SIZEOF_LONG_LONG__
        // __SIZEOF_SHORT__
        // __SIZEOF_POINTER__
        // __SIZEOF_FLOAT__
        // __SIZEOF_DOUBLE__
        // __SIZEOF_LONG_DOUBLE__
        // __SIZEOF_SIZE_T__
        // __SIZEOF_WCHAR_T__
        // __SIZEOF_WINT_T__
        // __SIZEOF_PTRDIFF_T__
        // __BYTE_ORDER__
        // __FLOAT_WORD_ORDER__
        //
        // System-specific Predefined Macros
        //
        ("_DEFAULT_SOURCE", "1"),
        ("_BSD_SOURCE", "1"),
        ("_SVID_SOURCE", "1"),
        ("_XOPEN_SOURCE", "600"),
        ("_XOPEN_SOURCE_EXTENDED", "1"),
        ("_LARGEFILE_SOURCE", "1"),
        ("_FILE_OFFSET_BITS", "64"),
        ("_POSIX_SOURCE", "1"),
        ("_POSIX_C_SOURCE", "200809L"),
        ("_ISOC99_SOURCE", "1"),
        ("_ISOC11_SOURCE", "1"),
        ("_ISOC2X_SOURCE", "1"),
        ("_ATFILE_SOURCE", "1"),
        ("_GNU_SOURCE", "1"),
        ("_REENTRANT", "1"),
        ("_THREAD_SAFE", "1"),
        ("_FORTIFY_SOURCE", "2"),
        ("_TANDEM_SOURCE", "1"),
        ("_NETBSD_SOURCE", "1"),
        ("_OPENBSD_SOURCE", "1"),
        ("_BSD_SOURCE", "1"),
        ("_SVID_SOURCE", "1"),
        ("_XOPEN_SOURCE", "600"),
        ("_XOPEN_SOURCE_EXTENDED", "1"),
        ("_LARGEFILE_SOURCE", "1"),
        ("_FILE_OFFSET_BITS", "64"),
        ("_POSIX_SOURCE", "1"),
        ("_POSIX_C_SOURCE", "200809L"),
        ("_ISOC99_SOURCE", "1"),
        ("_ISOC11_SOURCE", "1"),
        ("_ISOC2X_SOURCE", "1"),
        ("_ATFILE_SOURCE", "1"),
        ("_GNU_SOURCE", "1"),
        ("_REENTRANT", "1"),
        ("_THREAD_SAFE", "1"),
        ("_FORTIFY_SOURCE", "2"),
        ("_TANDEM_SOURCE", "1"),
        ("_NETBSD_SOURCE", "1"),
        ("_OPENBSD_SOURCE", "1"),
        ("_POSIX_THREADS", "1"),
        ("_POSIX_THREAD_SAFE_FUNCTIONS", "1"),
        ("_POSIX_REENTRANT_FUNCTIONS", "1"),
        ("_POSIX_C_SOURCE", "200809L"),
        ("_POSIX_SOURCE", "1"),
        ("_POSIX_C_SOURCE", "200809L"),
        ("_POSIX_C_SOURCE", "200809L"),
        ("_POSIX_C_SOURCE", "200809L"),
        ("__x86_64__", "1"),
    ];
    for (name, value) in macros {
        state.defines.insert(
            String::from(*name),
            MacroValue::String(String::from(*value)),
        );
    }
}

pub fn preprocess_unit(filepath: PathBuf) -> String {
    let mut content = String::new();
    File::open(&filepath)
        .expect("Failed to read the file")
        .read_to_string(&mut content)
        .expect("Failed to convert the file");
    let mut state = ParsingState {
        if_writing: true,
        ..Default::default()
    };
    add_default_macro(&mut state);
    state.new_file(
        filepath
            .file_name()
            .expect("Invalid filepath")
            .to_owned()
            .into_string()
            .expect("Invalid filename"),
        filepath
            .into_os_string()
            .into_string()
            .expect("Invalid path"),
    );
    preprocess(&content, &mut state)
}
