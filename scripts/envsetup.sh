SOURCED_PATH=$_
REAL_SOURCE=$(realpath "$SOURCED_PATH")
SOURCE_DIR=$(dirname "$REAL_SOURCE")
PROJECT_DIR=$(dirname "$SOURCE_DIR")
export PATH="$PROJECT_DIR"/chc/cmdline:"$PATH"
export PYTHONPATH="$PROJECT_DIR"
if [ ! -d "$PROJECT_DIR"/.venv ]; then
	python3 -m venv $PROJECT_DIR/.venv
fi
source "$PROJECT_DIR"/.venv/bin/activate

chc_build () {(
	cd "$PROJECT_DIR"
	# Filter emoji out of maturin because they mess up my gnu screen session
	set -e
	set -o pipefail
	maturin develop --color=always 2>&1 | tr -cd '\000-\177'
)}

chc_test () {(
	set -e
	chc_build
	chkc kendra test-sets
)}

chc_test_parallel () {(
	set -e
	chc_build
	chkc kendra list | grep '  id' | parallel --trim l chkc kendra test-set
)}

chc_demo () {(
	set -e
	chc_build
	cd $PROJECT_DIR/tests
	rm -r demo.build
	cp -r demo demo.build
	cd demo.build
	ninja
	ninja -t compdb > compile_commands.json
	chkc c-project parse ./ codehawk_demo
	chkc c-project analyze ./ codehawk_demo
	chkc c-project report ./ codehawk_demo
	chkc c-project make-callgraph ./ codehawk_demo
)}

chc_tables () {(
	set -e
	chc_build
	cd $PROJECT_DIR/tests
	rm -r demo.build
	cp -r demo demo.build
	cd demo.build
	ninja
	for FILE in $(ls *.c); do
		chkc c-file parse $FILE
		chkc c-file analyze $FILE
		chkc c-file report $FILE
		
		chkc c-file file-table api-param $FILE
		chkc c-file file-table attrparam $FILE
		chkc c-file file-table attribute $FILE
		chkc c-file file-table attributes $FILE
		chkc c-file file-table cfg-context $FILE
		# chkc c-file file-table compinfo $FILE # issue with types.c
		chkc c-file file-table constant $FILE
		chkc c-file file-table enuminfo $FILE
		chkc c-file file-table enumitem $FILE
		chkc c-file file-table exp $FILE
		chkc c-file file-table exp-context $FILE
		# chkc c-file file-table fieldinfo $FILE # issue with types.c
		chkc c-file file-table funarg $FILE
		chkc c-file file-table funargs $FILE
		chkc c-file file-table initinfo $FILE
		chkc c-file file-table lhost $FILE
		chkc c-file file-table location $FILE
		chkc c-file file-table lval $FILE
		chkc c-file file-table offset $FILE
		chkc c-file file-table offsetinfo $FILE
		chkc c-file file-table po-predicate $FILE
		chkc c-file file-table post-assume $FILE
		chkc c-file file-table post-request $FILE
		chkc c-file file-table program-context $FILE
		chkc c-file file-table s-offset $FILE
		chkc c-file file-table s-term $FILE
		chkc c-file file-table typeinfo $FILE
		chkc c-file file-table typsig $FILE
		chkc c-file file-table typsig-list $FILE
		chkc c-file file-table varinfo $FILE
		chkc c-file file-table xpredicate $FILE
	done
)}
