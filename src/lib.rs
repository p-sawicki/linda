use std::str::Chars;

#[derive(PartialEq, Debug)]
enum Value {
    Int(i32),
    Float(f64),
    String(String),
}
type Tuple = Vec<Value>;

struct Parser<'a> {
    curr: Option<char>,
    it: Chars<'a>,
}

const NO_OPENING_PARENTHESIS: &str = "Tuple needs to start with opening parenthesis ('(')!";
const ERROR_PARSING_TUPLE: &str = "Encountered an error while parsing tuple values!";
const NO_CLOSING_PARENTHESIS: &str = "Tuple needs to end with closing parenthesis (')')!";

impl<'a> Parser<'a> {
    fn new(s: &String) -> Parser {
        let it = s.chars();
        let curr = None;

        Parser { it, curr }
    }

    fn parse(&mut self) -> Result<Tuple, &str> {
        self.next();
        if !self.expect('(') {
            return Err(NO_OPENING_PARENTHESIS);
        }

        let mut values = Vec::new();
        while let Some(_) = self.curr {
            if self.expect(')') {
                return Ok(values);
            }

            match self.value() {
                Some(val) => values.push(val),
                None => return Err(ERROR_PARSING_TUPLE),
            }

            self.expect(',');
        }

        Err(NO_CLOSING_PARENTHESIS)
    }

    fn next(&mut self) {
        self.curr = self.it.next();
    }

    fn skip_ws(&mut self) {
        while matches!(self.curr, Some(c) if c.is_whitespace()) {
            self.next();
        }
    }

    fn expect(&mut self, c: char) -> bool {
        self.skip_ws();
        match self.curr {
            Some(val) if val == c => {
                self.next();
                true
            }
            _ => false,
        }
    }

    fn number(&mut self) -> u32 {
        let mut result = 0;
        while let Some(c) = self.curr {
            if c.is_digit(10) {
                result = result * 10 + c as u32 - '0' as u32;
                self.next();
            } else {
                break;
            }
        }

        result
    }

    fn string(&mut self) -> Option<Value> {
        let mut result = String::new();
        self.next();

        while let Some(c) = self.curr {
            match c {
                '"' => {
                    if result.ends_with('\\') {
                        result.pop();
                    } else {
                        self.next();
                        return Some(Value::String(result));
                    }
                }
                other => result.push(other),
            }

            self.next();
        }

        None
    }

    fn value(&mut self) -> Option<Value> {
        match self.curr {
            Some('"') => self.string(),
            Some(c) if c.is_digit(10) || c == '+' || c == '-' || c == '.' => {
                let mut sign = 1;
                if c == '+' {
                    self.next();
                } else if c == '-' {
                    sign = -1;
                    self.next();
                }

                let result = sign * self.number() as i32;
                if matches!(self.curr, Some(c) if c == '.') {
                    self.next();
                    let mut decimal = self.number() as f64;
                    decimal /= 10_f64.powf(decimal.log10().ceil());

                    Some(Value::Float(result as f64 + decimal))
                } else {
                    Some(Value::Int(result))
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer() {
        let input = String::from("(1)");
        let mut parser = Parser::new(&input);

        let result = parser.parse().unwrap();
        assert_eq!(result[0], Value::Int(1));
    }

    #[test]
    fn test_float() {
        let input = String::from("(2.5)");
        let mut parser = Parser::new(&input);

        let result = parser.parse().unwrap();
        assert_eq!(result[0], Value::Float(2.5));
    }

    #[test]
    fn test_string() {
        let input = String::from("(\"test\")");
        let mut parser = Parser::new(&input);

        let result = parser.parse().unwrap();
        assert_eq!(result[0], Value::String(String::from("test")));
    }
}
