extern crate diesel;
#[macro_use]
extern crate structopt;
extern crate sxd_document;
extern crate sxd_xpath;

use diesel::prelude::*;
use structopt::StructOpt;

mod sped;
mod xsd;

fn main() {
    let opt = Opt::from_args();
    let conn = PgConnection::establish(&opt.database_url)
        .expect(&format!("Error connecting to {}", &opt.database_url));
    match opt.cmd {
        FileType::Sped {
            ref specification_file,
        } => sped::run(specification_file, &conn, &opt.document_type),
        FileType::Xsd { ref package_url } => xsd::run(package_url, &conn, &opt.document_type),
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "specfile-cli")]
struct Opt {
    /// URL for the target database
    #[structopt(short = "d", long = "database-url")]
    database_url: String,
    /// Name of the specification file's document type
    #[structopt(short = "t", long = "document-type")]
    document_type: String,
    #[structopt(subcommand)]
    cmd: FileType,
}

#[derive(StructOpt, Debug)]
enum FileType {
    /// for sped specification files
    #[structopt(name = "sped")]
    Sped {
        #[structopt(short = "m", long = "specification-file")]
        specification_file: String,
    },
    /// for xml schema files
    #[structopt(name = "xsd")]
    Xsd {
        #[structopt(short = "u", long = "package-url")]
        package_url: String,
    },
}
