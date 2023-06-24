# The following line ensure we run from the project root
PROJECT_ROOT=$(git rev-parse --show-toplevel)
cd $PROJECT_ROOT

pallet=$1
features=""

# Check for --benchmarks flag and set features if present
while [ $# -gt 0 ]; do
    case "$1" in
    --benchmarks) features="--features runtime-benchmarks" ;;
    *) ;;
    esac
    shift
done

cargo test -p pallet-$pallet $features --tests -- --nocapture
