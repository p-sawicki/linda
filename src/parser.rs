use std::str::Chars;

use crate::tuple::*;
use crate::utils::*;

pub struct Parser<'a> {
    curr: Option<char>,
    it: Chars<'a>,
}

const INVALID_COMMAND: &str = "Invalid command given!";
const NO_OPENING_PARENTHESIS: &str = "Tuple needs to start with opening parenthesis ('(')!";
const ERROR_PARSING_TUPLE: &str = "Encountered an error while parsing tuple values!";
const NO_CLOSING_PARENTHESIS: &str = "Tuple needs to end with closing parenthesis (')')!";

impl<'a> Parser<'a> {
    pub fn new(s: &'a String) -> Parser<'a> {
        let it = s.chars();
        let curr = None;
        let mut parser = Parser { it, curr };
        parser.next();

        parser
    }

    pub fn parse(&mut self) -> Result<Command, &'static str> {
        match &self.word().to_lowercase()[..] {
            "out" => Ok(Command::Out(self.tuple()?)),
            "in" => Ok(Command::In(self.request()?, self.number() as Timeout)),
            "rd" | "read" => Ok(Command::Rd(self.request()?, self.number() as Timeout)),
            "inp" => Ok(Command::Inp(self.request()?)),
            "rdp" | "readp" => Ok(Command::Rdp(self.request()?)),
            "help" => Ok(Command::Help),
            "exit" => Ok(Command::Exit),
            _ => Err(INVALID_COMMAND),
        }
    }

    fn tuple(&mut self) -> Result<Tuple<Value>, &'static str> {
        self.skip_ws();
        if !self.check('(') {
            return Err(NO_OPENING_PARENTHESIS);
        }

        let mut values = Tuple::new();
        while let Some(_) = self.curr {
            if self.check(')') {
                return Ok(values);
            }

            match self.value() {
                Some(val) => values.push(val),
                None => return Err(ERROR_PARSING_TUPLE),
            }

            self.check(',');
        }

