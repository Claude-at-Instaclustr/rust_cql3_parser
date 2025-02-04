use bigdecimal::BigDecimal;
use bytes::Bytes;
use hex;
use itertools::Itertools;
use num::BigInt;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use uuid::Uuid;

/// A column definition.
/// This is used in many places, however the primary_key value should only be used in
/// the `create table` calls.  In all other cases it will yield an invalid statment.
#[derive(PartialEq, Debug, Clone)]
pub struct ColumnDefinition {
    /// the name of the column
    pub name: String,
    /// the data type for the column
    pub data_type: DataType,
    /// if set this column is the primary key.
    pub primary_key: bool,
}

impl Display for ColumnDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{}",
            self.name,
            self.data_type,
            if self.primary_key { " PRIMARY KEY" } else { "" }
        )
    }
}

/// the definition of a data type
#[derive(PartialEq, Debug, Clone)]
pub struct DataType {
    /// the name of the data type.
    pub name: DataTypeName,
    /// the definition of the data type.  Normally this is empty but may contain data types that
    /// comprise the named type. (e.g. `FROZEN<foo>` will have foo in the definition)
    pub definition: Vec<DataTypeName>,
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.definition.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}<{}>", self.name, self.definition.iter().join(", "))
        }
    }
}

/// An enumeration of data types.
#[derive(PartialEq, Debug, Clone)]
pub enum DataTypeName {
    Timestamp,
    Set,
    Ascii,
    BigInt,
    Blob,
    Boolean,
    Counter,
    Date,
    Decimal,
    Double,
    Float,
    Frozen,
    Inet,
    Int,
    List,
    Map,
    SmallInt,
    Text,
    Time,
    TimeUuid,
    TinyInt,
    Tuple,
    VarChar,
    VarInt,
    Uuid,
    /// defines a custom type.  Where the name is the name of the type.
    Custom(String),
}

impl Display for DataTypeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataTypeName::Timestamp => write!(f, "TIMESTAMP"),
            DataTypeName::Set => write!(f, "SET"),
            DataTypeName::Ascii => write!(f, "ASCII"),
            DataTypeName::BigInt => write!(f, "BIGINT"),
            DataTypeName::Blob => write!(f, "BLOB"),
            DataTypeName::Boolean => write!(f, "BOOLEAN"),
            DataTypeName::Counter => write!(f, "COUNTER"),
            DataTypeName::Date => write!(f, "DATE"),
            DataTypeName::Decimal => write!(f, "DECIMAL"),
            DataTypeName::Double => write!(f, "DOUBLE"),
            DataTypeName::Float => write!(f, "FLOAT"),
            DataTypeName::Frozen => write!(f, "FROZEN"),
            DataTypeName::Inet => write!(f, "INET"),
            DataTypeName::Int => write!(f, "INT"),
            DataTypeName::List => write!(f, "LIST"),
            DataTypeName::Map => write!(f, "MAP"),
            DataTypeName::SmallInt => write!(f, "SMALLINT"),
            DataTypeName::Text => write!(f, "TEXT"),
            DataTypeName::Time => write!(f, "TIME"),
            DataTypeName::TimeUuid => write!(f, "TIMEUUID"),
            DataTypeName::TinyInt => write!(f, "TINYINT"),
            DataTypeName::Tuple => write!(f, "TUPLE"),
            DataTypeName::VarChar => write!(f, "VARCHAR"),
            DataTypeName::VarInt => write!(f, "VARINT"),
            DataTypeName::Uuid => write!(f, "UUID"),
            DataTypeName::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl DataTypeName {
    pub fn from(name: &str) -> DataTypeName {
        match name.to_uppercase().as_str() {
            "ASCII" => DataTypeName::Ascii,
            "BIGINT" => DataTypeName::BigInt,
            "BLOB" => DataTypeName::Blob,
            "BOOLEAN" => DataTypeName::Boolean,
            "COUNTER" => DataTypeName::Counter,
            "DATE" => DataTypeName::Date,
            "DECIMAL" => DataTypeName::Decimal,
            "DOUBLE" => DataTypeName::Double,
            "FLOAT" => DataTypeName::Float,
            "FROZEN" => DataTypeName::Frozen,
            "INET" => DataTypeName::Inet,
            "INT" => DataTypeName::Int,
            "LIST" => DataTypeName::List,
            "MAP" => DataTypeName::Map,
            "SET" => DataTypeName::Set,
            "SMALLINT" => DataTypeName::SmallInt,
            "TEXT" => DataTypeName::Text,
            "TIME" => DataTypeName::Time,
            "TIMESTAMP" => DataTypeName::Timestamp,
            "TIMEUUID" => DataTypeName::TimeUuid,
            "TINYINT" => DataTypeName::TinyInt,
            "TUPLE" => DataTypeName::Tuple,
            "UUID" => DataTypeName::Uuid,
            "VARCHAR" => DataTypeName::VarChar,
            "VARINT" => DataTypeName::VarInt,
            _ => DataTypeName::Custom(name.to_string()),
        }
    }
}

