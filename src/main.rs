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
    let version = infer_version(&content);
    insert_spec_file(&conn, &opt.document_type, version, &content);
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

struct Version {
    major: i32,
    minor: i32,
    patch: i32,
}

fn infer_version(content: &str) -> Version {
    use sxd_document::parser;
    use sxd_xpath::evaluate_xpath;

    let package = parser::parse(&content).expect("failed to parse XML");
    let document = package.as_document();

    let patch = evaluate_xpath(&document, "/descritor-escrituracao/@versao")
        .expect("XPath evaluation failed")
        .string()
        .parse::<i32>()
        .expect("couldn't parse patch number");

    let major_and_minor = evaluate_xpath(&document, "/descritor-escrituracao/@id")
        .expect("XPath evaluation failed")
        .string();
    let (major, minor) = {
        let n = 3;
        if major_and_minor.len() < n {
            panic!("version string is too short: {:?}", major_and_minor)
        }
        let mut chars = major_and_minor.chars().collect::<Vec<char>>();
        let new_len = chars.len() - n;
        let tail = chars.drain(new_len..).collect::<Vec<char>>();

        let major = chars
            .into_iter()
            .collect::<String>()
            .parse::<i32>()
            .expect("parse error");
        let minor = tail.into_iter()
            .collect::<String>()
            .parse::<i32>()
            .expect("parse error");

        (major, minor)
    };
    Version {
        major,
        minor,
        patch,
    }
}

fn insert_spec_file(
    conn: &PgConnection,
    document_type: &str,
    version: Version,
    specification: &str,
) {
    use diesel::sql_query;
    use diesel::sql_types::{Int4, Text};
    use diesel::result::Error;
    use diesel::result::DatabaseErrorKind;

    let q = sql_query(
        "INSERT INTO specification.sped (document_type, major, minor, patch, specification) \
         VALUES ($1, $2, $3, $4, $5::xml);",
    ).bind::<Text, _>(document_type)
        .bind::<Int4, _>(version.major)
        .bind::<Int4, _>(version.minor)
        .bind::<Int4, _>(version.patch)
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
