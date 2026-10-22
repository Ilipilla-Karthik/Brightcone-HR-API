use serde::{Deserialize, Serialize};

/// Supported PostgreSQL data types in the entity schema registry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PgDataType {
    Varchar(Option<u32>),
    Text,
    Integer,
    BigInt,
    Float,
    Boolean,
    TimestampWithTimeZone,
    Jsonb,
    Custom(String),
}

impl PgDataType {
    /// Render the SQL data type expression for PostgreSQL DDL
    pub fn to_sql(&self) -> String {
        match self {
            PgDataType::Varchar(Some(len)) => format!("VARCHAR({})", len),
            PgDataType::Varchar(None) => "VARCHAR".to_string(),
            PgDataType::Text => "TEXT".to_string(),
            PgDataType::Integer => "INTEGER".to_string(),
            PgDataType::BigInt => "BIGINT".to_string(),
            PgDataType::Float => "REAL".to_string(),
            PgDataType::Boolean => "BOOLEAN".to_string(),
            PgDataType::TimestampWithTimeZone => "TIMESTAMP WITH TIME ZONE".to_string(),
            PgDataType::Jsonb => "JSONB".to_string(),
            PgDataType::Custom(s) => s.clone(),
        }
    }

    /// Check if an introspected PostgreSQL column type matches this definition
    pub fn matches_pg(&self, data_type: &str, udt_name: &str, char_len: Option<i32>) -> bool {
        let dt = data_type.to_lowercase();
        let udt = udt_name.to_lowercase();
        match self {
            PgDataType::Varchar(Some(len)) => {
                (dt.contains("character varying") || dt.contains("varchar") || udt == "varchar")
                    && (char_len == Some(*len as i32) || char_len.is_none())
            }
            PgDataType::Varchar(None) => {
                dt.contains("character varying")
                    || dt.contains("varchar")
                    || udt == "varchar"
                    || dt == "text"
            }
            PgDataType::Text => dt == "text" || udt == "text" || dt.contains("character varying"),
            PgDataType::Integer => dt.contains("int") || udt == "int4" || dt == "integer",
            PgDataType::BigInt => dt.contains("bigint") || udt == "int8",
            PgDataType::Float => {
                dt.contains("float") || dt.contains("real") || udt == "float4" || udt == "float8"
            }
            PgDataType::Boolean => dt.contains("bool") || udt == "bool",
            PgDataType::TimestampWithTimeZone => {
                dt.contains("timestamp") || udt.contains("timestamp")
            }
            PgDataType::Jsonb => dt.contains("json") || udt == "jsonb" || udt == "json",
            PgDataType::Custom(s) => dt.contains(&s.to_lowercase()),
        }
    }
}

/// Metadata description of a single column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: PgDataType,
    pub nullable: bool,
    pub default_expr: Option<String>,
    pub is_primary_key: bool,
    pub is_unique: bool,
}

impl ColumnDefinition {
    pub fn new(name: &str, data_type: PgDataType) -> Self {
        Self {
            name: name.to_string(),
            data_type,
            nullable: true,
            default_expr: None,
            is_primary_key: false,
            is_unique: false,
        }
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.nullable = false;
        self
    }

    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    pub fn default_val(mut self, expr: &str) -> Self {
        self.default_expr = Some(expr.to_string());
        self
    }

    /// Generate column definition DDL snippet (e.g. "title VARCHAR(255) NOT NULL DEFAULT ''")
    pub fn to_column_sql(&self) -> String {
        let mut sql = format!("\"{}\" {}", self.name, self.data_type.to_sql());
        if !self.nullable {
            sql.push_str(" NOT NULL");
        }
        if let Some(ref def) = self.default_expr {
            sql.push_str(&format!(" DEFAULT {}", def));
        }
        sql
    }
}

/// Metadata description of an index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

impl IndexDefinition {
    pub fn new(name: &str, columns: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            columns: columns.iter().map(|s| s.to_string()).collect(),
            unique: false,
        }
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn to_create_sql(&self, table_name: &str) -> String {
        let u = if self.unique { "UNIQUE " } else { "" };
        let cols = self
            .columns
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "CREATE {}INDEX IF NOT EXISTS \"{}\" ON \"{}\" ({});",
            u, self.name, table_name, cols
        )
    }
}

/// Metadata description of a foreign key constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyDefinition {
    pub name: String,
    pub column: String,
    pub referenced_table: String,
    pub referenced_column: String,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

/// Metadata description of a check constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConstraintDefinition {
    pub name: String,
    pub expression: String,
}

/// Authoritative metadata definition for a PostgreSQL table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub primary_key: Vec<String>,
    pub indexes: Vec<IndexDefinition>,
    pub foreign_keys: Vec<ForeignKeyDefinition>,
    pub check_constraints: Vec<CheckConstraintDefinition>,
}

impl TableDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: Vec::new(),
            primary_key: Vec::new(),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
        }
    }

    pub fn column(mut self, col: ColumnDefinition) -> Self {
        if col.is_primary_key && !self.primary_key.contains(&col.name) {
            self.primary_key.push(col.name.clone());
        }
        self.columns.push(col);
        self
    }

    pub fn index(mut self, idx: IndexDefinition) -> Self {
        self.indexes.push(idx);
        self
    }

    pub fn foreign_key(mut self, fk: ForeignKeyDefinition) -> Self {
        self.foreign_keys.push(fk);
        self
    }

    pub fn check(mut self, name: &str, expression: &str) -> Self {
        self.check_constraints.push(CheckConstraintDefinition {
            name: name.to_string(),
            expression: expression.to_string(),
        });
        self
    }

    /// Generate the full CREATE TABLE IF NOT EXISTS statement
    pub fn to_create_table_sql(&self) -> String {
        let mut parts = Vec::new();

        for col in &self.columns {
            parts.push(col.to_column_sql());
        }

        if !self.primary_key.is_empty() {
            let pk_cols = self
                .primary_key
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("PRIMARY KEY ({})", pk_cols));
        }

        for fk in &self.foreign_keys {
            let mut fk_sql = format!(
                "CONSTRAINT \"{}\" FOREIGN KEY (\"{col}\") REFERENCES \"{tbl}\" (\"{ref_col}\")",
                fk.name,
                col = fk.column,
                tbl = fk.referenced_table,
                ref_col = fk.referenced_column
            );
            if let Some(ref d) = fk.on_delete {
                fk_sql.push_str(&format!(" ON DELETE {}", d));
            }
            if let Some(ref u) = fk.on_update {
                fk_sql.push_str(&format!(" ON UPDATE {}", u));
            }
            parts.push(fk_sql);
        }

        for chk in &self.check_constraints {
            parts.push(format!(
                "CONSTRAINT \"{}\" CHECK ({})",
                chk.name, chk.expression
            ));
        }

        format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" (\n  {}\n);",
            self.name,
            parts.join(",\n  ")
        )
    }
}
