use std::{collections::HashMap, fmt, io, io::Write};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    Atom(char),
    Op(char),
    Eof,
}

struct Lexer {
    tokens: Vec<Token>,
}

impl Lexer {
    fn new(mut input: String) -> Lexer {
        let mut tokens = input
            .chars()
            .filter(|it| !it.is_ascii_whitespace())
            .map(|c| match c {
                '0'..='9' | 'a'..='z' | 'A'..='Z' => Token::Atom(c),
                _ => Token::Op(c),
            })
            .collect::<Vec<_>>();
        tokens.reverse();
        Lexer { tokens }
    }

    fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }

    fn peek(&mut self) -> Token {
        self.tokens.last().copied().unwrap_or(Token::Eof)
    }
}

enum Expression {
    Atom(String),
    Operation(char, Vec<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Atom(i) => write!(f, "{}", i),
            Expression::Operation(head, rest) => {
                write!(f, "({}", head)?;
                for s in rest {
                    write!(f, " {}", s)?
                }
                write!(f, ")")
            }
        }
    }
}

fn infix_binding_power(op: char) -> (f32, f32) {
    match op {
        '=' => (0.2, 0.1),
        '+' | '-' => (1.0, 1.1),
        '*' | '/' => (2.0, 2.1),
        '^' | '√' => (3.1, 3.0),
        '.' => (4.0, 4.1),
        _ => panic!("bad op: {:?}", op),
    }
}

fn parse_expression(lexer: &mut Lexer, min_bp: f32) -> Expression {
    let mut lhs = match lexer.next() {
        Token::Atom(it) => {
            let mut atom: String = Default::default();
            atom.push(it);
            loop {
                match lexer.peek() {
                    Token::Atom(it) => {
                        lexer.next();
                        atom.push(it);
                    }
                    Token::Op(..) => break,
                    Token::Eof => break,
                }
            }
            Expression::Atom(atom)
        }
        Token::Op('(') => {
            let lhs = parse_expression(lexer, 0.0);
            assert_eq!(lexer.next(), Token::Op(')'));
            lhs
        }
        t => panic!("bad token: {:?}", t),
    };
    loop {
        let op = match lexer.peek() {
            Token::Eof => break,
            Token::Op(')') => break,
            Token::Op(op) => op,
            t => panic!("bad token: {:?}", t),
        };
        let (l_bp, r_bp) = infix_binding_power(op);
        if l_bp < min_bp {
            break;
        }
        lexer.next();
        let rhs = parse_expression(lexer, r_bp);
        lhs = Expression::Operation(op, vec![lhs, rhs]);
    }
    lhs
}

impl Expression {
    fn from_str(input: String) -> Expression {
        let mut lexer = Lexer::new(input);
        parse_expression(&mut lexer, 0.0)
    }
    #[allow(unused)]
    fn is_asign(&self) -> Option<(String, &Expression)> {
        match self {
            Expression::Atom(_) => return None,
            Expression::Operation(c, operands) => {
                if *c == '=' {
                    let var_name = match operands.first().unwrap() {
                        Expression::Atom(c) => c.clone(),
                        _ => unreachable!(),
                    };
                    return Some((var_name, operands.last().unwrap()));
                }
                return None;
            }
        }
    }
    #[allow(unused)]
    fn eval(&self, variables: &HashMap<String, f32>) -> f32 {
        match self {
            Expression::Atom(c) => match c.parse::<f32>() {
                Ok(num) => num,
                Err(e) => *variables
                    .get(c)
                    .expect(&format!("Undefined variable {}", c)),
            },
            Expression::Operation(operator, operands) => {
                let lhs = operands.first().unwrap().eval(variables);
                let rhs = operands.last().unwrap().eval(variables);
                match operator {
                    '+' => return lhs + rhs,
                    '-' => return lhs - rhs,
                    '*' => return lhs * rhs,
                    '/' => return lhs / rhs,
                    '^' => return lhs.powf(rhs),
                    '√' => return lhs.powf(1.0 / (rhs)),
                    op => panic!("Bad operator: {}", op),
                }
            }
        }
    }
}

fn main() {
    let mut variables: HashMap<String, f32> = HashMap::new();
    loop {
        print!(">> ");
        io::stdout().flush().unwrap();
        let input = {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap();
            buf
        };
        if input.trim() == "exit" {
            break;
        }
        let expr = Expression::from_str(input);
        if let Some((var_name, lhs)) = expr.is_asign() {
            let value = lhs.eval(&variables);
            variables.insert(var_name, value);
            continue;
        }
        let value = expr.eval(&variables);
        println!("{}", value);
    }
}