/// An object that can be on either side of an `Operator`
#[derive(PartialEq, Debug, Clone, Eq, Ord, PartialOrd)]
pub enum Operand {
    /// A constant
    Const(String),
    /// a map displays as `{ String:String, String:String, ... }`
    Map(Vec<(String, String)>),
    /// a set of values.  Displays as `( String, String, ...)`
    Set(Vec<String>),
    /// a list of values.  Displays as `[String, String, ...]`
    List(Vec<String>),
    /// a tuple of values.  Displays as `{ Operand, Operand, ... }`
    Tuple(Vec<Operand>),
    /// A column name
    Column(String),
    /// A function name
    Func(String),
    /// A parameter.  The string will either be '?' or ':name'
    Param(String),
    /// the `NULL` value.
    Null,
    /// an arbitrary collection of Operands
    Collection(Vec<Operand>),
}

/// this is _NOT_ the same as `Operand::Const(string)`  This conversion encloses the value in
/// single quotes.
impl From<&str> for Operand {
    fn from(txt: &str) -> Self {
        Operand::Const(format!("'{}'", txt))
    }
}

impl From<&Bytes> for Operand {
    fn from(b: &Bytes) -> Self {
        Operand::from_hex(&hex::encode(b))
    }
}

impl From<&bool> for Operand {
    fn from(b: &bool) -> Self {
        Operand::Const(if *b {
            "TRUE".to_string()
        } else {
            "FALSE".to_string()
        })
    }
}

impl From<&u128> for Operand {
    fn from(i: &u128) -> Self {
        Operand::Const(i.to_string())
    }
}
impl From<&u64> for Operand {
    fn from(i: &u64) -> Self {
        Operand::Const(i.to_string())
    }
}
impl From<&u32> for Operand {
    fn from(i: &u32) -> Self {
        Operand::Const(i.to_string())
    }
}

impl From<&u16> for Operand {
    fn from(i: &u16) -> Self {
        Operand::Const(i.to_string())
    }
}

impl From<&u8> for Operand {
    fn from(i: &u8) -> Self {
        Operand::Const(i.to_string())
    }
}
impl From<&i128> for Operand {
    fn from(i: &i128) -> Self {
        Operand::Const(i.to_string())
    }
}

impl From<&i64> for Operand {
    fn from(i: &i64) -> Self {
        Operand::Const(i.to_string())
    }
}
impl From<&i32> for Operand {
    fn from(i: &i32) -> Self {
        Operand::Const(i.to_string())
    }
}

impl From<&i16> for Operand {
    fn from(i: &i16) -> Self {
        Operand::Const(i.to_string())
    }
}

impl From<&i8> for Operand {
    fn from(i: &i8) -> Self {
        Operand::Const(i.to_string())
    }
}

impl From<&f64> for Operand {
    fn from(i: &f64) -> Self {
        Operand::Const(i.to_string())
    }
}
impl From<&f32> for Operand {
    fn from(i: &f32) -> Self {
        Operand::Const(i.to_string())
    }
}

impl From<&BigInt> for Operand {
    fn from(b: &BigInt) -> Self {
        Operand::Const(b.to_string())
    }
}

impl From<&BigDecimal> for Operand {
    fn from(b: &BigDecimal) -> Self {
        Operand::Const(b.to_string())
    }
}

impl From<&IpAddr> for Operand {
    fn from(addr: &IpAddr) -> Self {
        Operand::from(addr.to_string().as_str())
    }
}

impl From<&Uuid> for Operand {
    fn from(uuid: &Uuid) -> Self {
        Operand::from(uuid.to_string().as_str())
    }
}

impl Operand {
    /// creates creates a properly formated Operand::Const for a hex string.
    fn from_hex(hex_str: &str) -> Operand {
        Operand::Const(format!("0x{}", hex_str))
    }

