use indexmap::IndexMap;
use std::io::{Error, ErrorKind, Read};

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
    iter: std::iter::Peekable<std::str::Chars<'a>>,
    tokens: Vec<Token>
}

impl<'a> Tokenizer<'a> {
    fn new(s: &'a str) -> Self {
        Tokenizer {
            iter: s.chars().peekable(),
            tokens: vec!()
        }
    }

    fn consume(&mut self, c: char) -> Result<(), String> {
        match self.iter.next() {
            None => Err("Unexpected end of file".to_string()),
            Some(x) if x == c => Ok(()),
            Some(other) => Err(format!("Unexpected character '{}'.", other))
        }
    }

    fn consume_str(&mut self, s: &str) -> Result<(), String> {
        for c in s.chars() {
            self.consume(c)?;
        }
        Ok(())
    }

    fn tokenize(mut self) -> Result<Vec<Token>, String> {
        loop {
            match self.iter.peek() {
                None => { break },
                Some(c) if c.is_ascii_whitespace() => {  // skip whitespace
                    self.iter.next();
                },
                Some('[') => {
                    self.tokens.push(Token::LBracket);
                    self.iter.next();
                }
                Some(']') => {
                    self.tokens.push(Token::RBracket);
                    self.iter.next();
                }
                Some('{') => {
                    self.tokens.push(Token::LBrace);
                    self.iter.next();
                }
                Some('}') => {
                    self.tokens.push(Token::RBrace);
                    self.iter.next();
                }
                Some(':') => {
                    self.tokens.push(Token::Colon);
                    self.iter.next();
                }
                Some(',') => {
                    self.tokens.push(Token::Comma);
                    self.iter.next();
                }
                Some('t') => {
                    self.consume_str("true")?;
                    self.tokens.push(Token::Bool(true));
                }
                Some('f') => {
                    self.consume_str("false")?;
                    self.tokens.push(Token::Bool(false));
                }
                Some('n') => {
                    self.consume_str("null")?;
                    self.tokens.push(Token::Null);
                }
                Some('"') => {
                    self.consume('"')?;
                    let mut s = String::new();
                    let mut escape = false;
                    loop {
                        // handle string escapes
                        match (self.iter.next(), escape) {
                            (None, _) => { return Err("Unexpected EOF".to_string()) }
                            (Some('\\'), false) => { escape = true; }
                            (Some(c), true) => {
                                escape = false;
                                let c = match c {
                                    't' => '\t',
                                    'n' => '\n',
                                    'r' => '\r',
                                    c => c,
                                };
                                s.push(c);
                            }
                            (Some('"'), false) => {
                                break;
                            }
                            (Some(c), false) => {
                                s.push(c);
                            }
                        }
                    }
                    self.tokens.push(Token::Str(s));
                }
                Some(c) if c.is_numeric() => {
                    let mut s = String::new();
                    loop {
                        match self.iter.peek().cloned() {
                            Some(c) if c.is_numeric() => {
                                self.iter.next();
                                s.push(c);
                            }
                            _ => {
                                break;
                            }
                        }
                    }
                    self.tokens.push(Token::Num(s.parse().map_err(|_| "Number out of range".to_string())?))
                }
                Some(c) => {
                    return Err(format!("Unexpected input `{}`", c))
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
                                Some(Token::RBracket) => {
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
}
