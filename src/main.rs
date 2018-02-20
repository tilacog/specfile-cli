extern crate diesel;
#[macro_use]
extern crate structopt;
extern crate sxd_document;
extern crate sxd_xpath;

use diesel::prelude::*;

use structopt::StructOpt;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

fn main() {
    let opt = Opt::from_args();
    let conn = PgConnection::establish(&opt.database_url)
        .expect(&format!("Error connecting to {}", &opt.database_url));
    let content = {
        let mut s = String::new();
        File::open(&opt.file)
            .expect("failed to open file")
            .read_to_string(&mut s)
            .expect("failed to read file");
        s
    };
    let (id, version) = infer_id_and_version(&content);
    insert_spec_file(&conn, &opt.document_type, &version, &id, &content);
}

#[derive(StructOpt, Debug)]
#[structopt(name = "specfile-cli",
            about = "Command line utility for inserting specification files into database.")]
struct Opt {
    #[structopt(help = "Input XML file", parse(from_os_str))]
    file: PathBuf,
    #[structopt(short = "d", long = "database-url", help = "URL for the target database")]
    database_url: String,
    #[structopt(short = "t", long = "document-type",
                help = "Name of the specification file's document type")]
    document_type: String,
}

fn infer_id_and_version(content: &str) -> (String, String) {
    use sxd_document::parser;
    use sxd_xpath::evaluate_xpath;

    let package = parser::parse(&content).expect("failed to parse XML");
    let document = package.as_document();

    let id = evaluate_xpath(&document, "/descritor-escrituracao/@versao")
        .expect("XPath evaluation failed");
    let version =
        evaluate_xpath(&document, "/descritor-escrituracao/@id").expect("XPath evaluation failed");

    (id.string(), version.string())
}

fn insert_spec_file(
    conn: &PgConnection,
    document_type: &str,
    version: &str,
    version_id: &str,
    specification: &str,
) {
    use diesel::sql_query;
    use diesel::sql_types::Text;
    use diesel::result::Error;
    use diesel::result::DatabaseErrorKind;

    let q = sql_query(
        "INSERT INTO specification.sped (document_type, version, version_id, specification) \
         VALUES ($1, $2, $3, $4::xml);",
    ).bind::<Text, _>(document_type)
        .bind::<Text, _>(version)
        .bind::<Text, _>(version_id)
        .bind::<Text, _>(specification);

    match q.execute(conn) {
        Ok(_) => println!("Done."),
        Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _info)) => {
            eprintln!("Error: specification already exists in database.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error:{}", e);
            std::process::exit(2);
        }
    }
}