    /// unescapes a CQL string
    /// Specifically converts `''` to `'` and removes the leading and
    /// trailing delimiters.  For all other strings this is method returns
    /// the argument.
    pub fn unescape(value: &str) -> String {
        if value.starts_with('\'') {
            let mut chars = value.chars();
            chars.next();
            chars.next_back();
            chars.as_str().replace("''", "'")
        } else if value.starts_with("$$") {
            /* to convert to a VarChar type we have to strip the delimiters off the front and back
            of the string.  Soe remove one char (front and back) in the case of `'` and two in the case of `$$`
             */
            let mut chars = value.chars();
            chars.next();
            chars.next();
            chars.next_back();
            chars.next_back();
            chars.as_str().to_string()
        } else {
            value.to_string()
        }
    }

    /// creates an Operand::Const from an unquoted string.
    /// if the string contains a "'" it will be quoted by the "$$" pattern.  if it contains "$$" and "'"
    /// it will be quoted by the "'" pattern and all existing "'" will be replaced with "''"
    pub fn escape(txt: &str) -> Operand {
        if txt.contains('\'') {
            if txt.contains("$$") {
                Operand::Const(format!("'{}'", txt.replace('\'', "''")))
            } else {
                Operand::Const(format!("$${}$$", txt))
            }
        } else {
            Operand::Const(txt.to_string())
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Column(text)
            | Operand::Func(text)
            | Operand::Const(text)
            | Operand::Param(text) => {
                write!(f, "{}", text)
            }
            Operand::Map(entries) => {
                let mut result = String::from('{');
                result.push_str(
                    entries
                        .iter()
                        .map(|(x, y)| format!("{}:{}", x, y))
                        .join(", ")
                        .as_str(),
                );
                result.push('}');
                write!(f, "{}", result)
            }
            Operand::Set(values) => {
                let mut result = String::from('{');
                result.push_str(values.iter().join(", ").as_str());
                result.push('}');
                write!(f, "{}", result)
            }
            Operand::List(values) => {
                let mut result = String::from('[');
                result.push_str(values.iter().join(", ").as_str());
                result.push(']');
                write!(f, "{}", result)
            }
            Operand::Tuple(values) => {
                let mut result = String::from('(');
                result.push_str(values.iter().join(", ").as_str());
                result.push(')');
                write!(f, "{}", result)
            }
            Operand::Null => write!(f, "NULL"),
            Operand::Collection(operands) => write!(f, "{}", operands.iter().join(", ").as_str()),
        }
    }
}

/// data item used in `Grant`, `ListPermissions` and `Revoke` statements.
#[derive(PartialEq, Debug, Clone)]
pub struct Privilege {
    /// the privilege that is being manipulated
    pub privilege: PrivilegeType,
    /// the resource on which the permission is applied
    pub resource: Option<Resource>,
    /// the role name that tis being modified.
    pub role: Option<String>,
}

/// the list of privileges recognized by the system.
#[derive(PartialEq, Debug, Clone)]
pub enum PrivilegeType {
    All,
    Alter,
    Authorize,
    Describe,
    Execute,
    Create,
    Drop,
    Modify,
    Select,
}

impl Display for PrivilegeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivilegeType::All => write!(f, "ALL PERMISSIONS"),
            PrivilegeType::Alter => write!(f, "ALTER"),
            PrivilegeType::Authorize => write!(f, "AUTHORIZE"),
            PrivilegeType::Describe => write!(f, "DESCRIBE"),
            PrivilegeType::Execute => write!(f, "EXECUTE"),
            PrivilegeType::Create => write!(f, "CREATE"),
            PrivilegeType::Drop => write!(f, "DROP"),
            PrivilegeType::Modify => write!(f, "MODIFY"),
            PrivilegeType::Select => write!(f, "SELECT"),
        }
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Ord, PartialOrd)]
pub struct RelationElement {
    /// the column, function or column list on the left side
    pub obj: Operand,
    /// the relational operator
    pub oper: RelationOperator,
    /// the value, func, argument list, tuple list or tuple
    pub value: Operand,
}

impl Display for RelationElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.obj, self.oper, self.value)
    }
}

impl RelationOperator {
    /// evaluates the expression for any PartialOrd implementation
    pub fn eval<T>(&self, left: &T, right: &T) -> bool
    where
        T: PartialOrd,
    {
        match self {
            RelationOperator::LessThan => left.lt(right),
            RelationOperator::LessThanOrEqual => left.le(right),
            RelationOperator::Equal => left.eq(right),
            RelationOperator::NotEqual => !left.eq(right),
            RelationOperator::GreaterThanOrEqual => left.ge(right),
            RelationOperator::GreaterThan => left.gt(right),
            RelationOperator::In => false,
            RelationOperator::Contains => false,
            RelationOperator::ContainsKey => false,
            RelationOperator::IsNot => false,
        }
    }
}

