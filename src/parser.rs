use crate::types::{Column, DataType, SqlValue};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum SqlCommand {
    CreateTable {
        table_name: String,
        columns: Vec<Column>,
    },
    Insert {
        table_name: String,
        values: Vec<SqlValue>,
    },
    Select {
        table_name: String,
    },
    Update {
        table_name: String,
        set_col: String,
        set_val: SqlValue,
        where_col: String,
        where_val: SqlValue,
    },
    Delete {
        table_name: String,
        where_col: String,
        where_val: SqlValue,
    },
    Help,
    Exit,
}

/// Parse raw SQL query into structured SqlCommand
pub fn parse_sql(sql: &str) -> Result<SqlCommand, String> {
    let trimmed = sql.trim().trim_end_matches(';');
    let upper = trimmed.to_uppercase();

    if upper == "EXIT" || upper == "QUIT" {
        return Ok(SqlCommand::Exit);
    }
    if upper == "HELP" {
        return Ok(SqlCommand::Help);
    }

    if upper.starts_with("CREATE TABLE") {
        let start_idx = trimmed.find('(').ok_or("Missing '(' in CREATE TABLE")?;
        let end_idx = trimmed.rfind(')').ok_or("Missing ')' in CREATE TABLE")?;

        let header_part = trimmed[..start_idx].trim();
        let tokens: Vec<&str> = header_part.split_whitespace().collect();
        if tokens.len() < 3 {
            return Err("Invalid CREATE TABLE syntax".into());
        }
        let table_name = tokens[2].to_string();

        let cols_str = &trimmed[start_idx + 1..end_idx];
        let mut columns = Vec::new();
        for col_def in cols_str.split(',') {
            let parts: Vec<&str> = col_def.trim().split_whitespace().collect();
            if parts.len() < 2 {
                return Err(format!("Invalid column definition: {}", col_def));
            }
            let col_name = parts[0].to_string();
            let dtype = DataType::from_str(parts[1])
                .ok_or_else(|| format!("Unknown data type: {}", parts[1]))?;
            columns.push(Column {
                name: col_name,
                data_type: dtype,
            });
        }

        return Ok(SqlCommand::CreateTable {
            table_name,
            columns,
        });
    }

    if upper.starts_with("INSERT INTO") {
        let values_keyword_idx = upper.find("VALUES").ok_or("Missing VALUES keyword")?;
        let table_part = trimmed[..values_keyword_idx].trim();
        let tokens: Vec<&str> = table_part.split_whitespace().collect();
        if tokens.len() < 3 {
            return Err("Invalid INSERT syntax".into());
        }
        let table_name = tokens[2].to_string();

        let vals_start = trimmed.find('(').ok_or("Missing '(' in VALUES")?;
        let vals_end = trimmed.rfind(')').ok_or("Missing ')' in VALUES")?;
        let vals_str = &trimmed[vals_start + 1..vals_end];

        let mut values = Vec::new();
        for val_str in vals_str.split(',') {
            let v = val_str.trim();
            if v.starts_with('\'') && v.ends_with('\'') {
                let inner = &v[1..v.len() - 1];
                // Check if file path exists on disk to load binary image data into BLOB
                if Path::new(inner).exists() {
                    match fs::read(inner) {
                        Ok(bytes) => values.push(SqlValue::Blob(bytes)),
                        Err(_) => values.push(SqlValue::Text(inner.to_string())),
                    }
                } else {
                    values.push(SqlValue::Text(inner.to_string()));
                }
            } else if let Ok(n) = v.parse::<i64>() {
                values.push(SqlValue::Int(n));
            } else {
                values.push(SqlValue::Text(v.to_string()));
            }
        }

        return Ok(SqlCommand::Insert { table_name, values });
    }

    if upper.starts_with("SELECT * FROM") {
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() < 4 {
            return Err("Invalid SELECT syntax".into());
        }
        let table_name = tokens[3].to_string();
        return Ok(SqlCommand::Select { table_name });
    }

    if upper.starts_with("UPDATE") {
        let set_idx = upper.find("SET").ok_or("Missing SET keyword")?;
        let where_idx = upper.find("WHERE").ok_or("Missing WHERE keyword")?;

        let table_name = trimmed[6..set_idx].trim().to_string();
        let set_clause = trimmed[set_idx + 3..where_idx].trim();
        let where_clause = trimmed[where_idx + 5..].trim();

        let set_parts: Vec<&str> = set_clause.split('=').map(|s| s.trim()).collect();
        if set_parts.len() != 2 {
            return Err("Invalid SET clause".into());
        }
        let set_col = set_parts[0].to_string();
        let set_val = parse_literal(set_parts[1]);

        let where_parts: Vec<&str> = where_clause.split('=').map(|s| s.trim()).collect();
        if where_parts.len() != 2 {
            return Err("Invalid WHERE clause".into());
        }
        let where_col = where_parts[0].to_string();
        let where_val = parse_literal(where_parts[1]);

        return Ok(SqlCommand::Update {
            table_name,
            set_col,
            set_val,
            where_col,
            where_val,
        });
    }

    if upper.starts_with("DELETE FROM") {
        let where_idx = upper.find("WHERE").ok_or("Missing WHERE in DELETE")?;
        let table_part = trimmed[..where_idx].trim();
        let tokens: Vec<&str> = table_part.split_whitespace().collect();
        if tokens.len() < 3 {
            return Err("Invalid DELETE syntax".into());
        }
        let table_name = tokens[2].to_string();

        let where_clause = trimmed[where_idx + 5..].trim();
        let where_parts: Vec<&str> = where_clause.split('=').map(|s| s.trim()).collect();
        if where_parts.len() != 2 {
            return Err("Invalid WHERE clause".into());
        }
        let where_col = where_parts[0].to_string();
        let where_val = parse_literal(where_parts[1]);

        return Ok(SqlCommand::Delete {
            table_name,
            where_col,
            where_val,
        });
    }

    Err(format!("Unknown or unsupported SQL command: {}", sql))
}

fn parse_literal(v: &str) -> SqlValue {
    if v.starts_with('\'') && v.ends_with('\'') {
        SqlValue::Text(v[1..v.len() - 1].to_string())
    } else if let Ok(n) = v.parse::<i64>() {
        SqlValue::Int(n)
    } else {
        SqlValue::Text(v.to_string())
    }
}