        Err(NO_CLOSING_PARENTHESIS)
    }

    fn request(&mut self) -> Result<Tuple<Request>, &'static str> {
        self.skip_ws();
        if !self.check('(') {
            return Err(NO_OPENING_PARENTHESIS);
        }

        let mut requests = Tuple::new();
        while let Some(_) = self.curr {
            if self.check(')') {
                return Ok(requests);
            }

            let type_name = match self.type_name() {
                Some(val) => val,
                _ => return Err(ERROR_PARSING_TUPLE),
            };

            let operator = match self.operator() {
                Some(op) => op,
                _ => return Err(ERROR_PARSING_TUPLE),
            };

            let value = if operator != ComparisonOperator::ANY {
                self.skip_ws();
                match self.value() {
                    Some(val) if val.is_same_type(&type_name) => val,
                    _ => return Err(ERROR_PARSING_TUPLE),
                }
            } else {
                type_name
            };

            requests.push(Request::new(value, operator));
            self.check(',');
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

    fn check(&mut self, c: char) -> bool {
        self.skip_ws();
        match self.curr {
            Some(val) if val == c => {
                self.next();
                true
            }
            _ => false,
        }
    }

    fn check_next<T>(&mut self, c: char, if_true: T, if_false: T) -> T {
        self.next();
        if self.check(c) {
            if_true
        } else {
            if_false
        }
    }

    fn number(&mut self) -> u32 {
        self.skip_ws();
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
                    return Some(Value::string(result));
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
                if self.check('-') {
                    sign = -1;
                } else {
                    self.check('+');
                }

                if let Some(c) = self.curr {
                    if !c.is_digit(10) && c != '.' {
                        return None;
                    }
                }

                let result = sign * self.number() as i32;
                if self.check('.') {
                    let mut decimal = self.number() as f64;
                    if decimal != 0.0 {
                        decimal /= 10_f64.powf(decimal.log10().ceil());
                    }

                    Some(Value::float(result as f64 + sign as f64 * decimal))
                } else {
                    Some(Value::int(result))
                }
            }
            _ => None,
        }
    }

    fn type_name(&mut self) -> Option<Value> {
        let mut name = String::new();
        while let Some(c) = self.curr {
            if self.check(':') {
                return match &name.to_lowercase()[..] {
                    "int" => Some(Value::Int(None)),
                    "float" => Some(Value::Float(None)),
                    "string" => Some(Value::String(None)),
                    _ => None,
                };
            }

            name.push(c);
            self.next();
        }

        None
    }

    fn operator(&mut self) -> Option<ComparisonOperator> {
        self.skip_ws();
        match self.curr {
            Some('=') => self.check_next('=', Some(ComparisonOperator::EQ), None),
            Some('!') => self.check_next('=', Some(ComparisonOperator::NEQ), None),
            Some('<') => self.check_next(
                '=',
                Some(ComparisonOperator::LE),
                Some(ComparisonOperator::LT),
            ),
            Some('>') => self.check_next(
                '=',
                Some(ComparisonOperator::GE),
                Some(ComparisonOperator::GT),
            ),
            Some('*') => {
                self.next();
                Some(ComparisonOperator::ANY)
            }
            _ => Some(ComparisonOperator::EQ),
        }
    }

    fn word(&mut self) -> String {
        self.skip_ws();
        let mut result = String::new();
        while let Some(c) = self.curr {
            if c.is_whitespace() {
                break;
            }

            result.push(c);
            self.next();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_impl(input: &str) -> Result<Tuple<Value>, &'static str> {
        Parser::new(&String::from(input)).tuple()
    }

    fn request_impl(input: &str) -> Result<Tuple<Request>, &'static str> {
        Parser::new(&String::from(input)).request()
    }

    fn command_impl(input: &str) -> Result<Command, &'static str> {
        Parser::new(&String::from(input)).parse()
    }

    fn parse(input: &str) -> Tuple<Value> {
        parse_impl(input).unwrap()
    }

    fn parse_err(input: &str) -> &str {
        parse_impl(input).err().unwrap()
    }

    fn request(input: &str) -> Tuple<Request> {
        request_impl(input).unwrap()
    }

    fn command(input: &str) -> Command {
        command_impl(input).unwrap()
    }

    fn command_err(input: &str) -> &str {
        command_impl(input).err().unwrap()
    }

    #[test]
    fn test_integer() {
        let result = parse("(1)");
        assert_eq!(result[0], Value::int(1));

        let result = parse("(+2)");
        assert_eq!(result[0], Value::int(2));

        let result = parse("(-3)");
        assert_eq!(result[0], Value::int(-3));
    }

    #[test]
    fn test_float() {
        let result = parse("(2.5)");
        assert_eq!(result[0], Value::float(2.5));

        let result = parse("(+.3)");
        assert_eq!(result[0], Value::float(0.3));

        let result = parse("(-4.)");
        assert_eq!(result[0], Value::float(-4.0));
    }

    #[test]
    fn test_string() {
        let result = parse("(\"test\")");
        assert_eq!(result[0], Value::string(String::from("test")));

        let result = parse("(\"te\\\"st\")");
        assert_eq!(result[0], Value::string(String::from("te\"st")));
    }

    #[test]
    fn test_multiple() {
        let result = parse("(+1, -3.14, \"test\", )");
        assert_eq!(result[0], Value::int(1));
        assert_eq!(result[1], Value::float(-3.14));
        assert_eq!(result[2], Value::string(String::from("test")));
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

    fn check_request(request: &Request, value: Value, op: ComparisonOperator) {
        assert_eq!(*request, Request::new(value, op));
    }

    #[test]
    fn test_request() {
        let result = request("(int: 1, float: >= 3.0, string: *, int: != 2, float: <= 3.14, string: < \"abc\", int: > 15, )");

        check_request(&result[0], Value::int(1), ComparisonOperator::EQ);
        check_request(&result[1], Value::float(3.0), ComparisonOperator::GE);
        check_request(&result[2], Value::String(None), ComparisonOperator::ANY);
        check_request(&result[3], Value::int(2), ComparisonOperator::NEQ);
        check_request(&result[4], Value::float(3.14), ComparisonOperator::LE);
        check_request(
            &result[5],
            Value::string(String::from("abc")),
            ComparisonOperator::LT,
        );
        check_request(&result[6], Value::int(15), ComparisonOperator::GT);
    }

    fn make_tuple<T>(mut vals: Vec<T>) -> Tuple<T> {
        let mut result = Tuple::new();
        result.append(&mut vals);
        result
    }

    #[test]
    fn test_command() {
        let result = command("out (1)");
        assert_eq!(result, Command::Out(make_tuple(vec![Value::int(1)])));

        let result = command("in (float: *) 10");
        assert_eq!(
            result,
            Command::In(
                make_tuple(vec![Request::new(
                    Value::Float(None),
                    ComparisonOperator::ANY
                )]),
                10 as Timeout
            )
        );

        let result = command("rd (string: \"a\") 3");
        assert_eq!(
            result,
            Command::Rd(
                make_tuple(vec![Request::new(
                    Value::string(String::from("a")),
                    ComparisonOperator::EQ
                )]),
                3 as Timeout
            )
        );

        let result = command("inp (int: <= 5)");
        assert_eq!(
            result,
            Command::Inp(make_tuple(vec![Request::new(
                Value::int(5),
                ComparisonOperator::LE
            )]))
        );

        let result = command("rdp ()");
        assert_eq!(result, Command::Rdp(make_tuple(vec![])));

        let result = command("help");
        assert_eq!(result, Command::Help);

        let result = command("exit");
        assert_eq!(result, Command::Exit);

        let result = command_err("q");
        assert_eq!(result, INVALID_COMMAND);
    }
}