/// A relation operator used in `WHERE` and `IF` clauses.
#[derive(PartialEq, Debug, Clone, Eq, PartialOrd, Ord)]
pub enum RelationOperator {
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    GreaterThan,
    In,
    Contains,
    ContainsKey,
    /// this is not used in normal cases it is used in the MaterializedView to specify
    /// a collumn that must not be null.
    IsNot,
}

impl Display for RelationOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationOperator::LessThan => write!(f, "<"),
            RelationOperator::LessThanOrEqual => write!(f, "<="),
            RelationOperator::Equal => write!(f, "="),
            RelationOperator::NotEqual => write!(f, "<>"),
            RelationOperator::GreaterThanOrEqual => write!(f, ">="),
            RelationOperator::GreaterThan => write!(f, ">"),
            RelationOperator::In => write!(f, "IN"),
            RelationOperator::Contains => write!(f, "CONTAINS"),
            RelationOperator::ContainsKey => write!(f, "CONTAINS KEY"),
            RelationOperator::IsNot => write!(f, "IS NOT"),
        }
    }
}

/// the structure of the TTL / Timestamp option.
#[derive(PartialEq, Debug, Clone)]
pub struct TtlTimestamp {
    /// the optional time-to-live value
    pub ttl: Option<u64>,
    /// the optional timestamp value
    pub timestamp: Option<u64>,
}

impl Display for TtlTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tl = match self.ttl {
            Some(t) => format!("TTL {}", t),
            _ => "".to_string(),
        };

        let tm = match self.timestamp {
            Some(t) => format!("TIMESTAMP {}", t),
            _ => "".to_string(),
        };

        if self.ttl.is_some() && self.timestamp.is_some() {
            write!(f, " USING {} AND {}", tl, tm)
        } else {
            write!(f, " USING {}", if self.ttl.is_some() { tl } else { tm })
        }
    }
}

/// The definition of the items in a WithElement
#[derive(PartialEq, Debug, Clone)]
pub enum WithItem {
    /// an option comprising the key (name) and the value for the option.
    Option { key: String, value: OptionValue },
    /// A clustering order clause.
    ClusterOrder(OrderClause),
    /// the ID the ID for the table/view.
    ID(String),
    /// use compact storage.
    CompactStorage,
}

impl Display for WithItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WithItem::Option { key, value } => write!(f, "{} = {}", key, value),
            WithItem::ClusterOrder(order) => write!(f, "CLUSTERING ORDER BY ({})", order),
            WithItem::ID(txt) => write!(f, "ID = {}", txt),
            WithItem::CompactStorage => write!(f, "COMPACT STORAGE"),
        }
    }
}

/// the order clause
#[derive(PartialEq, Debug, Clone)]
pub struct OrderClause {
    /// the column to order by.
    pub name: String,
    /// if `true` then the order is descending,
    pub desc: bool,
}

impl Display for OrderClause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.name,
            if self.desc { "DESC" } else { "ASC" }
        )
    }
}

/// the definition of an option value, is either literal string or a map of Key,value pairs.
#[derive(PartialEq, Debug, Clone)]
pub enum OptionValue {
    Literal(String),
    Map(Vec<(String, String)>),
}

impl Display for OptionValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionValue::Literal(txt) => write!(f, "{}", txt),
            OptionValue::Map(items) => write!(
                f,
                "{{{}}}",
                items.iter().map(|(x, y)| format!("{}:{}", x, y)).join(", ")
            ),
        }
    }
}

/// The definition of a primary key.
/// There must be at least one column specified in the partition.
#[derive(PartialEq, Debug, Clone)]
pub struct PrimaryKey {
    pub partition: Vec<String>,
    pub clustering: Vec<String>,
}

impl Display for PrimaryKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.partition.is_empty() && self.clustering.is_empty() {
            write!(f, "")
        } else if self.partition.len() == 1 {
            if self.clustering.is_empty() {
                write!(f, "PRIMARY KEY ({})", self.partition[0])
            } else {
                write!(
                    f,
                    "PRIMARY KEY ({}, {})",
                    self.partition[0],
                    self.clustering.join(", ")
                )
            }
        } else {
            write!(
                f,
                "PRIMARY KEY (({}), {})",
                self.partition.join(", "),
                self.clustering.join(", ")
            )
        }
    }
}

