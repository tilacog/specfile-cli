use diesel::prelude::*;
use std::path::Path;

pub fn run<P: AsRef<Path>>(path: P, conn: &PgConnection, document_type: &str) {
    unimplemented!();
}
