use crate::lexer::{SpannedToken, Token};
use std::mem::discriminant;

// Stmt = Statement
#[derive(Debug)]
pub enum ASTNodeStmt {
    StmtEmpty {}, // Empty statement. Essentially just `;`.
    StmtReturn { expr: ASTNodeExpr },
    StmtScope {
        content: Box<ASTNode>,
    },
}

// Decl = Declaration
#[derive(Debug)]
pub enum ASTNodeDecl {
    // All DeclGroupings are essentially equivalent to how C++ handles `private: [...]` inside a struct. Note that content must be enclosed within Brackets.
    // Note that groupings overwrite individual modifiers if applicable.
    DeclGroupingPrivate {},
    DeclGroupingDefine {},
    DeclGroupingPublic {},
    DeclGroupingRequire {},
    DeclScope{
        body: Vec<ASTNodeDecl>,
    },

    DeclStruct {
        kind: ASTNodeDeclStructKind,
        body: Box<ASTNodeDecl>,
    },
    // Enums will be done later, as they have quite unique syntax when compared to other data structures.
    /*DeclEnum {
        kind: ASTNodeDeclEnumKind,
        body: Vec<ASTNodeDecl>,
    },*/
    DeclVariable {
        ident: String,
        r#type: String,
        init_kind: ASTNodeDeclVariableInitKind,
    },
    DeclFunction {
        ident: String,
        ret: String,
        args: Vec<Box<ASTNodeDecl>>, // Args are ASTNodeDecl::DeclVariable instances.
        body: Vec<ASTNodeStmt>,          // Body can be a set of Decls and or Stmts, thus it uses ASTNodeStmt::StmtScope.
    },
}

#[derive(Debug)]
pub struct ASTNodeDeclModifiers {
    is_const: bool,
    is_static: bool,
    is_comptime: bool,
    is_private: bool,
    is_required: bool,
    is_defined: bool,
}

impl ASTNodeDeclModifiers {
    fn new() -> Self {
        Self {
            is_const: false,
            is_static: false,
            is_comptime: false,
            is_private: true, // Default is private
            is_required: false,
            is_defined: false,
        }
    }
}

#[derive(Debug)]
pub enum ASTNodeDeclVariableInitKind {
    None,                                  // E.g., `int foo;`
    DirectInit { args: Vec<ASTNodeExpr> }, // E.g., `int foo(1);`
    AssignInit { right: ASTNodeExpr },     // E.g., `int foo = 1;`
}

#[derive(Debug)]
pub enum ASTNodeDeclStructKind {
    Class,
    Struct,
}

#[derive(Debug)]
pub enum ASTNodeDeclEnumKind {
    CStyle,
    Struct,
    Class,
}

// Expr = Expression
#[derive(Debug)]
pub enum ASTNodeExpr {
    // # Atoms ---
    // Literals
    StringLit {
        str: String,
    },
    CharLit {
        chr: char,
    },
    IntLit {
        int: u64,
    },
    FloatLit {
        flt: f64,
    },

    Grouping {
        expr: Box<ASTNodeExpr>,
    },

    // # Functions ---
    // Operator Functions
    FnOpBinary {
        kind: ASTNodeExprBinaryOp,
        left: Box<ASTNodeExpr>,
        right: Box<ASTNodeExpr>,
    },
    FnOpUnary {
        kind: ASTNodeExprUnaryOp,
        operand: Box<ASTNodeExpr>,
    },
}

#[derive(Debug)]
pub enum ASTNodeExprUnaryOp {}

#[derive(Debug)]
pub enum ASTNodeExprBinaryOp {
    OpPlus,
}

#[derive(Debug)]
pub enum ASTNode {
    Stmt { stmt: ASTNodeStmt },
    Decl { decl: ASTNodeDecl },
    Expr { expr: ASTNodeExpr },
}

pub struct Parser {
    tokens: Vec<SpannedToken>,
    symbol_table: Vec<String>,
    cursor: usize, // Current position inside 'tokens'
    peek_stack: Vec<usize>,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Parser {
        Parser {
            tokens,
            symbol_table: Vec::new(),
            cursor: 0,
            peek_stack: Vec::new(),
        }
    }