/// A list of resource types recognized by the system
#[derive(PartialEq, Debug, Clone)]
pub enum Resource {
    /// all the functins optionally within a keyspace
    AllFunctions(Option<String>),
    /// all the keyspaces
    AllKeyspaces,
    /// all the roles
    AllRoles,
    /// the specific function.
    Function(FQName),
    /// the specific keyspace
    Keyspace(String),
    /// the specified role.
    Role(String),
    /// the specified table.
    Table(FQName),
}

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Resource::AllFunctions(str) => {
                if let Some(str) = str {
                    write!(f, "ALL FUNCTIONS IN KEYSPACE {}", str)
                } else {
                    write!(f, "ALL FUNCTIONS")
                }
            }
            Resource::AllKeyspaces => write!(f, "ALL KEYSPACES"),
            Resource::AllRoles => write!(f, "ALL ROLES"),
            Resource::Function(func) => write!(f, "FUNCTION {}", func),
            Resource::Keyspace(keyspace) => write!(f, "KEYSPACE {}", keyspace),
            Resource::Role(role) => write!(f, "ROLE {}", role),
            Resource::Table(table) => write!(f, "TABLE {}", table),
        }
    }
}

pub struct WhereClause {}
impl WhereClause {
    /// return a map of column names to relation elements
    pub fn get_column_relation_element_map(
        where_clause: &[RelationElement],
    ) -> BTreeMap<String, Vec<RelationElement>> {
        let mut result: BTreeMap<String, Vec<RelationElement>> = BTreeMap::new();

        for relation_element in where_clause {
            if let Operand::Column(key) = &relation_element.obj {
                if let Some(value) = result.get_mut(key) {
                    value.push(relation_element.clone());
                } else {
                    result.insert(key.clone(), vec![relation_element.clone()]);
                }
            }
        }

        result
    }

    /// get the unordered set of column names for found in the where clause
    pub fn get_column_list(where_clause: Vec<RelationElement>) -> HashSet<String> {
        where_clause
            .into_iter()
            .filter_map(|relation_element| match relation_element.obj {
                Operand::Column(name) => Some(name),
                _ => None,
            })
            .collect()
    }
}

#[derive(PartialEq, Debug, Clone, Hash, Eq, Deserialize)]
pub struct FQName {
    pub keyspace: Option<String>,
    pub name: String,
}

impl FQName {
    pub fn simple(name: &str) -> FQName {
        FQName {
            keyspace: None,
            name: name.into(),
        }
    }

    pub fn new(keyspace: &str, name: &str) -> FQName {
        FQName {
            keyspace: Some(keyspace.into()),
            name: name.into(),
        }
    }

    /// extracts the keyspace,  Return default if none
    pub fn extract_keyspace<'a>(&'a self, default: &'a str) -> &'a str {
        if let Some(keyspace) = &self.keyspace {
            keyspace
        } else {
            default
        }
    }
}

impl Display for FQName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(keyspace) = &self.keyspace {
            write!(f, "{}.{}", keyspace, self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl From<&FQName> for std::string::String {
    fn from(fqname: &FQName) -> Self {
        fqname.to_string()
    }
}

impl From<FQName> for std::string::String {
    fn from(fqname: FQName) -> Self {
        fqname.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Operand;

    #[test]
    pub fn test_operand_unescape() {
        let tests = [
            (
                "'Women''s Tour of New Zealand'",
                "Women's Tour of New Zealand",
            ),
            (
                "$$Women's Tour of New Zealand$$",
                "Women's Tour of New Zealand",
            ),
            (
                "$$Women''s Tour of New Zealand$$",
                "Women''s Tour of New Zealand",
            ),
            ("55", "55"),
        ];
        for (arg, expected) in tests {
            assert_eq!(expected, Operand::unescape(arg).as_str());
        }
        assert_eq!(
            Operand::Null.to_string(),
            Operand::unescape(Operand::Null.to_string().as_str())
        );
    }

    #[test]
    pub fn test_operand_escape() {
        let tests = [
            (
                "$$Women's Tour of New Zealand$$",
                "Women's Tour of New Zealand",
            ),
            (
                "'Women''s Tour of New Zealand makes big $$'",
                "Women's Tour of New Zealand makes big $$",
            ),
            ("55", "55"),
        ];
        for (expected, arg) in tests {
            assert_eq!(Operand::Const(expected.to_string()), Operand::escape(arg));
        }
    }
}
