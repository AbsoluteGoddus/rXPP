use std::num::{ParseFloatError, ParseIntError};
use colored::Colorize;

#[derive(Debug, Clone)]
pub enum LexerError {
    ErrorUnhandledChar,
    ErrorFailedToParseFloat { err: ParseFloatError },
    ErrorFailedToParseInt { err: ParseIntError },
    ErrorIncompleteStringLiteral,
    ErrorIncompleteCharLiteral,
}

#[derive(Debug, Clone)]
pub enum Token {
    // Literals
    TkStringLiteral { content: String },
    TkCharLiteral { c: char },
    TkIntLiteral { v: u64 },
    TkFloatLiteral { v: f64 },
    TkIdentLiteral { name: String },

    // Keywords
    TkLet,    // let x = 1;
    TkReturn, // return 0;

    TkStruct, // struct
    TkClass,  // class

    TkEnum,   // enum
    TkExtend, // extend
    TkBy,     // by in "extend ... by ... {}"

    TkUse, // use ...
    TkAs,  // as ...

    TkImport, // import ... [as ...]

    TkGoto,   // goto ...
    TkSwitch, // switch (param) {...}

    TkBreak,    // for (...) { break; }
    TkContinue, // for (...) { continue; }

    TkModule, // module ... { ... }
    TkNamespace, // namespace ... { ... }

    TkDefine, // "define ... as ..." (Macro Definition) or inside extensions: "define foo(...) {...}", for example.
    TkExtern, // extern "C" {...}

    TkConst,  // const ...
    TkStatic, // static ...
    TkCompTime, // comptime ...

    TkPublic,  // `public: [...]`
    TkPrivate, // `private: [...]`

    TkPub, // `pub decl [...]`

    TkRequire, // `require decl [...]`
    /* TkProvide, // `provide decl [...]` */ // Replaced in favor of having any freestanding function inside an extension automatically act as being "provided".

    // Operators
    TkOpMinus, // -
    TkOpPlus,  // +
    TkOpMod,   // %
    TkOpMul,   // *
    TkOpDiv,   // /

    TkOpComma, // ,

    TkOpColon, // :

    TkOpAssign,   // =
    TkOpIncrease, // +=
    TkOpDecrease, // -=
    TkOpMulEq,    // *=
    TkOpDivEq,    // /=

    TkOpHash, // #
    TkOpSeq,  // ...

    TkOpDot,      // .
    TkOpArrow,    // ->
    TkOpFatArrow, // =>
    TkOpNeg,      // !

    TkOpAnd, // &&
    TkOpOr,  // ||

    TkOpBinAnd, // &
    TkOpBinOr,  // |
    TkOpBinXor, // ^
    TkOpBinNot, // ~

    // Comparators
    TkCompEqual,  // ==
    TkCompNEqual, // !=
    TkCompLess,   // <
    TkCompGreat,  // >
    TkCompLEqual, // <=
    TkCompGEqual, // >=

    // Misc
    TkMiscSemi,  // ;
    TkMiscScope, // ::

    TkMiscParenL, // (
    TkMiscParenR, // )

    TkMiscBraceL, // {
    TkMiscBraceR, // }

    TkMiscSquareL, // [
    TkMiscSquareR, // ]

    TkEoF, // End of File.

    TkError { c: char, err: LexerError }, // If any error happened, it will be propagated with this. 'c' is the char that the lexer failed on, and err is the error data.

    // Misc (Not yet used but reserved)
    TkMiscDollar,   // $
    TkMiscAt,       // @
    TkMiscQuestion, // ?
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    tk_begin: usize,
    tk_end: usize,
    pub token: Token,
}

#[derive(Debug)]
pub struct Lexer {
    pub source: String,
    ptr: usize,
    peek_stack: usize, // Number of characters peeked ahead of `ptr`, not yet committed
}

