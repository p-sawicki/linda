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
    fn new(s: &'a String) -> Parser<'a> {
        let it = s.chars();
        let curr = None;

        Parser { it, curr }
    }

    fn parse(&mut self) -> Result<Tuple, &'static str> {
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
            if c == '"' {
                if result.ends_with('\\') {
                    result.pop();
                } else {
                    self.next();
                    return Some(Value::String(result));
                }
            }

            result.push(c);
            self.next();
        }

        None
    }

    fn value(&mut self) -> Option<Value> {
        match self.curr {
            Some('"') => self.string(),
            Some(c) if c.is_digit(10) || c == '+' || c == '-' || c == '.' => {
                let mut sign = 1;
                if self.expect('-') {
                    sign = -1;
                } else {
                    self.expect('+');
                }

                if let Some(c) = self.curr {
                    if !c.is_digit(10) && c != '.' {
                        return None;
                    }
                }

                let result = sign * self.number() as i32;
                if self.expect('.') {
                    let mut decimal = self.number() as f64;
                    if decimal != 0.0 {
                        decimal /= 10_f64.powf(decimal.log10().ceil());
                    }

                    Some(Value::Float(result as f64 + sign as f64 * decimal))
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

    fn parse_impl(input: &str) -> Result<Vec<Value>, &'static str> {
        Parser::new(&String::from(input)).parse()
    }

    fn parse(input: &str) -> Vec<Value> {
        parse_impl(input).unwrap()
    }

    fn parse_err(input: &str) -> &str {
        parse_impl(input).err().unwrap()
    }

    #[test]
    fn test_integer() {
        let result = parse("(1)");
        assert_eq!(result[0], Value::Int(1));

        let result = parse("(+2)");
        assert_eq!(result[0], Value::Int(2));

        let result = parse("(-3)");
        assert_eq!(result[0], Value::Int(-3));
    }

    #[test]
    fn test_float() {
        let result = parse("(2.5)");
        assert_eq!(result[0], Value::Float(2.5));

        let result = parse("(+.3)");
        assert_eq!(result[0], Value::Float(0.3));

        let result = parse("(-4.)");
        assert_eq!(result[0], Value::Float(-4.0));
    }

    #[test]
    fn test_string() {
        let result = parse("(\"test\")");
        assert_eq!(result[0], Value::String(String::from("test")));

        let result = parse("(\"te\\\"st\")");
        assert_eq!(result[0], Value::String(String::from("te\"st")));
    }

    #[test]
    fn test_multiple() {
        let result = parse("(+1, -3.14, \"test\", )");
        assert_eq!(result[0], Value::Int(1));
        assert_eq!(result[1], Value::Float(-3.14));
        assert_eq!(result[2], Value::String(String::from("test")));
    }

    #[test]
    fn test_err() {
        let result = parse_err("1");
        assert_eq!(result, NO_OPENING_PARENTHESIS);

        let result = parse_err("(1");
        assert_eq!(result, NO_CLOSING_PARENTHESIS);

        let result = parse_err("(+-1)");
        assert_eq!(result, ERROR_PARSING_TUPLE);
    }
}
