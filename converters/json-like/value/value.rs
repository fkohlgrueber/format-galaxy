use codespan_reporting::{diagnostic::{self, Diagnostic, Label}, files::SimpleFile, term::{self, termcolor::{ColorChoice, StandardStream}}};
use indexmap::IndexMap;
use std::{io::{Error, ErrorKind, Read}, iter::Peekable};

use str_tree::StrTree;

#[derive(Default)]
struct ValueWriter {
    v: Vec<u8>
}

impl ValueWriter {
    pub fn write_str(&mut self, s: &str) {
        self.write_u64(s.len() as u64);
        self.v.extend(s.bytes());
    }

    pub fn write_u64(&mut self, n: u64) {
        self.v.extend(&n.to_le_bytes())
    }

    pub fn write_u8(&mut self, n: u8) {
        self.v.push(n);
    }
}

#[derive(Default)]
struct ValueReader<T: Read> {
    r: T
}

impl<T> ValueReader<T> 
where T: Read 
{
    pub fn from_reader(r: T) -> Self {
        ValueReader { r }
    }

    pub fn read_string(&mut self) -> Result<String, Error> {
        let len = self.read_u64()?;
        let mut buf = Vec::new();
        buf.resize(len as usize, 0u8);
        self.r.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    pub fn read_u64(&mut self) -> Result<u64, Error> {
        let mut buf = [0u8; 8];
        self.r.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        let mut buf = [0u8; 1];
        self.r.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

static INDENT_SIZE: usize = 2;

#[derive(Default)]
struct ValuePrinter {
    indent: usize,
    s: String
}

impl ValuePrinter {
    fn print_newline(&mut self) {
        self.s.push('\n');
        for _ in 0..self.indent*INDENT_SIZE {
            self.s.push(' ');
        }
    }

    fn print_str(&mut self, s: &str) {
        self.s.push_str(&format!("{:?}", s));
        /*self.s.push('"');
        for c in s.escape_default() {
            self.s.push(c);
        }
        self.s.push('"');*/
    }

    fn print_bool(&mut self, b: bool) {
        let s = if b {
            "true"
        } else {
            "false"
        };
        self.s.push_str(s);
    }

    fn print_null(&mut self) {
        self.s.push_str("null");
    }

    fn print_u64(&mut self, n: u64) {
        self.s.push_str(&n.to_string());
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn print(&mut self, c: char) {
        self.s.push(c);
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Num(u64),
    Str(String),
    Bool(bool),
    Null
}

struct Tokenizer<'a> {
    iter: std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'a>>>,
    tokens: Vec<Token>,
    input: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn new(s: &'a str) -> Self {
        Tokenizer {
            iter: s.chars().enumerate().peekable(),
            tokens: vec!(),
            input: s,
        }
    }

    fn format_diagnostic(&self, diag: Diagnostic<()>) -> String {
        let file = SimpleFile::new("input", self.input);
        let mut writer = termcolor::Buffer::no_color();
        let config = codespan_reporting::term::Config::default();
        term::emit(&mut writer, &config, &file, &diag).unwrap();
        String::from_utf8(writer.into_inner()).unwrap()
    }

    fn consume(&mut self, c: char) -> Result<(), String> {
        match self.iter.next() {
            None => {
                let num_chars = self.input.chars().count();
                let diag = Diagnostic::error()
                    .with_message("Unexpected end of input")
                    .with_labels(vec!(
                        Label::primary((), num_chars..num_chars+1).with_message(format!("Expected character `{}`", c)),
                    ));
                Err(self.format_diagnostic(diag))
            },
            Some((_idx, x)) if x == c => Ok(()),
            Some((idx, other)) => {
                let diag = Diagnostic::error()
                    .with_message("Unexpected input")
                    .with_labels(vec!(
                        Label::primary((), idx..idx+1).with_message(format!("Expected character `{}`, found `{}`", c, other)),
                    ));
                Err(self.format_diagnostic(diag))
            }
        }
    }

    fn consume_str(&mut self, s: &str) -> Result<(), String> {
        for c in s.chars() {
            self.consume(c)?;
        }
        Ok(())
    }

    fn tok_string(&mut self) -> Result<String, String> {
        self.consume('"')?;
        let mut s = String::new();
        let mut escape = false;
        loop {
            // handle string escapes
            match (self.iter.next(), escape) {
                (None, _) => { 
                    let num_chars = self.input.chars().count();
                    let diag = Diagnostic::error()
                        .with_message("Unexpected end of input")
                        .with_labels(vec!(
                            Label::primary((), num_chars..num_chars+1).with_message("Expected more characters "),
                        ));
                    return Err(self.format_diagnostic(diag));
                }
                (Some((_idx, '\\')), false) => { escape = true; }
                (Some((_idx, c)), true) => {
                    escape = false;
                    let c = match c {
                        't' => '\t',
                        'n' => '\n',
                        'r' => '\r',
                        c => c,
                    };
                    s.push(c);
                }
                (Some((_idx, '"')), false) => {
                    break;
                }
                (Some((_idx, c)), false) => {
                    s.push(c);
                }
            }
        }
        Ok(s)
    }

    fn tokenize(mut self) -> Result<Vec<Token>, String> {
        loop {
            match self.iter.peek() {
                None => { break },
                Some((_idx, c)) if c.is_ascii_whitespace() => {  // skip whitespace
                    self.iter.next();
                },
                Some((_idx, '[')) => {
                    self.tokens.push(Token::LBracket);
                    self.iter.next();
                }
                Some((_idx, ']')) => {
                    self.tokens.push(Token::RBracket);
                    self.iter.next();
                }
                Some((_idx, '{')) => {
                    self.tokens.push(Token::LBrace);
                    self.iter.next();
                }
                Some((_idx, '}')) => {
                    self.tokens.push(Token::RBrace);
                    self.iter.next();
                }
                Some((_idx, ':')) => {
                    self.tokens.push(Token::Colon);
                    self.iter.next();
                }
                Some((_idx, ',')) => {
                    self.tokens.push(Token::Comma);
                    self.iter.next();
                }
                Some((_idx, 't')) => {
                    self.consume_str("true")?;
                    self.tokens.push(Token::Bool(true));
                }
                Some((_idx, 'f')) => {
                    self.consume_str("false")?;
                    self.tokens.push(Token::Bool(false));
                }
                Some((_idx, 'n')) => {
                    self.consume_str("null")?;
                    self.tokens.push(Token::Null);
                }
                Some((_idx, '"')) => {
                    let s = self.tok_string()?;
                    self.tokens.push(Token::Str(s));
                }
                Some((start_idx, c)) if c.is_numeric() => {
                    let start_idx = *start_idx;
                    let mut s = String::new();
                    let mut end_idx = start_idx;
                    loop {
                        match self.iter.peek().cloned() {
                            Some((idx, c)) if c.is_numeric() => {
                                self.iter.next();
                                s.push(c);
                                end_idx = idx;
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    
                    let n = match s.parse() {
                        Ok(n) => n,
                        Err(_) => {
                            // number literal is too large
                            let diag = Diagnostic::error()
                                .with_message("Integer literal too large")
                                .with_labels(vec!(
                                    Label::primary((), start_idx..end_idx+1).with_message("Provided integer literal is too large"),
                                ));
                            return Err(self.format_diagnostic(diag));
                        }
                    };

                    self.tokens.push(Token::Num(n));
                }
                Some((idx, c)) => {
                    let diag = Diagnostic::error()
                        .with_message("Unexpected input")
                        .with_labels(vec!(
                            Label::primary((), *idx..*idx+1).with_message(format!("Unexpected character `{}`", c)),
                        ));
                    return Err(self.format_diagnostic(diag));
                }
            }
        }
        Ok(self.tokens)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(u64),
    String(String),
    Array(Vec<Value>),
    Object(IndexMap<String, Value>),
}

impl Value {
    fn type_id(&self) -> u8 {
        use Value::*;
        match self {
            Null => 0,
            Bool(_) => 1,
            Number(_) => 2,
            String(_) => 3,
            Array(_) => 4,
            Object(_) => 5,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut w = ValueWriter::default(); 
        self.serialize_(&mut w);
        return w.v;
    }

    fn serialize_(&self, w: &mut ValueWriter) {
        // write type id
        w.write_u8(self.type_id());
        use Value::*;
        match self {
            Null => {},
            Bool(b) => {
                w.write_u8(*b as u8);
            },
            Number(n) => {
                w.write_u64(*n);
            },
            String(s) => {
                w.write_str(s)
            }
            Array(a) => {
                w.write_u64(a.len() as u64);
                for val in a {
                    val.serialize_(w);
                }
            }
            Object(o) => {
                w.write_u64(o.len() as u64);
                for (key, val) in o {
                    w.write_str(key);
                    val.serialize_(w);
                }
            }
        }
    }

    pub fn deserialize<T: Read>(reader: T) -> Result<Self, Error> {
        let mut r = ValueReader::from_reader(reader);
        Self::deserialize_(&mut r)
    }

    fn deserialize_<T: Read>(r: &mut ValueReader<T>) -> Result<Self, Error> {
        // write type id
        let type_id = r.read_u8()?;
        use Value::*;
        let val = match type_id {
            0 => {
                Null
            }
            1 => {
                let byte = r.read_u8()?;
                if byte > 1 {
                    return Err(Error::new(ErrorKind::InvalidData, "Unexpected value for boolean byte."));
                }
                Bool(byte == 1)
            }
            2 => {
                Number(r.read_u64()?)
            }
            3 => {
                String(r.read_string()?)
            }
            4 => {
                let len = r.read_u64()? as usize;
                let mut v = Vec::with_capacity(len);
                for _ in 0..len {
                    v.push(Self::deserialize_(r)?);
                }
                Array(v)
            }
            5 => {
                let len = r.read_u64()? as usize;
                let mut m = IndexMap::with_capacity(len);
                for _ in 0..len {
                    let key = r.read_string()?;
                    let val = Self::deserialize_(r)?;
                    m.insert(key, val);
                }
                Object(m)
            }
            _ => {
                return Err(Error::new(ErrorKind::InvalidData, "Unexpected value for type id byte."));
            }
        };
        Ok(val)
    }

    pub fn pretty_print(&self) -> String {
        let mut printer = ValuePrinter::default();
        self.pretty_print_(&mut printer);
        printer.s
    }

    fn pretty_print_(&self, p: &mut ValuePrinter) {
        use Value::*;
        match self {
            Null => p.print_null(),
            Bool(b) => p.print_bool(*b),
            Number(n) => p.print_u64(*n),
            String(s) => p.print_str(s),
            Array(a) => {
                p.print('[');
                if !a.is_empty() {
                    p.indent();
                    for v in a {
                        p.print_newline();
                        v.pretty_print_(p);
                        p.print(',');
                    }
                    p.dedent();
                    p.print_newline();
                }
                p.print(']');
            }
            Object(o) => {
                p.print('{');
                if !o.is_empty() {
                    p.indent();
                    for (key, val) in o {
                        p.print_newline();
                        p.print_str(key);
                        p.print(':');
                        p.print(' ');
                        val.pretty_print_(p);
                        p.print(',');
                    }
                    p.dedent();
                    p.print_newline();
                }
                p.print('}');
            }
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        let tokens = Tokenizer::new(s).tokenize()?;
        let mut iter = tokens.into_iter().peekable();
        let res = Self::parse_(&mut iter);
        if iter.next().is_some() {
            return Err("Unexpected characters".to_string())
        }
        res
    }

    fn parse_(iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>) -> Result<Self, String> {
        match iter.next() {
            None => Err("Unexpected EOF".to_string()),
            Some(tok) => {
                Ok(match tok {
                    Token::Null => Value::Null,
                    Token::Bool(b) => Value::Bool(b),
                    Token::Num(n) => Value::Number(n),
                    Token::Str(s) => Value::String(s),
                    Token::LBracket => {
                        // parse array
                        let mut elmts = vec!();
                        if iter.peek() == Some(&Token::RBracket) {
                            iter.next();
                            return Ok(Value::Array(elmts));
                        }
                        elmts.push(Self::parse_(iter)?);
                        loop {
                            match iter.next() {
                                Some(Token::Comma) => {
                                    if iter.peek() == Some(&Token::RBracket) {
                                        iter.next();
                                        return Ok(Value::Array(elmts));
                                    } else {
                                        elmts.push(Self::parse_(iter)?);
                                    }
                                }
                                Some(Token::RBracket) => {
                                    return Ok(Value::Array(elmts));
                                }
                                Some(_) => { return Err("Unexpected token.".to_string()); }
                                None => { return Err("Unexpected EOF".to_string()); }
                            }
                        }
                    }
                    Token::LBrace => {
                        // parse object
                        let mut elmts = IndexMap::new();
                        if iter.peek() == Some(&Token::RBrace) {
                            iter.next();
                            return Ok(Value::Object(elmts));
                        }
                        // parse element
                        let key = match iter.next() {
                            Some(Token::Str(s)) => s,
                            _ => { return Err("Unexpected token.".to_string())}
                        };
                        match iter.next() {
                            Some(Token::Colon) => {},
                            _ => { return Err("Unexpected token.".to_string())}
                        };
                        elmts.insert(key, Self::parse_(iter)?);
                        loop {
                            match iter.next() {
                                Some(Token::Comma) => {
                                    if iter.peek() == Some(&Token::RBrace) {
                                        iter.next();
                                        return Ok(Value::Object(elmts));
                                    } else {
                                        // parse element
                                        let key = match iter.next() {
                                            Some(Token::Str(s)) => s,
                                            _ => { return Err("Unexpected token.".to_string())}
                                        };
                                        match iter.next() {
                                            Some(Token::Colon) => {},
                                            _ => { return Err("Unexpected token.".to_string())}
                                        };
                                        elmts.insert(key, Self::parse_(iter)?);
                                    }
                                }
                                Some(Token::RBrace) => {
                                    return Ok(Value::Object(elmts));
                                }
                                Some(_) => { return Err("Unexpected token.".to_string()); }
                                None => { return Err("Unexpected EOF".to_string()); }
                            }
                        }
                    }
                    _ => { return Err("Unexpected token!".to_string()); }
                })
            }
        }
    }

    pub fn pretty_print_2(&self) -> String {
        let mut printer = ValuePrinter::default();
        self.pretty_print_2_(&mut printer);
        printer.s
    }
    
    fn pretty_print_2_(&self, p: &mut ValuePrinter) {
        use Value::*;
        match self {
            Null => p.print_null(),
            Bool(b) => p.print_bool(*b),
            Number(n) => p.print_u64(*n),
            String(s) => p.print_str(s),
            Array(a) => {
                p.s.push_str("[]");
                if !a.is_empty() {
                    p.indent();
                    for v in a {
                        p.print_newline();
                        v.pretty_print_2_(p);
                    }
                    p.dedent();
                }
                //p.print_newline();
            }
            Object(o) => {
                p.s.push_str("{}");
                if !o.is_empty() {
                    p.indent();
                    for (key, val) in o {
                        p.print_newline();
                        p.print_str(key);
                        p.print(':');
                        p.print(' ');
                        val.pretty_print_2_(p);
                    }
                    p.dedent();
                }
                //p.print_newline();
            }
        }
    }

    pub fn parse_indented(s: &str) -> Result<Self, String> {
        let st = parse_single_tree(s)?;
        parse_str_tree(st)
    }
}

fn lines_with_indent(s: &str) -> impl Iterator<Item = (usize, &str)> {
    s.lines().map(|line| {
        let content = line.trim_start_matches(" ");
        let num_preceeding_spaces = line.len() - content.len();
        (num_preceeding_spaces, content)
    })
}

fn parse_single_tree(s: &str) -> Result<StrTree, String> {
    let mut iter = lines_with_indent(s).into_iter().peekable();
    let st = parse_one_root(&mut iter)?;
    if iter.next().is_some() {
        Err("Unexpected additional input".to_string())
    } else {
        Ok(st)
    }
}

#[allow(unused)]
fn parse_multiple_trees(s: &str) -> Result<Vec<StrTree>, String> {
    let mut iter = lines_with_indent(s).into_iter().peekable();
    let mut trees = vec!();
    while iter.peek().is_some() {
        let st = parse_one_root(&mut iter)?;
        trees.push(st);
    }
    Ok(trees)
}

fn parse_one_root<'a, I: Iterator<Item = (usize, &'a str)>>(line_iter: &mut Peekable<I>) -> Result<StrTree<'a>, String> {
    let (indent, content) = line_iter.next().ok_or("Unexpected EOF".to_string())?;
    let mut stack = vec!((indent, StrTree::new(content, vec!())));

    while let Some((indent, content)) = line_iter.peek() {
        
        // check that dedents only return to indent level that are on the stack
        let orig_top_indent = stack.last().unwrap().0;
        if *indent < orig_top_indent {
            if !stack.iter().any(|x| x.0 == *indent) {
                return Err("Dedent to a level that was skipped previously.".to_string());
            }
        }

        while *indent <= stack.last().unwrap().0 {
            // pop from the stack
            let tmp = stack.pop().unwrap();
            match stack.last_mut() {
                Some(elmt) => elmt.1.children.push(tmp.1),
                None => {
                    // we've reached a line that has the same or less indentation than the root node.
                    // return without consuming the new line
                    return Ok(tmp.1);
                }
            }
        }

        stack.push((*indent, StrTree::new(content, vec!())));
        line_iter.next();
    }

    // collapse stack
    while stack.len() > 1 {
        let tmp = stack.pop().unwrap().1;
        stack.last_mut().unwrap().1.children.push(tmp);
    }

    Ok(stack.pop().unwrap().1)
}

fn parse_line(s: &str) -> Result<Value, String> {
    let tokens = Tokenizer::new(s).tokenize()?;
    parse_line_inner(&tokens)
}

fn parse_line_inner(tokens: &[Token]) -> Result<Value, String> {
    let value = match tokens {
        [Token::LBrace, Token::RBrace] => Value::Object(IndexMap::new()),
        [Token::LBracket, Token::RBracket] => Value::Array(vec!()),
        [Token::Bool(b)] => Value::Bool(*b),
        [Token::Null] => Value::Null,
        [Token::Str(s)] => Value::String(s.clone()),
        [Token::Num(n)] => Value::Number(*n),
        _ => { return Err("Unexpected line".to_string())}
    };
    
    Ok(value)
}

fn parse_str_tree(st: StrTree) -> Result<Value, String> {
    parse_str_tree_inner(parse_line(st.elmt)?, st.children)
    
}

fn parse_str_tree_inner(line_value: Value, children: Vec<StrTree>) -> Result<Value, String> {
    let value = match line_value {
        Value::Array(_) => {
            let res: Result<Vec<_>, _> = children.into_iter().map(|s| parse_str_tree(s)).collect();
            Value::Array(res?)
        }
        Value::Object(_) => {
            let mut map = IndexMap::new();
            for child in children {
                let tokens = Tokenizer::new(child.elmt).tokenize()?;
                let key = match &tokens[..2] {
                    [Token::Str(s), Token::Colon] => {
                        s.clone()
                    }
                    _ => { return Err("Failed to parse line as an object attribute".to_string()); }
                };
                let line_value = parse_line_inner(&tokens[2..])?;
                let value = parse_str_tree_inner(line_value, child.children)?;
                map.insert(key, value);
            }
            Value::Object(map)
        }
        val => {
            if !children.is_empty() {
                return Err("This node may not have children".to_string());
            }
            val
        }
    };

    Ok(value)
}


#[test]
fn codespan_test() {
    let s = "{\n  \"a\": 12345,\n  \"b\": \"hello\"\n}";
    
    use codespan_reporting::files::SimpleFile;
    let file = SimpleFile::new("test", s);

    let diagnostic = Diagnostic::error()
        .with_message("I don't like this number")
        .with_labels(vec!(
            Label::primary((), 9..14).with_message("Expected another number")
        ));

    let writer = StandardStream::stderr(ColorChoice::Never);
    let config = codespan_reporting::term::Config::default();

    term::emit(&mut writer.lock(), &config, &file, &diagnostic).unwrap();

    assert!(false);
}

#[cfg(test)]
mod test {
    use super::*;
    use str_tree::str_tree;

    #[test]
    fn test_str_tree() {
        let s = "a\n  b\n  c\n    d\n      e\n  f";
        let exp = StrTree::new("a", vec!(
            StrTree::new("b", vec!()),
            StrTree::new("c", vec!(
                StrTree::new("d", vec!(
                    StrTree::new("e", vec!()),
                )),
            )),
            StrTree::new("f", vec!()),
        ));
        let res = parse_single_tree(s).unwrap();
        assert_eq!(exp, res)
    }

    #[test]
    fn test_parse_one_root_empty() {
        let lines = vec!();

        let mut iter = lines.into_iter().peekable();
        assert!(parse_one_root(&mut iter).is_err());
    }

    #[test]
    fn test_parse_one_root_regular() {
        let lines = vec!(
            (0, "a"),
            (1, "b"),
            (1, "c"),
            (2, "d"),
            (3, "e"),
            (1, "f"),
        );
        let exp = str_tree!(
            "a" => {
              "b",
              "c" => {
                "d" => {
                    "e",
                },
              },
              "f",
            }
        );

        let mut iter = lines.into_iter().peekable();
        assert_eq!(parse_one_root(&mut iter), Ok(exp));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_parse_two_roots_regular() {
        let lines = vec!(
            (0, "a"),
            (1, "b"),
            (2, "c"),
            (0, "d"),
            (1, "e"),
            (2, "f"),
        );
        let exp1 = str_tree!(
            "a" => {
              "b" => {
                  "c"
              },
            }
        );
        let exp2 = str_tree!(
            "d" => {
              "e" => {
                  "f"
              },
            }
        );

        let mut iter = lines.into_iter().peekable();
        assert_eq!(parse_one_root(&mut iter), Ok(exp1));
        assert_eq!(parse_one_root(&mut iter), Ok(exp2));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_non_zero_start_indent() {
        let lines = vec!(
            (1, "a"),
            (2, "b"),
            (2, "c"),
        );
        let exp = str_tree!(
            "a" => {
              "b",
              "c",
            }
        );

        let mut iter = lines.into_iter().peekable();
        assert_eq!(parse_one_root(&mut iter), Ok(exp));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_skipping_indent_levels() {
        let lines = vec!(
            (1, "a"),
            (4, "b"),
            (9, "c"),
            (9, "d"),
            (4, "e"),
        );
        let exp = str_tree!(
            "a" => {
              "b" => {
                  "c",
                  "d"
              },
              "e",
            }
        );

        let mut iter = lines.into_iter().peekable();
        assert_eq!(parse_one_root(&mut iter), Ok(exp));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_dedent_to_unknown_level() {
        let lines = vec!(
            (1, "a"),
            (4, "b"),
            (3, "c"),
        );

        let mut iter = lines.into_iter().peekable();
        assert!(dbg!(parse_one_root(&mut iter)).is_err());
    }

    #[test]
    fn test_dedent_to_unknown_level_below_initial() {
        // don't know whether this case is relevant at all, but let's just keep track of the current behavior using the test.
        let lines = vec!(
            (1, "a"),
            (4, "b"),
            (0, "c"),
        );
        
        let mut iter = lines.into_iter().peekable();
        assert!(dbg!(parse_one_root(&mut iter)).is_err());
    }

    #[test]
    fn test_parse_single_tree() {
        let res = parse_single_tree("abc\n def");
        let exp = str_tree!(
            "abc" => {
              "def"
            }
        );
        assert_eq!(res, Ok(exp));
    }

    #[test]
    fn test_parse_single_tree_additional_lines() {
        let res = parse_single_tree("abc\n def\nadditional");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_multiple_trees() {
        let res = parse_multiple_trees("abc\n def\nxyz\n    foo");
        let exp = vec!(
            str_tree!(
                "abc" => {
                  "def"
                }
            ),
            str_tree!(
                "xyz" => {
                    "foo"
                }
            ),
        );
        assert_eq!(res, Ok(exp));
    }
}