impl Lexer {
    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut out: Vec<SpannedToken> = Vec::new();
        while self.ptr < self.source.len() {
            self.discard(); // safety — ensure clean state each iteration
            match self.current() {
                '\'' => {
                    self.tokenize_string_or_char_literal(&mut out);
                }
                '"' => {
                    self.tokenize_string_or_char_literal(&mut out);
                },
                '.' => {
                    if self.can_peek_n(Some(2)) && self.peek_n(Some(2)) == ".." {
                        self.push_apply_consume(&mut out, Token::TkOpSeq);
                    } else if self.discard() && self.can_peek() && self.peek().is_digit(10) {
                        self.discard();
                        self.tokenize_number_literal(&mut out);
                    } else {
                        self.discard_push_consume(&mut out, Token::TkOpDot);
                    }
                }
                '|' => {
                    if self.can_peek() && self.peek() == '|' {
                        self.push_apply_consume(&mut out, Token::TkOpOr);
                    } else {
                        self.discard_push_consume(&mut out, Token::TkOpBinOr);
                    }
                }
                '&' => {
                    if self.can_peek() && self.peek() == '&' {
                        self.push_apply_consume(&mut out, Token::TkOpAnd);
                    } else {
                        self.discard_push_consume(&mut out, Token::TkOpBinAnd);
                    }
                }
                '^' => {
                    out.push(self.new_token(Token::TkOpBinXor));
                    self.consume();
                }
                '~' => {
                    out.push(self.new_token(Token::TkOpBinNot));
                    self.consume();
                }
                '?' => {
                    out.push(self.new_token(Token::TkMiscQuestion));
                    self.consume();
                }
                '$' => {
                    out.push(self.new_token(Token::TkMiscDollar));
                    self.consume();
                }
                '@' => {
                    out.push(self.new_token(Token::TkMiscAt));
                    self.consume();
                }
                '(' => {
                    out.push(self.new_token(Token::TkMiscParenL));
                    self.consume();
                }
                ')' => {
                    out.push(self.new_token(Token::TkMiscParenR));
                    self.consume();
                }
                '{' => {
                    out.push(self.new_token(Token::TkMiscBraceL));
                    self.consume();
                }
                '}' => {
                    out.push(self.new_token(Token::TkMiscBraceR));
                    self.consume();
                }
                '[' => {
                    out.push(self.new_token(Token::TkMiscSquareL));
                    self.consume();
                }
                ']' => {
                    out.push(self.new_token(Token::TkMiscSquareR));
                    self.consume();
                }
                ';' => {
                    out.push(self.new_token(Token::TkMiscSemi));
                    self.consume();
                }
                '#' => {
                    out.push(self.new_token(Token::TkOpHash));
                    self.consume();
                }
                '+' => {
                    self.discard_push_consume(&mut out, Token::TkOpPlus);
                }
                '-' => {
                    let peeked = if self.can_peek() { self.peek() } else { '\0' };
                    if peeked == '>' {
                        self.push_apply_consume(&mut out, Token::TkOpArrow);
                    } else {
                        self.discard_push_consume(&mut out, Token::TkOpMinus);
                    }
                }
                '!' => {
                    if self.can_peek() && self.peek() == '=' {
                        self.push_apply_consume(&mut out, Token::TkCompNEqual);
                    } else {
                        self.discard_push_consume(&mut out, Token::TkOpNeg);
                    }
                }
                '*' => {
                    self.discard_push_consume(&mut out, Token::TkOpMul);
                }
                '/' => {
                    if self.can_peek() {
                        match self.peek() {
                            '/' => self.consume_single_line_comment(),
                            '*' => self.consume_multi_line_comment(),
                            _ => self.discard_push_consume(&mut out, Token::TkOpDiv),
                        }
                    } else {
                        self.discard_push_consume(&mut out, Token::TkOpDiv);
                    }
                }
                '%' => {
                    out.push(self.new_token(Token::TkOpMod));
                    self.consume();
                }
                ',' => {
                    out.push(self.new_token(Token::TkOpComma));
                    self.consume();
                }
                '=' => {
                    if self.can_peek() {
                        match self.peek() {
                            '>' => self.push_apply_consume(&mut out, Token::TkOpFatArrow),
                            '=' => self.push_apply_consume(&mut out, Token::TkCompEqual),
                            _ => self.discard_push_consume(&mut out, Token::TkOpAssign),
                        }
                    } else {
                        self.discard_push_consume(&mut out, Token::TkOpAssign);
                    }
                }
                ':' => {
                    if self.can_peek() && self.peek() == ':' {
                        self.push_apply_consume(&mut out, Token::TkMiscScope);
                    } else {
                        self.discard_push_consume(&mut out, Token::TkOpColon);
                    }
                }
                '<' => {
                    if self.can_peek() {
                        match self.peek() {
                            '=' => self.push_apply_consume(&mut out, Token::TkCompLEqual),
                            _ => self.discard_push_consume(&mut out, Token::TkCompLess),
                        }
                    } else {
                        self.discard_push_consume(&mut out, Token::TkCompLess);
                    }
                }
                '>' => {
                    if self.can_peek() {
                        match self.peek() {
                            '=' => self.push_apply_consume(&mut out, Token::TkCompGEqual),
                            _ => self.discard_push_consume(&mut out, Token::TkCompGreat),
                        }
                    } else {
                        self.discard_push_consume(&mut out, Token::TkCompGreat);
                    }
                }
                c if c.is_whitespace() => {
                    self.consume();
                }
                c if c.is_alphabetic() || c == '_' => {
                    self.tokenize_ident_or_keyword(&mut out);
                }
                c if c.is_digit(10) => {
                    self.tokenize_number_literal(&mut out);
                }
                c => {
                    println!(
                        "[{}] {}: '{}'. Current Lexer State: '{}'",
                        "ERROR".bold().red(),
                        "Encountered unhandled char",
                        c,
                        self.dump_state()
                    );
                    out.push(SpannedToken {
                        tk_begin: self.ptr,
                        tk_end: self.ptr + 1,
                        token: Token::TkError {
                            c,
                            err: LexerError::ErrorUnhandledChar,
                        },
                    });
                    self.consume();
                }
            }
        }

