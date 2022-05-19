# CQL3 Parser

This is a full implementation of an Apache Cassandra CQL3 query language parser.
The goal of this package is to parse CQL3 statements into a structure that can be used in a 
multi-threaded envrionment.  It uses the tree-sitter-cql3 parser to parse the original query and
then constructs an editable thread safe representation of the query.


## Common usage

```rust
use crate::cassandra_ast::CassandraAST;
use crate::cassandra_statement::CassandraStatement;
use crate::select::{Named, SelectElement};

let ast = CassandraAST::new("select foo from myTable" );
// verify that there was no error
assert!( !ast.has_error() );
// get the parsed statement
let stmt : CassandraStatement = ast.statement;
match stmt {
    CassandraStatement::Select(select) => {
        select.columns.push( SelectElement::Column( Named {
            name : "bar".as_string(),
            alias : Some( "baz".as_string() ),
            }));
        select.order_clause = Some( OrderClause { name : "baz".as_string() } );
        },
    _ => {}
}
let edited_stmt = stmt.to_string();
```

The above code changes `SELECT foo FROM myTable` to `Select foo, bar AS baz FROM myTable ORDER BY baz ASC`.

*_NOTE_*: It is possible to create invalid statements.  If in doubt reparse the new statement to verify that it is syntactically correct.

## Package Structure

 * The parser is in the `cassandra_ast` module.
 * The Statements are in the `cassandra_statements` module.
 * The data for the statements are found in various modules named for the statement (e.g. `create_table` has the Create Table specific structs).
 * Structures that are common to several packages are found in the `common` module.
 * Many of the `Drop` statements have the same structure, it is in `common_drop`.
 * The statements dealing with Roles (e.g. `Create Role`) utilize the `role_common` module.

## A Note on Errors

When a statement is absolutely unparsable the parser will return a `CassandraStatement::Unknown`
object.  For example `CassandraAST::new("This is an invalid statement");` yields 
`CassandraStatement::Unknown("This is an invalid statement")`.  However, if a statement is 
partially parsable multiple results are returned.  For example `CassandraAST::new("SELECT * FROM foo WHERE some invalid part");` yields
`CassandraStatement::Select( select )` where the select is the result of parsing `"SELECT * FROM foo` followed  by 
`CassandraStatement::Unknown("SELECT * FROM foo WHERE some invalid part")`.

