pub mod tokenizer {
    use crate::{lexing_preprocessor::{
        lexing_preprocessor::refactor,
        parse_err::{self},
    }, intermediate::{dictionary::{ConstValue, ShallowType}, Kind}, tree_walker::tree_walker::Line};
    const RESERVED_CHARS: &str = " +-*/=%;:,.({<[]>})&|!?\"'\\";
    pub fn tokenize(
        file: &[u8],
        format: bool,
    ) -> (Vec<Tokens>, Vec<(usize, usize)>, Vec<parse_err::Errors>) {
        let allocation_size = (file.len() as f64 * 0.7) as usize;
        let mut tokens: Vec<Tokens> = Vec::with_capacity(allocation_size);
        let mut text_pos: Vec<(usize, usize)> = Vec::with_capacity(allocation_size);
        text_pos.push((0, 0));
        let mut errors: Vec<parse_err::Errors> = vec![];

        let mut i = 0;
        while i < file.len() {
            let res = get_token(&file[i..]);
            text_pos.push((
                text_pos[text_pos.len() - 1].0 + res.1,
                text_pos[text_pos.len() - 1].1,
            ));
            if let Tokens::Whitespace(txt) = &res.0 {
                if txt == "\n" {
                    let len = text_pos.len() - 1;
                    text_pos[len].1 += 1;
                    text_pos[len].0 = 0;
                }
            }
            tokens.push(res.0);
            i += res.1;
        }
        if !format {
            return (tokens, text_pos, errors);
        }
        if let Ok(refactored) = refactor(tokens, text_pos, &mut errors) {
            return (refactored.0, refactored.1, errors);
        } else {
            println!("neco se pokazilo");
            panic!();
        }
    }
    pub fn get_token(line: &[u8]) -> (Tokens, usize) {
        let len = find_ws_str(line, &RESERVED_CHARS);
        let len = if len == 0 { 1 } else { len };
        let str = &line[0..len];
        let token = parse_token(std::str::from_utf8(str).unwrap());
        return (token, str.len());
    }
    pub fn parse_token(string: &str) -> Tokens {
        // +-*/=%;:,.({<[]>})&|!?"'\
        match string {
            "+" => Tokens::Operator(Operators::Plus),
            "-" => Tokens::Operator(Operators::Minus),
            "*" => Tokens::Operator(Operators::Star),
            "/" => Tokens::Operator(Operators::Slash),
            "=" => Tokens::Operator(Operators::Equal),
            "%" => Tokens::Operator(Operators::Mod),
            "&" => Tokens::Operator(Operators::Ampersant),
            "|" => Tokens::Operator(Operators::Pipe),
            "!" => Tokens::Operator(Operators::Not),
            "?" => Tokens::Optional,
            ";" => Tokens::Semicolon,
            ":" => Tokens::Colon,
            "," => Tokens::Comma,
            "." => Tokens::Dot,
            "\"" => Tokens::DoubleQuotes,
            r"'" => Tokens::Quotes,
            "(" => Tokens::Parenteses(false),
            ")" => Tokens::Parenteses(true),
            "{" => Tokens::CurlyBracket(false),
            "}" => Tokens::CurlyBracket(true),
            "<" => Tokens::Operator(Operators::AngleBracket(false)),
            ">" => Tokens::Operator(Operators::AngleBracket(true)),
            "[" => Tokens::SquareBracket(false),
            "]" => Tokens::SquareBracket(true),
            " " => Tokens::Space,
            _ => {
                if is_whitespace(string) {
                    Tokens::Whitespace(string.to_string())
                } else {
                    Tokens::Text(string.to_string())
                }
            }
        }
    }
    fn is_whitespace(str: &str) -> bool {
        for char in str.chars() {
            if !char.is_whitespace() {
                return false;
            }
        }
        true
    }
    pub fn deparse_token(token: &Tokens) -> String {
        // +-*/=%;:,.({<[]>})&|!?"'\
        match token {
            Tokens::Operator(Operators::Plus) => "+".to_string(),
            Tokens::Operator(Operators::Minus) => "-".to_string(),
            Tokens::Operator(Operators::Star) => "*".to_string(),
            Tokens::Operator(Operators::Slash) => "/".to_string(),
            Tokens::Operator(Operators::Equal) => "=".to_string(),
            Tokens::Operator(Operators::Mod) => "%".to_string(),
            Tokens::Operator(Operators::And) => "&&".to_string(),
            Tokens::Operator(Operators::Or) => "||".to_string(),
            Tokens::Operator(Operators::Ampersant) => "&".to_string(),
            Tokens::Operator(Operators::Pipe) => "|".to_string(),
            Tokens::Operator(Operators::Not) => "!".to_string(),
            Tokens::Operator(Operators::AddEq) => "+=".to_string(),
            Tokens::Operator(Operators::SubEq) => "-=".to_string(),
            Tokens::Operator(Operators::MulEq) => "*=".to_string(),
            Tokens::Operator(Operators::DivEq) => "/=".to_string(),
            Tokens::Operator(Operators::DoubleEq) => "==".to_string(),
            Tokens::Operator(Operators::NotEqual) => "!=".to_string(),
            Tokens::Operator(Operators::LessEq) => "<=".to_string(),
            Tokens::Operator(Operators::MoreEq) => ">=".to_string(),
            Tokens::Optional => "?".to_string(),
            Tokens::Semicolon => ";".to_string(),
            Tokens::Colon => ":".to_string(),
            Tokens::Comma => ",".to_string(),
            Tokens::Dot => ".".to_string(),
            Tokens::DoubleQuotes => "\"".to_string(),
            Tokens::Quotes => r"'".to_string(),
            Tokens::Parenteses(false) => "(".to_string(),
            Tokens::Parenteses(true) => ")".to_string(),
            Tokens::CurlyBracket(false) => "{".to_string(),
            Tokens::CurlyBracket(true) => "}".to_string(),
            Tokens::Operator(Operators::AngleBracket(false)) => "<".to_string(),
            Tokens::Operator(Operators::AngleBracket(true)) => ">".to_string(),
            Tokens::SquareBracket(false) => "[".to_string(),
            Tokens::SquareBracket(true) => "]".to_string(),
            Tokens::Space => " ".to_string(),
            Tokens::Text(string) => string.to_string(),
            Tokens::DoubleColon => "::".to_string(),
            Tokens::Number(num, t) => {
                let mut res = num.to_string();
                res.push(*t);
                res
            }
            Tokens::String(string) => {
                let mut res = "\"".to_string();
                res.push_str(string);
                res.push('"');
                res
            },
            Tokens::Char(char) => {
                let mut res = "'".to_string();
                res.push(*char);
                res.push('\'');
                res
            },
            Tokens::Whitespace(string) => string.to_string(),
            Tokens::Tab => "\t".to_string(),
            Tokens::Deleted => unreachable!("deleted token"),
            Tokens::DocComment(string) => {
                let mut res = "///".to_string();
                res.push_str(string);
                res   
            }
            Tokens::EndOfFile => unreachable!("end of file token"),
            
        }
    }
    pub fn find_ws_str(expression: &[u8], tokens_str: &str) -> usize {
        let mut idx = 0;

        for char in expression {
            if tokens_str.contains(*char as char) || (*char as char).is_whitespace() {
                break;
            }
            idx += 1;
        }
        idx
    }
    /// "+-*/=%;:,.({<[]>})&|!?\"'\\"
    #[derive(Debug, PartialEq, Clone)]
    pub enum Tokens {
        /// opening 0, closing 1
        Parenteses(bool),
        /// opening 0, closing 1
        CurlyBracket(bool),
        /// opening 0, closing 1
        SquareBracket(bool),
        Operator(Operators),
        Colon,
        Dot,
        Semicolon,
        Comma,
        Quotes,
        DoubleQuotes,
        Optional,
        Space,
        /// content
        String(String),
        Char(char),
        Whitespace(String),
        /// in case we can not identify token at the moment
        Text(String),
        DoubleColon,
        Number(f64, char),
        Tab,
        Deleted,
        DocComment(String),
        EndOfFile,
    }
    impl std::fmt::Display for Tokens {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", deparse_token(self))
        }
    }
    #[derive(Debug, PartialEq, Clone, Copy, Eq)]
    pub enum Operators {
        Plus,
        Minus,
        Star,
        Slash,
        Mod,
        AddEq,
        SubEq,
        MulEq,
        DivEq,
        Equal,
        DoubleEq,
        NotEqual,
        LessEq,
        MoreEq,
        And,
        Or,
        Not,
        Ampersant,
        Pipe,
        /// opening 0, closing 1
        AngleBracket(bool),
    }
    impl Tokens {
        pub fn into_const_number(&self, line: Line) -> Option<(ConstValue, Kind)> {
            match self {
                Tokens::Number(num, t) => {
                    match t {
                        'i' => {
                            let val = ConstValue::Int(*num as i64);
                            let kind = val.gen_type(line)?;
                            Some((val, kind))
                        },
                        'f' => {
                            let val = ConstValue::Float(*num);
                            let kind = val.gen_type(line)?;
                            Some((val, kind))
                        }
                        'u' => {
                            let val = ConstValue::Uint(*num as usize);
                            let kind = val.gen_type(line)?;
                            Some((val, kind))
                        }
                        'c' => {
                            let val = ConstValue::Char(*num as u8 as char);
                            let kind = val.gen_type(line)?;
                            Some((val, kind))
                        }
                        'n' => {
                            let val = ConstValue::Number(*num);
                            let kind = val.gen_type(line)?;
                            Some((val, kind))
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        }
    }
}
