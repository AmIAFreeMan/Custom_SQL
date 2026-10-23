use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int,
    Text,
    Blob, // Suitable for raw image bytes or binary data
}

impl DataType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "INT" => Some(DataType::Int),
            "TEXT" => Some(DataType::Text),
            "BLOB" => Some(DataType::Blob),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SqlValue {
    Int(i64),
    Text(String),
    Blob(Vec<u8>), // Binary data (e.g. image bytes)
    Null,
}

impl SqlValue {
    /// Convert value to serialized string format for disk storage.
    pub fn serialize(&self) -> String {
        match self {
            SqlValue::Int(n) => format!("INT:{}", n),
            SqlValue::Text(t) => format!("TEXT:{}", t),
            SqlValue::Blob(b) => format!("BLOB:{}", hex_encode(b)),
            SqlValue::Null => "NULL:".to_string(),
        }
    }

    /// Deserialize disk string back into structured SqlValue.
    pub fn deserialize(s: &str) -> Self {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() < 2 {
            return SqlValue::Null;
        }
        match parts[0] {
            "INT" => SqlValue::Int(parts[1].parse().unwrap_or(0)),
            "TEXT" => SqlValue::Text(parts[1].to_string()),
            "BLOB" => SqlValue::Blob(hex_decode(parts[1])),
            _ => SqlValue::Null,
        }
    }
}

impl fmt::Display for SqlValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlValue::Int(n) => write!(f, "{}", n),
            SqlValue::Text(t) => write!(f, "{}", t),
            SqlValue::Blob(b) => write!(f, "[BLOB Image: {} bytes]", b.len()),
            SqlValue::Null => write!(f, "NULL"),
        }
    }
}

/// Convert byte array to hexadecimal string for storage
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Convert hexadecimal string back to byte array
fn hex_decode(s: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i + 1 < chars.len() {
        let hex_str: String = chars[i..=i + 1].iter().collect();
        if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
            bytes.push(byte);
        }
        i += 2;
    }
    bytes
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
}