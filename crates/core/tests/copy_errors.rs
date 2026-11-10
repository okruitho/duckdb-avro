//! `COPY ... TO (FORMAT AVRO)`: the errors of invalid statements, options and data.

use std::path::PathBuf;

use duckdb_neo::{
    Parameters, Result,
    connection::Connection,
    environment::{Environment, StorageLocation},
    error::DuckDBError,
};

fn connect() -> Result<Connection> {
    let conn = Environment::new()?.open(StorageLocation::InMemory)?.connect()?;
    avro_core::copy::copy_function().register(&conn)?;
    Ok(conn)
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("avro_copy_errors_{}_{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_error(conn: &Connection, dir: &std::path::Path, query: &str, options: &str) -> duckdb_neo::Error {
    let target = dir.join("out.avro");
    let sql = format!("COPY ({query}) TO '{}' ({options})", target.to_str().unwrap());
    match conn.execute(sql.as_str(), Parameters::None) {
        Ok(_) => panic!("expected an error from {sql}"),
        Err(error) => error,
    }
}

#[track_caller]
fn assert_error(error: &duckdb_neo::Error, code: DuckDBError, message: &str) {
    assert_eq!(error.code, code, "{}", error.message);
    assert!(error.message.contains(message), "{:?} does not contain {:?}", error.message, message);
}

#[test]
fn probe() -> Result<()> {
    let conn = connect()?;
    let dir = test_dir("probe");
    let error = copy_error(&conn, &dir, "SELECT 42 a", "FORMAT AVRO, NOT_RECOGNIZED 'with_value', ROOT_NAME 'test'");
    println!("{:?}", error);
    let error = copy_error(&conn, &dir, "SELECT NULL field1", "FORMAT AVRO, FIELD_IDS {'field1': {'__duckdb_field_id': 301, '__duckdb_nullable': 'foo'}}");
    println!("{:?}", error);
    let error = copy_error(&conn, &dir, "SELECT NULL::INTEGER field1", "FORMAT AVRO, FIELD_IDS {'field1': {'__duckdb_field_id': 301, '__duckdb_nullable': false}}");
    println!("{:?}", error);
    let error = copy_error(&conn, &dir, "SELECT 1 a", "FORMAT AVRO, ROOT_NAME NULL::VARCHAR");
    println!("{:?}", error);
    let error = copy_error(&conn, &dir, "SELECT 1 a", "FORMAT AVRO, FIELD_IDS {'a': NULL}");
    println!("{:?}", error);
    let error = copy_error(&conn, &dir, "SELECT 1 a", "FORMAT AVRO, FIELD_IDS {'a': {'__duckdb_nullable': false}}");
    println!("{:?}", error);
    assert_error(&error, DuckDBError::RuntimeInternal, "x");
    Ok(())
}
