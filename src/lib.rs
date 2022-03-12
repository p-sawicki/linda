use std::{process::Command, str::Chars};

#[derive(PartialEq, Debug)]
enum Value {
    Int(Option<i32>),
    Float(Option<f64>),
    String(Option<String>),
}

#[derive(PartialEq, Debug)]
enum ComparisonOperator {
    EQ,
    NEQ,
    GE,
    GT,
    LE,
    LT,
    ANY,
}

type Tuple = Vec<Value>;
type TupleRequest = Vec<Request>;

struct Request {
    value: Value,
    op: ComparisonOperator,
}

struct Parser<'a> {
    curr: Option<char>,
    it: Chars<'a>,
}

const NO_OPENING_PARENTHESIS: &str = "Tuple needs to start with opening parenthesis ('(')!";
const ERROR_PARSING_TUPLE: &str = "Encountered an error while parsing tuple values!";
const NO_CLOSING_PARENTHESIS: &str = "Tuple needs to end with closing parenthesis (')')!";

impl Value {
    fn int(i: i32) -> Value {
        Value::Int(Some(i))
    }

    fn float(f: f64) -> Value {
        Value::Float(Some(f))
    }

    fn string(s: String) -> Value {
        Value::String(Some(s))
    }

    fn is_int(&self) -> bool {
        matches!(self, Value::Int(c))
    }

    fn is_float(&self) -> bool {
        matches!(self, Value::Float(c))
    }

    fn is_string(&self) -> bool {
        matches!(self, Value::String(c))
    }

    fn is_same_type(&self, other: &Value) -> bool {
        self.is_int() && other.is_int()
            || self.is_float() && other.is_float()
            || self.is_string() && other.is_string()
    }
}

impl<'a> Parser<'a> {
    fn new(s: &'a String) -> Parser<'a> {
        let it = s.chars();
        let curr = None;

        Parser { it, curr }
    }

    fn parse(&mut self) -> Result<Tuple, &'static str> {
        self.next();
        if !self.check('(') {
            return Err(NO_OPENING_PARENTHESIS);
        }

        let mut values = Vec::new();
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

    fn parse_request(&mut self) -> Result<TupleRequest, &'static str> {
        self.next();
        if !self.check('(') {
            return Err(NO_OPENING_PARENTHESIS);
        }

        let mut requests = Vec::new();
        while let Some(_) = self.curr {
            if self.check(')') {
                return Ok(requests);
            }

            let type_name = match self.type_name() {
                Some(val) => val,
                _ => return Err(ERROR_PARSING_TUPLE),
            };

            let op = match self.operator() {
                Some(op) => op,
                _ => return Err(ERROR_PARSING_TUPLE),
            };

            let value = if op != ComparisonOperator::ANY {
                self.skip_ws();
                match self.value() {
                    Some(val) if val.is_same_type(&type_name) => val,
                    _ => return Err(ERROR_PARSING_TUPLE),
                }
            } else {
                type_name
            };

            requests.push(Request { value, op });
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_impl(input: &str) -> Result<Vec<Value>, &'static str> {
        Parser::new(&String::from(input)).parse()
    }

    fn request_impl(input: &str) -> Result<Vec<Request>, &'static str> {
        Parser::new(&String::from(input)).parse_request()
    }

    fn parse(input: &str) -> Vec<Value> {
        parse_impl(input).unwrap()
    }

    fn parse_err(input: &str) -> &str {
        parse_impl(input).err().unwrap()
    }

    fn request(input: &str) -> Vec<Request> {
        request_impl(input).unwrap()
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
        assert_eq!(request.value, value);
        assert_eq!(request.op, op);
    }

    #[test]
    fn test_request() {
        let result = request("(int: 1, float: >= 3.0, string: *, int: != 2, float: <= 3.14, string: < \"abc\", int: > 15, )");
        assert_eq!(result[0].value, Value::int(1));
        assert_eq!(result[0].op, ComparisonOperator::EQ);

        assert_eq!(result[1].value, Value::float(3.0));
        assert_eq!(result[1].op, ComparisonOperator::GE);

        assert_eq!(result[2].value, Value::String(None));
        assert_eq!(result[2].op, ComparisonOperator::ANY);

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
}
