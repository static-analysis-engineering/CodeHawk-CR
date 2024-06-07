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
)}
