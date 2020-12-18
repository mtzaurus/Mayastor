#!/usr/bin/env bash

## e2e tests disabled until we can make them more reliable
# exit 0

set -eux

SCRIPTDIR=$(dirname "$(realpath "$0")")
TESTS="install basic_volume_io csi uninstall"
DEVICE=
REGISTRY=
TAG=
TESTDIR=$(realpath "$SCRIPTDIR/../test/e2e")

help() {
  cat <<EOF
Usage: $0 [OPTIONS]

Options:
  --device <path>           Device path to use for storage pools.
  --registry <host[:port]>  Registry to pull the mayastor images from.
  --tag <name>              Docker image tag of mayastor images (default "ci")
  --tests <list of tests>   Lists of tests to run, delimited by spaces (default: "$TESTS")

Examples:
  $0 --registry 127.0.0.1:5000 --tag a80ce0c
EOF
}

# Parse arguments
while [ "$#" -gt 0 ]; do
  case "$1" in
    -d|--device)
      shift
      DEVICE=$1
      ;;
    -r|--registry)
      shift
      REGISTRY=$1
      ;;
    -t|--tag)
      shift
      TAG=$1
      ;;
    -T|--tests)
      shift
      TESTS="$1"
      ;;
    -h|--help)
      help
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      help
      exit 1
      ;;
  esac
  shift
done

if [ -z "$DEVICE" ]; then
  echo "Device for storage pools must be specified"
  help
  exit 1
fi
export e2e_pool_device=$DEVICE
if [ -n "$TAG" ]; then
  export e2e_image_tag="$TAG"
fi
if [ -n "$REGISTRY" ]; then
  export e2e_docker_registry="$REGISTRY"
fi

test_failed=0

# Run go test in directory specified as $1 (relative path)
function runGoTest {
    cd "$TESTDIR"
    echo "Running go test in $PWD/\"$1\""
    if [ -z "$1" ] || [ ! -d "$1" ]; then
        return 1
    fi

    cd "$1"
    if ! go test -v . -ginkgo.v -ginkgo.progress -timeout 0; then
        return 1
    fi

    return 0
}

# Check if $2 is in $1
contains() {
    [[ $1 =~ (^|[[:space:]])$2($|[[:space:]]) ]] && return 0  || return 1
}

for dir in $TESTS; do
  # defer uninstall till after other tests have been run.
  if [ "$dir" != "uninstall" ] ;  then
      if ! runGoTest "$dir" ; then
          test_failed=1
          break
      fi

      if ! ("$SCRIPTDIR"/e2e_check_pod_restarts.sh) ; then
          test_failed=1
          break
      fi
  fi
done

if [ "$test_failed" -ne 0 ]; then
    if ! "$SCRIPTDIR"/e2e-cluster-dump.sh ; then
        # ignore failures in the dump script
        :
    fi
fi

# Always run uninstall test if specified
if contains "$TESTS" "uninstall" ; then
    if ! runGoTest "uninstall" ; then
        test_failed=1
        if ! "$SCRIPTDIR"/e2e-cluster-dump.sh --clusteronly ; then
            # ignore failures in the dump script
            :
        fi
    fi
fi

if [ "$test_failed" -ne 0 ]; then
    echo "At least one test has FAILED!"
  exit 1
fi

echo "All tests have PASSED!"
exit 0
