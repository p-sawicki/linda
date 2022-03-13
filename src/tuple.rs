#[derive(PartialEq, Debug)]
pub enum Value {
    Int(Option<i32>),
    Float(Option<f64>),
    String(Option<String>),
}

#[derive(PartialEq, Debug)]
pub enum ComparisonOperator {
    EQ,
    NEQ,
    GE,
    GT,
    LE,
    LT,
    ANY,
}

pub type Tuple = Vec<Value>;
pub type TupleRequest = Vec<Request>;

#[derive(Debug, PartialEq)]
pub struct Request {
    value: Value,
    op: ComparisonOperator,
}

impl Value {
    pub fn int(i: i32) -> Value {
        Value::Int(Some(i))
    }

    pub fn float(f: f64) -> Value {
        Value::Float(Some(f))
    }

    pub fn string(s: String) -> Value {
        Value::String(Some(s))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_same_type(&self, other: &Value) -> bool {
        self.is_int() && other.is_int()
            || self.is_float() && other.is_float()
            || self.is_string() && other.is_string()
    }
}

impl Request {
    pub fn new(value: Value, op: ComparisonOperator) -> Request {
        Request { value, op }
    }
}