        out.push(SpannedToken {
            tk_begin: self.ptr,
            tk_end: self.ptr,
            token: Token::TkEoF,
        });

        out
    }

    fn consume_single_line_comment(&mut self) {
        self.discard();
        self.consume(); // consume the first '/'
        while !self.reached_end() && self.current() != '\n' {
            self.consume();
        }
    }

    // No errors for unterminated comments.
    fn consume_multi_line_comment(&mut self) {
        self.discard(); // reset peek_stack
        self.consume(); // consume the '/'
        self.consume(); // consume the '*'

        while !self.reached_end() {
            if self.current() == '*' && self.can_peek() && self.peek() == '/' {
                self.discard();
                self.consume(); // consume '*'
                self.consume(); // consume '/'
                return;
            }
            self.discard();
            self.consume();
        }
        // unterminated comment — could push error here
    }

    fn push_apply_consume(&mut self, v: &mut Vec<SpannedToken>, token: Token) {
        v.push(self.new_token(token));
        self.apply();
        self.consume();
    }

    fn discard_push_consume(&mut self, v: &mut Vec<SpannedToken>, token: Token) {
        self.discard();
        v.push(self.new_token(token));
        self.consume();
    }

    fn new_token(&self, token: Token) -> SpannedToken {
        SpannedToken {
            tk_begin: self.ptr,
            tk_end: self.ptr + self.peek_stack + 1,
            token,
        }
    }

    fn tokenize_number_literal(&mut self, v: &mut Vec<SpannedToken>) {
        let begin = self.ptr;
        let mut digit = String::new();
        let mut is_float = false;

        // consume current char first
        let first = self.current();
        if first == '.' {
            is_float = true;
        }
        digit.push(first);
        self.consume();

        loop {
            if self.reached_end() {
                break;
            }
            match self.current() {
                c if c.is_digit(10) => {
                    digit.push(c);
                    self.consume();
                }
                '.' => {
                    let next = self.source.chars().nth(self.ptr + 1);
                    match next {
                        Some(c) if c.is_digit(10) => {
                            // genuine decimal point — continue as float
                            is_float = true;
                            digit.push('.');
                            self.consume();
                        }
                        Some('.') => {
                            // this is the start of ".." or "..." — not a decimal point, stop here
                            break;
                        }
                        _ => {
                            // trailing dot, e.g. "5." — consume it and stop
                            is_float = true;
                            digit.push('.');
                            self.consume();
                            break;
                        }
                    }
                }
                ',' => {
                    self.consume(); // separator, skip
                }
                _ => break,
            }
        }

        let token = if is_float {
            match digit.parse::<f64>() {
                Ok(f) => Token::TkFloatLiteral { v: f },
                Err(e) => Token::TkError {
                    c: '\0',
                    err: LexerError::ErrorFailedToParseFloat { err: e },
                },
            }
        } else {
            match digit.parse::<u64>() {
                Ok(n) => Token::TkIntLiteral { v: n },
                Err(e) => Token::TkError {
                    c: '\0',
                    err: LexerError::ErrorFailedToParseInt { err: e },
                },
            }
        };

        v.push(SpannedToken {
            tk_begin: begin,
            tk_end: self.ptr,
            token,
        });
    }

    fn tokenize_string_or_char_literal(&mut self, v: &mut Vec<SpannedToken>) {
        if self.current() == '"' {
            // string branch
            let begin = self.ptr;
            self.consume(); // opening "
            let mut str = String::new();

            while !self.reached_end() && self.current() != '"' {
                if self.current() == '\\' {
                    self.consume(); // consume backslash
                    if self.reached_end() {
                        break;
                    }
                    let escaped = match self.current() {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '"' => '"',
                        '\\' => '\\',
                        c => c,
                    };
                    str.push(escaped);
                    self.consume();
                } else {
                    str.push(self.current());
                    self.consume();
                }
            }

            if self.reached_end() {
                v.push(SpannedToken {
                    tk_begin: begin,
                    tk_end: self.ptr,
                    token: Token::TkError {
                        c: '\0',
                        err: LexerError::ErrorIncompleteStringLiteral,
                    },
                });
            } else {
                self.consume(); // closing "
                v.push(SpannedToken {
                    tk_begin: begin,
                    tk_end: self.ptr,
                    token: Token::TkStringLiteral { content: str },
                });
            }
        } else {
            // char branch
            let begin = self.ptr;
            self.consume(); // opening '

            if self.reached_end() {
                v.push(SpannedToken {
                    tk_begin: begin,
                    tk_end: self.ptr,
                    token: Token::TkError {
                        c: '\0',
                        err: LexerError::ErrorIncompleteCharLiteral,
                    },
                });
                return;
            }

            let c = if self.current() == '\\' {
                self.consume(); // backslash
                if self.reached_end() {
                    v.push(SpannedToken {
                        tk_begin: begin,
                        tk_end: self.ptr,
                        token: Token::TkError {
                            c: '\0',
                            err: LexerError::ErrorIncompleteCharLiteral,
                        },
                    });
                    return;
                }
                // I will not bother to implement '\nnn', '\xhh...', '\uhhhh' or '\Uhhhhhhhh'.
                let escaped = match self.current() {
                    'a' => 0x07 as char,
                    'b' => 0x08 as char,
                    'e' => 0x1B as char,
                    'f' => 0x0C as char,
                    'n' => '\n',
                    't' => '\t',
                    'v' => 0x0B as char,
                    '0' => '\0',
                    'r' => '\r',
                    '\'' => '\'',
                    '\\' => '\\',
                    other => other,
                };
                self.consume(); // the escape char
                escaped
            } else {
                let ch = self.current();
                self.consume(); // the char itself
                ch
            };

            if self.reached_end() || self.current() != '\'' {
                v.push(SpannedToken {
                    tk_begin: begin,
                    tk_end: self.ptr,
                    token: Token::TkError {
                        c: '\0',
                        err: LexerError::ErrorIncompleteCharLiteral,
                    },
                });
            } else {
                self.consume(); // closing '
                v.push(SpannedToken {
                    tk_begin: begin,
                    tk_end: self.ptr,
                    token: Token::TkCharLiteral { c },
                });
            }
        }
    }

    fn tokenize_ident_or_keyword(&mut self, v: &mut Vec<SpannedToken>) {
        let begin = self.ptr;

        let mut str = String::new();

        while !self.reached_end() && (self.current().is_alphanumeric() || self.current() == '_') {
            str.push(self.consume());
        }

        let tk = SpannedToken {
            tk_begin: begin,
            tk_end: self.ptr,
            token: match str.as_str() {
                "let" => Token::TkLet,
                "return" => Token::TkReturn,
                "struct" => Token::TkStruct,
                "class" => Token::TkClass,
                "enum" => Token::TkEnum,
                "extend" => Token::TkExtend,
                "by" => Token::TkBy,
                "as" => Token::TkAs,
                "use" => Token::TkUse,
                "import" => Token::TkImport,
                "goto" => Token::TkGoto,
                "switch" => Token::TkSwitch,
                "break" => Token::TkBreak,
                "continue" => Token::TkContinue,
                "define" => Token::TkDefine,
                "extern" => Token::TkExtern,
                "module" => Token::TkModule,
                "namespace" => Token::TkNamespace,
                "const" => Token::TkConst,
                "static" => Token::TkStatic,
                "comptime" => Token::TkCompTime,
                "private" => Token::TkPrivate,
                "public" => Token::TkPublic,
                "pub" => Token::TkPub,
                "require" => Token::TkRequire,
                _ => Token::TkIdentLiteral { name: str },
            },
        };

        v.push(tk);
    }

    fn reached_end(&self) -> bool {
        self.ptr >= self.source.len()
    }

    // NOTE: peek_stack represents "how many characters beyond the NEXT one
    // (i.e. beyond `ptr`) have already been examined". The first peek always
    // looks at `ptr + 1` — one past `current()` — not at `ptr` itself.
    fn can_peek_n(&self, len: Option<usize>) -> bool {
        let len = len.unwrap_or(1);
        self.ptr + self.peek_stack + 1 + len <= self.source.len()
    }

    fn can_peek(&self) -> bool {
        self.can_peek_n(Some(1))
    }

    fn current(&self) -> char {
        self.source.chars().nth(self.ptr).unwrap()
    }

    fn peek_n(&mut self, len: Option<usize>) -> &str {
        let len = len.unwrap_or(1);
        let start = self.ptr + self.peek_stack + 1; // +1 — look PAST current char
        let slice = &self.source[start..start + len];
        self.peek_stack += len;

        slice
    }

    fn peek(&mut self) -> char {
        self.peek_n(Some(1)).chars().nth(0).unwrap()
    }

    fn apply(&mut self) {
        self.ptr += self.peek_stack;
        self.peek_stack = 0;
    }

    fn discard(&mut self) -> bool {
        self.peek_stack = 0;
        true
    }

    fn consume(&mut self) -> char {
        self.ptr += 1;
        self.discard();
        self.source.as_bytes()[self.ptr - 1] as char
    }

    fn dump_state(&self) -> colored::ColoredString {
        format!(
            "{} the dump occurred around this region: {}",
            format!("{:#?}", self).blue(),
            self.source[(self.ptr.saturating_sub(10))..(self.ptr + 10).min(self.source.len())]
                .yellow()
        )
            .into()
    }

    pub fn new() -> Self {
        Self {
            source: String::from(""),
            ptr: 0,
            peek_stack: 0,
        }
    }
}