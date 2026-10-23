use crate::parser::SqlCommand;
use crate::types::{Column, DataType, SqlValue, TableSchema};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct DatabaseEngine {
    data_dir: String,
}

impl DatabaseEngine {
    pub fn new(data_dir: &str) -> Self {
        let _ = fs::create_dir_all(data_dir);
        DatabaseEngine {
            data_dir: data_dir.to_string(),
        }
    }

    fn meta_path(&self, table_name: &str) -> String {
        format!("{}/{}.meta", self.data_dir, table_name)
    }

    fn data_path(&self, table_name: &str) -> String {
        format!("{}/{}.data", self.data_dir, table_name)
    }

    fn load_schema(&self, table_name: &str) -> Result<TableSchema, String> {
        let path = self.meta_path(table_name);
        if !Path::new(&path).exists() {
            return Err(format!("Table '{}' does not exist.", table_name));
        }
        let file = fs::File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut columns = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 2 {
                let dtype = DataType::from_str(parts[1]).unwrap_or(DataType::Text);
                columns.push(Column {
                    name: parts[0].to_string(),
                    data_type: dtype,
                });
            }
        }
        Ok(TableSchema {
            name: table_name.to_string(),
            columns,
        })
    }

    pub fn execute(&self, cmd: SqlCommand) -> Result<String, String> {
        match cmd {
            SqlCommand::Help => Ok("\nAvailable SQL Commands:\n\
                    ------------------------------------------------------------------\n\
                    CREATE TABLE table_name (col1 INT, col2 TEXT, col3 BLOB);\n\
                    INSERT INTO table_name VALUES (1, 'Alice', 'path/to/image.png');\n\
                    SELECT * FROM table_name;\n\
                    UPDATE table_name SET col2 = 'NewVal' WHERE col1 = 1;\n\
                    DELETE FROM table_name WHERE col1 = 1;\n\
                    EXIT;\n"
                .to_string()),
            SqlCommand::Exit => Ok("Goodbye!".to_string()),

            SqlCommand::CreateTable {
                table_name,
                columns,
            } => {
                let m_path = self.meta_path(&table_name);
                if Path::new(&m_path).exists() {
                    return Err(format!("Table '{}' already exists.", table_name));
                }

                // Write metadata file (.meta)
                let mut file = fs::File::create(m_path).map_err(|e| e.to_string())?;
                for col in &columns {
                    writeln!(file, "{}|{:?}", col.name, col.data_type)
                        .map_err(|e| e.to_string())?;
                }

                // Create empty data file (.data)
                fs::File::create(self.data_path(&table_name)).map_err(|e| e.to_string())?;
                Ok(format!("Table '{}' created successfully.", table_name))
            }

            SqlCommand::Insert { table_name, values } => {
                let schema = self.load_schema(&table_name)?;
                if values.len() != schema.columns.len() {
                    return Err(format!(
                        "Column count mismatch. Expected {}, got {}",
                        schema.columns.len(),
                        values.len()
                    ));
                }

                let serialized_row: Vec<String> = values.iter().map(|v| v.serialize()).collect();
                let row_line = serialized_row.join(",") + "\n";

                let mut file = OpenOptions::new()
                    .append(true)
                    .open(self.data_path(&table_name))
                    .map_err(|e| e.to_string())?;
                file.write_all(row_line.as_bytes())
                    .map_err(|e| e.to_string())?;

                Ok("1 row inserted successfully.".to_string())
            }

            SqlCommand::Select { table_name } => {
                let schema = self.load_schema(&table_name)?;
                let file =
                    fs::File::open(self.data_path(&table_name)).map_err(|e| e.to_string())?;
                let reader = BufReader::new(file);

                let headers: Vec<String> =
                    schema.columns.iter().map(|c| c.name.clone()).collect();
                let mut output = format!("{}\n", headers.join(" | "));
                output.push_str(&"-".repeat(output.len() * 2));
                output.push('\n');

                let mut count = 0;
                for line in reader.lines() {
                    let line = line.map_err(|e| e.to_string())?;
                    if line.trim().is_empty() {
                        continue;
                    }

                    let parts: Vec<&str> = line.split(',').collect();
                    let mut row_vals = Vec::new();
                    for p in parts {
                        let val = SqlValue::deserialize(p);
                        row_vals.push(val.to_string());
                    }
                    output.push_str(&format!("{}\n", row_vals.join(" | ")));
                    count += 1;
                }
                output.push_str(&format!("\n({} rows)", count));
                Ok(output)
            }

            SqlCommand::Update {
                table_name,
                set_col,
                set_val,
                where_col,
                where_val,
            } => {
                let schema = self.load_schema(&table_name)?;
                let set_idx = schema
                    .columns
                    .iter()
                    .position(|c| c.name == set_col)
                    .ok_or_else(|| format!("Column '{}' not found", set_col))?;
                let where_idx = schema
                    .columns
                    .iter()
                    .position(|c| c.name == where_col)
                    .ok_or_else(|| format!("Column '{}' not found", where_col))?;

                let file =
                    fs::File::open(self.data_path(&table_name)).map_err(|e| e.to_string())?;
                let reader = BufReader::new(file);
                let mut updated_rows = Vec::new();
                let mut modified_count = 0;

                for line in reader.lines() {
                    let line = line.map_err(|e| e.to_string())?;
                    if line.trim().is_empty() {
                        continue;
                    }

                    let mut parts: Vec<String> =
                        line.split(',').map(|s| s.to_string()).collect();
                    if parts.len() > where_idx {
                        let current_where_val = SqlValue::deserialize(&parts[where_idx]);
                        if current_where_val == where_val {
                            parts[set_idx] = set_val.serialize();
                            modified_count += 1;
                        }
                    }
                    updated_rows.push(parts.join(","));
                }

                // Overwrite table file with updated values
                let mut file =
                    fs::File::create(self.data_path(&table_name)).map_err(|e| e.to_string())?;
                for row in updated_rows {
                    writeln!(file, "{}", row).map_err(|e| e.to_string())?;
                }

                Ok(format!("{} row(s) updated successfully.", modified_count))
            }

            SqlCommand::Delete {
                table_name,
                where_col,
                where_val,
            } => {
                let schema = self.load_schema(&table_name)?;
                let where_idx = schema
                    .columns
                    .iter()
                    .position(|c| c.name == where_col)
                    .ok_or_else(|| format!("Column '{}' not found", where_col))?;

                let file =
                    fs::File::open(self.data_path(&table_name)).map_err(|e| e.to_string())?;
                let reader = BufReader::new(file);
                let mut remaining_rows = Vec::new();
                let mut deleted_count = 0;

                for line in reader.lines() {
                    let line = line.map_err(|e| e.to_string())?;
                    if line.trim().is_empty() {
                        continue;
                    }

                    let parts: Vec<String> =
                        line.split(',').map(|s| s.to_string()).collect();
                    let mut keep = true;
                    if parts.len() > where_idx {
                        let current_where_val = SqlValue::deserialize(&parts[where_idx]);
                        if current_where_val == where_val {
                            keep = false;
                            deleted_count += 1;
                        }
                    }
                    if keep {
                        remaining_rows.push(line);
                    }
                }

                // Rewrite file without deleted rows
                let mut file =
                    fs::File::create(self.data_path(&table_name)).map_err(|e| e.to_string())?;
                for row in remaining_rows {
                    writeln!(file, "{}", row).map_err(|e| e.to_string())?;
                }

                Ok(format!("{} row(s) deleted successfully.", deleted_count))
            }
        }
    }
}