    // Parse Pass 1
    fn parse_p1(&mut self) {
        self.push_();
        loop {
            let next = self.peek_one(None);
            match next.token {
                Token::TkClass => {
                    let tk = self.peek_one(Some(Token::TkIdentLiteral {
                        name: String::new(),
                    }));
                    let ident: String;
                    if let Token::TkIdentLiteral { name } = tk.token {
                        ident = name
                    } else {
                        panic!("Expected TkIntLiteral!!!"); // Should be unreachable.
                    }

                    self.symbol_table.push(ident);
                }
                Token::TkStruct => {
                    let tk = self.peek_one(Some(Token::TkIdentLiteral {
                        name: String::new(),
                    }));
                    let mut ident: String;
                    if let Token::TkIdentLiteral { name } = tk.token {
                        ident = name
                    } else {
                        panic!("Expected TkIntLiteral!!!"); // Should be unreachable.
                    }

                    self.symbol_table.push(ident);
                }
                Token::TkEnum => {
                    let tk = self.preview_one();
                    match tk.token {
                        Token::TkIdentLiteral { name } => {
                            self.symbol_table.push(name);
                        }
                        _ => {}
                    }
                }
                Token::TkEoF => {
                    break;
                }
                _ => {}
            }
        }

        self.cursor = 0;
        self.peek_stack.clear();
    }

    pub fn parse(&mut self) -> Vec<ASTNode> {
        self.parse_p1();
        let mut out: Vec<ASTNode> = Vec::new();

        // For now just parse this APEX `{return <!left:IntLit> + <!right:IntLit>};`
        while self.cursor < self.tokens.len() {
            self.push_();
            match &self.preview_one().token {
                Token::TkEoF => {
                    break;
                }
                tk => match tk {
                    Token::TkPub => {
                        let decl = ASTNode::Decl {
                            decl: self.consume_decl(),
                        };
                        out.push(decl);
                    }
                    Token::TkConst => {
                        let decl = ASTNode::Decl {
                            decl: self.consume_decl(),
                        };
                        out.push(decl);
                    }
                    Token::TkStatic => {
                        let decl = ASTNode::Decl {
                            decl: self.consume_decl(),
                        };
                        out.push(decl);
                    }
                    Token::TkCompTime => {
                        let decl = ASTNode::Decl {
                            decl: self.consume_decl(),
                        };
                        out.push(decl);
                    }
                    Token::TkClass => {
                        let decl = ASTNode::Decl {
                            decl: self.consume_decl(),
                        };
                        out.push(decl);
                    }
                    Token::TkStruct => {
                        let decl = ASTNode::Decl {
                            decl: self.consume_decl(),
                        };
                        out.push(decl);
                    }
                    Token::TkEnum => {
                        let decl = ASTNode::Decl {
                            decl: self.consume_decl(),
                        };
                        out.push(decl);
                    }
                    Token::TkIdentLiteral { name } => {
                        let ident = name;
                        if self.symbol_table.contains(ident) {
                            let decl = ASTNode::Decl {
                                decl: self.consume_decl(),
                            };
                            out.push(decl);
                        } else {
                            let stmt = ASTNode::Stmt {
                                stmt: self.consume_stmt(),
                            };
                            out.push(stmt);
                        }
                    }
                    _ => {
                        panic!("Found unhandled token type: `{:?}` in `parse()`.", tk);
                    }
                },
            }
            self.apply_();
        }

        out
    }

    // consumption functions
    // Note that the cursor will always be in front of the first token to be parsed, not on top or after it.

