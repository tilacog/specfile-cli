#!/usr/bin/env bash
# Searches a given directory for sped specification files and tries to insert them on a database

set -o errexit
set -o pipefail
set -o nounset
# set -o xtrace

TARGET_DIRECTORY=$1
DATABASE_URI="postgres://postgres@localhost/postgres"
declare -a document_types=("ecd" "ecf" "efd" "efd_contrib")

for DT_NAME in "${document_types[@]}"
do
    FILES=$(find "${TARGET_DIRECTORY}" -type f -iname "${DT_NAME}*")
    for FILE in $FILES
    do
	./target/release/specfile-cli "${FILE}" --database-url "${DATABASE_URI}" --document-type "${DT_NAME}" || true
    done
done