    fn consume_decl(&mut self) -> ASTNodeDecl {
        #[derive(Debug)]
        enum Kind {
            Enum,
            Class,
            Struct,
            EnumClass,
            EnumStruct,
            Undefined,
            Function,
            Variable,
        }

        let mut modifiers = ASTNodeDeclModifiers::new();
        let mut out: ASTNodeDecl;
        let mut ident: String;
        let mut r#type: String;
        let mut t: SpannedToken;
        let mut kind = Kind::Undefined;

        loop {
            t = self.peek_one(None);
            match t.token {
                Token::TkPub => modifiers.is_private = false,
                Token::TkCompTime => modifiers.is_comptime = true,
                Token::TkStatic => modifiers.is_static = true,
                Token::TkConst => modifiers.is_const = true,
                Token::TkIdentLiteral { name } => {
                    if self.symbol_table.contains(&name) {
                        r#type = name;
                    } else {
                        ident = name; // There can only be one identifier, and it must be at the end of the declaration before either the body or the argument list.
                        break;
                    }
                }
                Token::TkEnum => match kind {
                    Kind::Undefined => kind = Kind::Enum,
                    _ => panic!(
                        "Encountered `enum` declaration after specifying the declaration to be of type: {:?}",
                        kind
                    ),
                },
                Token::TkClass => match kind {
                    Kind::Undefined => kind = Kind::Class,
                    Kind::Enum => kind = Kind::EnumClass,
                    _ => panic!(
                        "Encountered `class` declaration after specifying the declaration to be of type: {:?}",
                        kind
                    ),
                },
                Token::TkStruct => match kind {
                    Kind::Undefined => kind = Kind::Struct,
                    Kind::Enum => kind = Kind::EnumStruct,
                    _ => panic!(
                        "Encountered `struct` declaration after specifying the declaration to be of type: {:?}",
                        kind
                    ),
                },
                _ => {
                    panic!(
                        "Found unhandled token `{:?}`, whilst trying to consume decl.",
                        t
                    );
                }
            }
        }

        if let Kind::Undefined = kind {
            // To be valid, the next valid tokens are either:
            // Token::TkMiscParenL, e.g., for `int foo(1);` or `int add(int a, int b);`, or
            // Token::TkOpAsign, as in e.g., `int foo = 1;`
            self.push_(); // Safe the current state, as to recover it after discovering what this declaration is.
            let mut depth = 1;
            loop {
                let p = self.peek_one(None);
                match p.token {
                    Token::TkMiscParenL => depth += 1,
                    Token::TkMiscParenR => depth -= 1,
                    _ => {}
                }
                if depth == 0 {
                    break;
                }
            }
            let p = self.peek_one(None);
            match p.token {
                Token::TkMiscBraceL => kind = Kind::Function,
                Token::TkOpAssign => kind = Kind::Variable,
                Token::TkMiscSemi => kind = Kind::Variable,
                _ => panic!(
                    "Found invalid declaration. Expected either `{{` or `;` token, but found: `{:?}` instead.",
                    self.tokens
                ),
            }
            self.discard_(); // Recover the state before the search.
        }

        match kind {
            k => panic!("Encountered unhandled declaration kind: {:?}", k),
        }

        out
    }

    fn consume_stmt(&mut self) -> ASTNodeStmt {
        self.push_();
        let next = self.peek_one_safe(None);
        let mut stmt: ASTNodeStmt = ASTNodeStmt::StmtEmpty {};
        match next {
            Some(stk) => match stk.token {
                Token::TkReturn => {
                    let expr = self.consume_expr();
                    stmt = ASTNodeStmt::StmtReturn { expr };
                }
                _ => {
                    panic!(
                        "Found unhandled token: {:?} in statement consumption!!!",
                        stk
                    );
                }
            },
            None => {
                // Do nothing. `;` is a technically valid statement.
                // Upon further looks, this seems unreachable, or rather if it was reached, the peek directly after will fail and panic.
            }
        }

        self.peek_one(Some(Token::TkMiscSemi));
        self.apply_();

        stmt
    }

    // low = Low Priority
    fn consume_arithmetic_low(&mut self) -> ASTNodeExpr {
        self.push_();

        let mut left = self.consume_atom();
        loop {
            self.push_();
            let next = self.peek_one_safe(None);
            match next {
                None => {
                    self.discard_();
                    break;
                }
                Some(op) => match op.token {
                    Token::TkOpPlus => {
                        self.apply_();
                        let right = self.consume_atom();
                        left = ASTNodeExpr::FnOpBinary {
                            kind: ASTNodeExprBinaryOp::OpPlus,
                            left: Box::from(left),
                            right: Box::from(right),
                        };
                    }
                    _ => {
                        self.discard_();
                        break;
                    }
                },
            }
        }
        self.apply_();
        left
    }

    fn consume_atom(&mut self) -> ASTNodeExpr {
        self.push_();
        let t = self.peek_one(None);
        match t.token {
            Token::TkIntLiteral { v } => {
                self.apply_();
                ASTNodeExpr::IntLit { int: v }
            }
            Token::TkMiscParenL => {
                let content = self.consume_expr();
                self.peek_one(Some(Token::TkMiscParenR));

                self.apply_();

                ASTNodeExpr::Grouping {
                    expr: Box::from(content),
                }
            }
            _ => {
                panic!("Found token: {:?}, which is not an Atom!", t);
            }
        }
    }

    // A DeclScope is limited compared to a Stmt scope, in that it cannot contain executable code, but only other Decls.
    fn consume_decl_scope(&mut self) -> ASTNodeDecl {
        let depth = 1;
        self.peek_one(Some(Token::TkMiscBraceL));
        let v: Vec<ASTNodeDecl> = Vec::new();
        loop {
            let p = self.preview_one();
            match p.token {
                Token::TkStruct =>
                _ => panic!("Found unhandled Token: {:?}, whilst trying to consume DeclScope.", p);
            }
        }
    }

    // Utility only. Links to the lowest precedence function.
    fn consume_expr(&mut self) -> ASTNodeExpr {
        self.consume_arithmetic_low()
    }

    fn peek_one(&mut self, expect: Option<Token>) -> SpannedToken {
        match expect {
            Some(expect) => {
                let (v, complete) = self.peek_(0, 1, false, Some(vec![expect]));
                if !complete {
                    eprintln!("Tried to peek out of bounds.");
                    panic!();
                }
                v[0].clone()
            }
            None => {
                let (v, complete) = self.peek_(0, 1, false, None);
                if !complete {
                    eprintln!("Tried to peek out of bounds.");
                    panic!();
                }
                v[0].clone()
            }
        }
    }

    fn preview_one(&mut self) -> SpannedToken {
        self.push_();
        let (v, complete) = self.peek_(0, 1, false, None);
        if !complete {
            eprintln!("Tried to peek out of bounds.");
            panic!();
        }
        self.discard_();
        v[0].clone()
    }

    fn peek_one_safe(&mut self, expect: Option<Token>) -> Option<SpannedToken> {
        match expect {
            Some(expect) => {
                let (v, complete) = self.peek_(0, 1, false, Some(vec![expect]));
                if !complete { None } else { Some(v[0].clone()) }
            }
            None => {
                let (v, complete) = self.peek_(0, 1, false, None);
                if !complete { None } else { Some(v[0].clone()) }
            }
        }
    }

    // Peek stack operations
    fn push_(&mut self) {
        if self.peek_stack.is_empty() {
            self.peek_stack.push(self.cursor);
        } else {
            let head = self.peek_stack.len() - 1;
            self.peek_stack.push(self.peek_stack[head]);
        }
    }

    fn apply_(&mut self) {
        if self.peek_stack.is_empty() {
            panic!("Tried to apply an empty peek stack.");
        } else if self.peek_stack.len() == 1 {
            self.cursor = self.peek_stack.pop().unwrap(); // Shouldn't fail.
        } else {
            let v = self.peek_stack.pop().unwrap(); // Shouldn't fail either.
            let head = self.peek_stack.len() - 1;
            self.peek_stack[head] = v;
        }
    }

    fn discard_(&mut self) {
        self.peek_stack.pop();
    }

    fn stack_advance_(&mut self, amount: usize) {
        if !self.peek_stack.is_empty() {
            let head = self.stack_head_pos_();
            self.peek_stack[head] += amount;
        } else {
            panic!("Tried to advance empty stack.");
        }
    }

    fn stack_retreat_(&mut self, amount: usize) {
        let head = self.stack_head_pos_();
        self.peek_stack[head] -= amount;
    }

    fn stack_head_pos_(&self) -> usize {
        if self.peek_stack.is_empty() {
            panic!("Tried to get head position of empty stack.");
        } else {
            self.peek_stack.len().saturating_sub(1)
        }
    }

    fn stack_head_(&self) -> usize {
        if self.peek_stack.is_empty() {
            0
        } else {
            let head = self.peek_stack.len().saturating_sub(1);
            self.peek_stack[head]
        }
    }

    // Full peek function, all options
    fn peek_(
        &mut self,
        offset: usize,
        length: usize,
        push_to_stack: bool,
        expect: Option<Vec<Token>>,
    ) -> (Vec<SpannedToken>, bool) {
        if push_to_stack {
            self.push_();
        }

        let start = offset.saturating_add(self.stack_head_());

        let requested_end = start.saturating_add(length);
        let end = requested_end.min(self.tokens.len());
        let complete = requested_end <= self.tokens.len();

        let tokens = self.tokens[start..end].to_vec();

        if let Some(expected) = expect {
            for (actual, expected) in tokens.iter().zip(expected.iter()) {
                if discriminant(&actual.token) != discriminant(expected) {
                    eprintln!(
                        "Expected token: {:?}, but instead found: {:?}.",
                        expected, actual.token
                    );
                }
            }
        }

        self.stack_advance_(tokens.len());

        (tokens, complete)
    }
}
